from collections.abc import Sequence
from pathlib import Path
from typing import Any

from attrs import Factory, define, evolve, field
from loguru import logger

from winter import nix
from winter.compiler.rules.base import BaseRule
from winter.compiler.rules.list_inherits import ListInheritsRule
from winter.compiler.rules.locals import LocalsRule
from winter.parser.ast import (
    AST,
    CallableHead,
    Context,
    FuncParam,
    InheritNode,
    OneBind,
    Project,
    StringLikeAST,
    TypedReference,
    UnwrappedParam,
)

from .types import OPERATOR_MAPPING, UNARY_OPERATORS, OpSpecialCase


@define
class CompilerError(RuntimeError):
    msg: str
    wrapped_errors: Sequence[Exception] = field(default=())


EXPECTED_NON_EMPTY_SEL = (
    "Expected at least two nodes within a selection (head and tail)"
)
UNEXPECTED_NONE_OPTIM = "Unexpected `None` after AST optimization"
SKIPPING_CONTEXT = "Skipping Context `{}` since no Nix root AST was compiled."


def _mk_rules() -> list[BaseRule[Any]]:
    return [
        LocalsRule(),
        ListInheritsRule(),
    ]


@define
class CompilerOutput:
    tree: dict[Path, nix.AST.Type]


@define
class Compiler:
    project: Project
    error_t: type[CompilerError] = field(default=CompilerError, init=False)
    rules: list[BaseRule[Any]] = field(default=Factory(_mk_rules), init=False)
    _input_tree: dict[Path, Context] = field(init=False)
    _output_tree: dict[Path, nix.AST.Type] = field(init=False)

    def __attrs_post_init__(self) -> None:
        self._input_tree = {
            (c.file.relative_to(self.project.dir)): c for c in self.project.contexts
        }
        self._output_tree = {}

    type _Res[X] = tuple[X | None, CompilerError | None]
    type _NixRes = _Res[nix.AST.Type | None]

    def compile(self) -> _Res[CompilerOutput]:
        inp_tree, err = self._optimize_tree(self._input_tree)
        if inp_tree is None or err:
            return self._error("Error during `_optimize_tree`", err)
        self._input_tree = inp_tree
        self._output_tree = {}

        for ctx_key in self._input_tree:
            self._compile_context(ctx_key)

        return CompilerOutput(tree=self._output_tree), None

    def _compile_context(self, ctx_key: Path) -> None:
        ctx = self._input_tree[ctx_key]
        # ast, by this point, should be already fully optimized
        nix_ast: nix.AST.Type | None = None

        for node in ctx.body:
            if nix_ast is not None:
                msg = (
                    f"At Context `{ctx_key}`, Winter couldn't compile into "
                    "one top-level Nix value."
                )
                raise CompilerError(msg)
            nix_ast, err = self._parse_one_ast(node)
            if nix_ast is None or err:
                msg = f"Couldn't compile Context `{ctx_key}`"
                raise CompilerError(
                    msg,
                    wrapped_errors=(err,) if err else (),
                )
        if nix_ast is None:
            logger.warning(SKIPPING_CONTEXT, ctx_key)
        else:
            self._output_tree[ctx_key] = nix_ast

    def _optimize_tree(self, tree: dict[Path, Context]) -> _Res[dict[Path, Context]]:
        out: dict[Path, Context] = {}
        for k, v in tree.items():
            out_ctx, err = self._optimize_context(v)
            if out_ctx is None or err:
                return self._error("Error during `_optimize_context`", err)
            out[k] = out_ctx

        return out, None

    def _optimize_context(self, ctx: Context) -> _Res[Context]:
        out_body: list[AST.Type] = []

        for node in ctx.body:
            out_node, err = self._optimize_ast(node)
            if out_node is None or err:
                return self._error("Error during `_optimize_ast` call", err)
            assert out_node
            out_body.append(out_node)
        return evolve(ctx, body=out_body), None

    def _error(
        self,
        msg: str,
        *other_errors: Exception | None,
    ) -> tuple[None, CompilerError]:
        return None, CompilerError(
            msg=msg,
            wrapped_errors=[x for x in other_errors if x],
        )

    # compiler optimizations and source pre-processing
    def _optimize_ast(self, x: AST.Type) -> _Res[AST.Type]:
        for rule in self.rules:
            out, err = rule.run(x)
            if err:
                msg = f"Error during rule '{rule!r}': {err!r}"
                return self._error(msg)
            if out is None:
                msg = f"Rule '{rule!r}' didn't return an AST!"
                return self._error(msg)
            x = out
        return x, None

    def _parse_one_ast(self, x: AST.Type) -> _NixRes:
        match x:
            case AST.NIL():
                return nix.AST.NULL(), None
            case AST.BOOL(value=v):
                return nix.AST.BOOL(value=v), None
            case AST.NUMBER(value=v):
                return nix.AST.NUMBER(value=v), None
            case AST.IDENTIFIER(value=v):
                return nix.AST.IDENTIFIER(value=v), None
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
                ), None
            case AST.LIST(items=items):
                nix_items, errors = self._parse_list_like(items)
                if nix_items is None:
                    return None, errors
                return nix.AST.LIST(entries=nix_items), None
            case AST.SELECTION(items=items):
                if len(items) < 2:
                    return self._error(EXPECTED_NON_EMPTY_SEL)
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
                ), None
            case AST.MAP(binds=binds, inherits=inherits):
                entries, errors = self._parse_bindings((*binds, *inherits))
                if entries is None:
                    return None, errors
                return nix.AST.ATTRSET(entries=entries), None
            case AST.CALLABLE():
                return self._parse_callable(x)
            case AST.LET(binds=binds, inherits=inherits, body=body):
                entries, errors = self._parse_bindings((*binds, *inherits))
                if entries is None:
                    return None, errors
                nix_body, errors = self._parse_one_ast(body)
                if nix_body is None:
                    return None, errors
                return nix.AST.LET(entries=entries, body=nix_body), None
            case AST.FUNCTION():
                return self._parse_fn(x)
            case AST.IF(
                condition=condition,
                case_true=case_true,
                case_false=case_false,
            ):
                nix_cond, errors = self._parse_one_ast(condition)
                if nix_cond is None:
                    return None, errors
                nix_case_true, errors = self._parse_one_ast(case_true)
                if nix_case_true is None:
                    return None, errors
                nix_case_false, errors = self._parse_one_ast(case_false)
                if nix_case_false is None:
                    return None, errors
                return nix.AST.IF(
                    condition=nix_cond,
                    case_true=nix_case_true,
                    case_false=nix_case_false,
                ), None
            case AST.LOCALS() | AST.INHERITS():
                raise RuntimeError(
                    "Optimizations should have taken care of locals/inherits!",
                )

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
                ), None
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
                ), None
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
                ), None
            case _:
                ref = self._parse_typed_ref(x)
                return nix.LambdaArg.PLAIN(value=ref), None

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
        return unw_params, None

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
                return nix.AST.APPLY(function=nix_head, args=nix_tail), None

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
                    return self._error(msg)
                return nix.AST.UNARY_OP(
                    value=unary_val,
                    op=sym,
                ), None
            case [lhs, rhs]:
                return nix.AST.BINARY_OP(lhs=lhs, rhs=rhs, op=sym), None
            case [lhs, *tail]:
                rhs, errors = self._parse_callable_op(op=op, nix_tail=tail)
                if rhs is None:
                    return None, errors
                return nix.AST.BINARY_OP(
                    lhs=lhs,
                    rhs=rhs,
                    op=sym,
                ), None
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
        return nix_binds, None

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
        return nix_list, None

    def _handle_str_part(self, x: StringLikeAST.Type) -> _Res[nix.StringLikePart.Type]:
        match x:
            case StringLikeAST.TEXT(value=v):
                return nix.StringLikePart.TEXT(value=v), None
            case StringLikeAST.EXPR(value=v):
                expr, err = self._parse_one_ast(v)
                if expr is None or err:
                    return self._error(
                        "Error during string part compilation",
                        err,
                    )
                return nix.StringLikePart.EXPR(value=expr), None

    def _todo(self) -> _NixRes:
        raise NotImplementedError
