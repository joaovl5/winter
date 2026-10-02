from collections.abc import Sequence
from typing import override

from attrs import define, field
from loguru import logger

from winter import nix
from winter.parser.ast import (
    AST,
    CallableHead,
    FuncParam,
    InheritNode,
    OneBind,
    StringLikeAST,
    TypedReference,
    UnwrappedParam,
)
from winter.utils.voyager import Voyager

from .types import OPERATOR_MAPPING, UNARY_OPERATORS, OpSpecialCase


class CompilerError(RuntimeError): ...


EXPECTED_ONE_NODE = "Expected a single node within the parsed string interp"
EXPECTED_NON_EMPTY_SEL = (
    "Expected at least two nodes within a selection (head and tail)"
)


@define
class Compiler(Voyager[AST.Type, nix.AST.Type, CompilerError]):
    error_t: type[CompilerError] = field(default=CompilerError, init=False)

    type _Res[X] = Compiler.Result[X]
    type _NixRes = _Res[nix.AST.Type | None]

    @override
    def _parse_one(self) -> _NixRes:
        if (x := self._next()) is None:
            return None, []
        return self._parse_one_ast(x)

    # compiler optimizations and source pre-processing
    def _optimize_ast(self, x: AST.Type) -> _Res[AST.Type | None]:
        pass

    def _parse_one_ast(self, x: AST.Type) -> _NixRes:
        match x:
            case AST.NIL():
                return nix.AST.NULL(), []
            case AST.BOOL(value=v):
                return nix.AST.BOOL(value=v), []
            case AST.NUMBER(value=v):
                return nix.AST.NUMBER(value=v), []
            case AST.IDENTIFIER(value=v):
                return nix.AST.IDENTIFIER(value=v), []
            case AST.STRING_LIKE(parts=parts, is_path=is_path):
                nix_parts: list[nix.StringLikePart.Type] = []
                for part in parts:
                    nix_part, errors = self._handle_str_part(part)
                    if nix_part is None:
                        return None, errors
                    nix_parts.append(nix_part)
                return nix.AST.STRING_LIKE(
                    value=nix_parts,
                    is_path=is_path,
                ), []
            case AST.LIST(items=items):
                nix_items, errors = self._parse_list_like(items)
                if nix_items is None:
                    return None, errors
                return nix.AST.LIST(entries=nix_items), []
            case AST.SELECTION(items=items):
                if len(items) < 2:
                    return None, self._single_err(EXPECTED_NON_EMPTY_SEL)
                head, tail = items[0], items[1:]

                nix_head, errors = self._parse_one_ast(head)
                if nix_head is None:
                    return None, errors
                nix_tail, errors = self._parse_list_like(tail)
                if nix_tail is None:
                    return None, errors
                return nix.AST.SELECTION(
                    base=nix_head,
                    path=nix_tail,
                ), []
            case AST.MAP(binds=binds, inherits=inherits):
                entries, errors = self._parse_bindings((*binds, *inherits))
                if entries is None:
                    return None, errors
                return nix.AST.ATTRSET(entries=entries), []
            case AST.CALLABLE():
                return self._parse_callable(x)
            case AST.LET(binds=binds, inherits=inherits, body=body):
                entries, errors = self._parse_bindings((*binds, *inherits))
                if entries is None:
                    return None, errors
                nix_body, errors = self._parse_one_ast(body)
                if nix_body is None:
                    return None, errors
                return nix.AST.LET(entries=entries, body=nix_body), []
            case AST.LOCALS() | AST.INHERITS():
                return self._todo()
            case AST.FUNCTION():
                return self._parse_fn(x)

            case _:
                return self._todo()

    def _parse_fn(self, x: AST.FUNCTION) -> _NixRes:
        nix_body_lst, errors = self._parse_list_like(x.body)
        if nix_body_lst is None:
            return None, errors
        nix_body: nix.AST.Type
        match nix_body_lst:
            case []:
                raise RuntimeError("function body was empty")
            case [node]:
                nix_body = node
            case _:
                raise RuntimeError("function body had more than one node")
        nix_params: list[nix.LambdaArg.Type] = []
        for param in x.params:
            nix_param, errors = self._parse_fn_param(param)
            if nix_param is None:
                return None, errors
            nix_params.append(nix_param)
        return self._parse_fn_inner(nix_body=nix_body, nix_params=nix_params)

    def _parse_fn_inner(
        self,
        nix_body: nix.AST.Type,
        nix_params: list[nix.LambdaArg.Type],
    ) -> _NixRes:
        match nix_params:
            case []:
                raise RuntimeError("function args were empty")
            case [arg]:
                return nix.AST.LAMBDA(
                    arg=arg,
                    body=nix_body,
                ), []
            case [arg, *rest]:
                nested, errors = self._parse_fn_inner(
                    nix_body=nix_body,
                    nix_params=rest,
                )
                if nested is None:
                    return None, errors
                return nix.AST.LAMBDA(
                    arg=arg,
                    body=nested,
                ), []
            case _:
                raise RuntimeError("assumedly unreachable state")

    def _parse_fn_param(self, x: FuncParam.Type) -> _Res[nix.LambdaArg.Type | None]:
        match x:
            case FuncParam.UNWRAPPED(
                name=name,
                params=params,
                allow_extra=allow_extra,
            ):
                unw_params, errors = self._parse_unwrapped_params(params)
                if unw_params is None:
                    return None, errors
                return nix.LambdaArg.UNWRAPPED(
                    name=self._parse_typed_ref(name) if name else None,
                    params=unw_params,
                    allow_extra=allow_extra,
                ), []
            case _:
                ref = self._parse_typed_ref(x)
                return nix.LambdaArg.PLAIN(value=ref), []

    def _parse_unwrapped_params(
        self,
        params: list[UnwrappedParam],
    ) -> _Res[list[tuple[str, nix.AST.Type | None]] | None]:
        unw_params: list[tuple[str, nix.AST.Type | None]] = []
        for p_ref, p_def in params:
            nix_def = None
            if p_def is not None:
                nix_def, errors = self._parse_one_ast(p_def)
                if nix_def is None:
                    return None, errors
            unw_params.append((self._parse_typed_ref(p_ref), nix_def))
        return unw_params, []

    def _parse_callable(self, x: AST.CALLABLE) -> _NixRes:
        nix_tail, errors = self._parse_list_like(x.tail)
        if nix_tail is None:
            return None, errors
        match x.head:
            case CallableHead.OPERATOR() as op:
                return self._parse_callable_op(op=op, nix_tail=nix_tail)
            case _:  # expr
                nix_head, errors = self._parse_one_ast(x.head)
                if nix_head is None:
                    return None, errors
                return nix.AST.APPLY(function=nix_head, args=nix_tail), []

    def _parse_callable_op(
        self,
        op: CallableHead.OPERATOR,
        nix_tail: list[nix.AST.Type],
        *,
        unary_ok: bool = True,
    ) -> _NixRes:
        sym = OPERATOR_MAPPING[op]
        if isinstance(sym, OpSpecialCase):
            return self._todo()
        match nix_tail:
            case []:
                raise RuntimeError("empty op tail should never happen")
            case [unary_val]:
                if not unary_ok:
                    raise RuntimeError("unary is not ok in this situation")
                if op not in UNARY_OPERATORS:
                    msg = f"Operator `{sym}` cannot be unary!"
                    return None, self._single_err(msg)
                return nix.AST.UNARY_OP(
                    value=unary_val,
                    op=sym,
                ), []
            case [lhs, rhs]:
                return nix.AST.BINARY_OP(lhs=lhs, rhs=rhs, op=sym), []
            case [lhs, *tail]:
                rhs, errors = self._parse_callable_op(op=op, nix_tail=tail)
                if rhs is None:
                    return None, errors
                return nix.AST.BINARY_OP(
                    lhs=lhs,
                    rhs=rhs,
                    op=sym,
                ), []
            case _:
                raise RuntimeError("Unexpected callable case")

    def _parse_bindings(
        self, binds: Sequence[AST.MAP.MapBind | InheritNode | OneBind]
    ) -> _Res[list[nix.Binding.Type] | None]:
        nix_binds: list[nix.Binding.Type] = []
        for bind in binds:
            nix_bind: nix.Binding.Type
            match bind:
                case (str(ns), xs):  # inherit
                    nix_bind = (
                        ns,
                        [self._parse_typed_ref(ref) for ref in xs]
                        if xs is not None
                        else None,
                    )
                case (list(path_parts), x):  # map bind
                    nix_path_parts, errors = self._parse_list_like(path_parts)
                    if nix_path_parts is None:
                        return None, errors
                    nix_x, errors = self._parse_one_ast(x)
                    if nix_x is None:
                        return None, errors
                    nix_bind = nix_path_parts, nix_x
                case (TypedReference(name=name), x):  # normal bind
                    parts: list[nix.AST.Type] = []
                    parts.append(nix.AST.IDENTIFIER(value=name))
                    nix_x, errors = self._parse_one_ast(x)
                    if nix_x is None:
                        return None, errors
                    nix_bind = (parts, nix_x)
            nix_binds.append(nix_bind)
        return nix_binds, []

    def _parse_typed_ref(self, ref: TypedReference) -> str:
        return ref.name

    def _parse_list_like(
        self, list: Sequence[AST.Type]
    ) -> _Res[list[nix.AST.Type] | None]:
        nix_list: list[nix.AST.Type] = []
        for item in list:
            nix_item, errors = self._parse_one_ast(item)
            if nix_item is None:
                return None, errors
            nix_list.append(nix_item)
        return nix_list, []

    def _handle_str_part(
        self, x: StringLikeAST.Type
    ) -> _Res[nix.StringLikePart.Type | None]:
        match x:
            case StringLikeAST.TEXT(value=v):
                return nix.StringLikePart.TEXT(value=v), []
            case StringLikeAST.EXPR(value=v):
                exprs, errs = Compiler(
                    source=iter((v,)),
                ).parse_total()
                match exprs:
                    case [expr]:
                        return nix.StringLikePart.EXPR(value=expr), errs
                    case _:
                        return None, [
                            *errs,
                            *self._single_err(EXPECTED_ONE_NODE),
                        ]

    @override
    def _resolve_some_pos(self, pos: int) -> tuple[int, int]:
        node = self._get_source_elem(pos)
        if node is None:
            if pos > 0:
                return self._resolve_some_pos(pos - 1)
            else:
                logger.warning("Had to fallback to (0, 0)")
                return 0, 0
        return node.pos_start, node.pos_end

    def _todo(self) -> _NixRes:
        raise NotImplementedError
