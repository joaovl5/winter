from collections.abc import Sequence
from typing import TYPE_CHECKING

from attrs import define

from winter.lexer.lexemes import Lexeme
from winter.parser.ast import AST, FuncParam, TypedReference, UnwrappedParam

if TYPE_CHECKING:
    from .parser import Parser

UNTERMINATED_FN = "Unterminated function definition"
UNTERMINATED_FN_PARAMS = "Unterminated function params"
EXPECTED_AT = "Expected `@` sign for naming unpacked argument"
EXPECTED_BRACE = "Expected `{` for unpacked argument"  # }
EXPECTED_ATOM = "Expected identifier for param"
UNEXPECTED_UNPACKED_TYPE_SIG = "Unpacked function param cannot have type signature"
UNEXPECTED_DOUBLE_ELLIPSIS = "Cannot have more than one ellipsis in unpacked param"
UNEXPECTED_TOKEN_PARAMS = "Unexpected token where function params should be"
EMPTY_BODY = "Function body must not be empty"
EMPTY_PARAMS = "Function must have at least one param"


@define
class FuncParser:
    p: Parser
    type _Res[X] = Parser._Res[X]
    type _ASTRes = Parser._ASTRes

    def parse_fn(self) -> _ASTRes:
        all_errors = self.p._mk_err_box()
        _ = self.p._next()  # in `fn`
        pos_start, _ = self.p._get_resolved_pos()

        # potentially parse the name, if it exists
        if (name_tk := self.p._peek()) is None:
            return None, self.p._single_err(UNTERMINATED_FN)
        name: TypedReference | None = None
        if isinstance(name_tk, Lexeme.ATOM):
            name, errors = self.p._ref.parse_typed_ref()
            all_errors.extend(errors)
            if name is None:
                return None, all_errors
        params, errors = self._params()
        if params is None:
            return None, errors
        if len(params) == 0:
            return None, self.p._single_err(EMPTY_PARAMS)
        body: list[AST.Type] = []
        while True:
            if (next_tk := self.p._peek()) is None:
                return None, self.p._single_err(UNTERMINATED_FN)
            if isinstance(next_tk, Lexeme.CLOSE_PAREN):
                break
            ast_value, errors = self.p._parse_one()
            if ast_value is None:
                return None, self.p._single_err(UNTERMINATED_FN)
            body.append(ast_value)
        _ = self.p._next()
        if len(body) == 0:
            return None, self.p._single_err(EMPTY_BODY)
        _, pos_end = self.p._get_resolved_pos(-1)
        return AST.FUNCTION(
            pos_start=pos_start,
            pos_end=pos_end,
            name=name,
            params=list(params),
            body=body,
        ), []

    def _params(self) -> _Res[Sequence[FuncParam.Type] | None]:
        if (p_start_tk := self.p._peek()) is None:
            return None, self.p._single_err(UNTERMINATED_FN)
        match p_start_tk:
            case Lexeme.ATOM():
                return self._p_unpacked_named()
            case Lexeme.OPEN_BRACE():
                return self._p_unpacked()
            case Lexeme.OPEN_BRACKET():
                return self._p_list()
            case _:
                return None, self.p._single_err(UNEXPECTED_TOKEN_PARAMS)

    def _p_list(self) -> _Res[Sequence[FuncParam.Type] | None]:
        _ = self.p._next()
        out: list[FuncParam.Type] = []
        while True:
            if (tk := self.p._peek()) is None:
                return None, self.p._single_err(UNTERMINATED_FN_PARAMS)
            if isinstance(tk, Lexeme.CLOSE_BRACKET):
                break
            param, err = self._p_one()
            if param is None:
                return None, err
            out.extend(param)
        _ = self.p._next()
        return out, []

    def _p_one(self) -> _Res[Sequence[FuncParam.Type] | None]:
        if (tk := self.p._peek()) is None:
            return None, self.p._single_err(UNTERMINATED_FN_PARAMS)
        if isinstance(tk, Lexeme.OPEN_BRACE):
            return self._p_unpacked(name_ref=None)
        if not isinstance(tk, Lexeme.ATOM):
            return None, self.p._single_err(EXPECTED_ATOM)
        typed_ref, errors = self.p._ref.parse_typed_ref()
        if typed_ref is None:
            return None, errors
        if (next_tk := self.p._peek()) and isinstance(next_tk, Lexeme.AT):
            _ = self.p._next()
            return self._p_unpacked(name_ref=typed_ref)
        return (typed_ref,), []

    def _p_unpacked_named(self) -> _Res[Sequence[FuncParam.UNWRAPPED] | None]:
        if (nth_1 := self.p._peek_nth(1)) is None:
            return None, self.p._single_err(UNTERMINATED_FN)
        if not isinstance(nth_1, Lexeme.AT):
            return None, self.p._single_err(EXPECTED_AT)
        if (nth_2 := self.p._peek_nth(2)) is None:
            return None, self.p._single_err(UNTERMINATED_FN)
        if not isinstance(nth_2, Lexeme.OPEN_BRACE):
            return None, self.p._single_err(EXPECTED_BRACE)
        typed_ref, errors = self.p._ref.parse_typed_ref()
        if typed_ref is None:
            return typed_ref, errors
        _ = self.p._next()
        return self._p_unpacked(name_ref=typed_ref)

    def _p_unpacked(
        self,
        *,
        name_ref: TypedReference | None = None,
    ) -> _Res[Sequence[FuncParam.UNWRAPPED] | None]:
        if name_ref and name_ref.annotation:
            return None, self.p._single_err(UNEXPECTED_UNPACKED_TYPE_SIG)
        if (p_start_tk := self.p._next()) is None:
            return None, self.p._single_err(UNTERMINATED_FN_PARAMS)
        if not isinstance(p_start_tk, Lexeme.OPEN_BRACE):
            return None, self.p._single_err(EXPECTED_BRACE)
        params: list[UnwrappedParam] = []
        allow_extra: bool = False
        while True:
            if (tk := self.p._peek()) is None:
                return None, self.p._single_err(UNTERMINATED_FN_PARAMS)
            if isinstance(tk, Lexeme.CLOSE_BRACE):
                break
            if isinstance(tk, Lexeme.ELLIPSIS):
                _ = self.p._next()
                if not allow_extra:
                    allow_extra = True
                    continue
                else:
                    return None, self.p._single_err(UNEXPECTED_DOUBLE_ELLIPSIS)
            if not isinstance(tk, Lexeme.ATOM):
                return None, self.p._single_err(EXPECTED_ATOM)
            typed_ref, errors = self.p._ref.parse_typed_ref()
            if typed_ref is None:
                return None, errors
            if not (
                (next_tk := self.p._peek())
                and isinstance(next_tk, Lexeme.QUESTION_MARK)
            ):
                params.append((typed_ref, None))
                continue
            # handle default value
            _ = self.p._next()  # skip question mark
            default_val, errors = self.p._parse_one()
            if default_val is None:
                return None, errors
            params.append((typed_ref, default_val))
        _ = self.p._next()

        return (
            FuncParam.UNWRAPPED(
                name=name_ref,
                params=params,
                allow_extra=allow_extra,
            ),
        ), []
