from typing import override

from attrs import define, field

from winter.lexer.lexemes import Lexeme
from winter.parser.ast import AST
from winter.parser.parser_binds import BindsParser
from winter.utils.voyager import Voyager


class ParserError(RuntimeError): ...


@define
class Parser(Voyager[Lexeme.Type, AST.Type, ParserError]):
    error_t: type[ParserError] = field(default=ParserError, init=False)
    _binds: BindsParser = field(init=False)

    @override
    def __attrs_post_init__(self) -> None:
        self._binds = BindsParser(self)

    type _Res[X] = Parser.Result[X]
    type _ASTRes = _Res[AST.Type | None]

    @override
    def _parse_one(self) -> _ASTRes:
        all_errors = self._mk_err_box()

        if (tk := self._peek()) is None:
            return None, all_errors
        match tk:
            case Lexeme.OPEN_PAREN:
                return self._parse_paren()
            case Lexeme.OPEN_BRACKET:
                return self._parse_list()
            case Lexeme.OPEN_BRACE:
                return self._parse_map()
            case _:
                return self._parse_scalar()

    def _parse_paren(self) -> _ASTRes:
        all_errors = self._mk_err_box()

        if (open_tk := self._next()) is None:
            all_errors.push("Missing opening parenthesis")
            return None, all_errors
        if (head_tk := self._peek()) is None:
            all_errors.push("Unterminated block")
            return None, all_errors
        if out := self._parse_paren_special(head_tk=head_tk):
            return out
        return self._todo()

    def _parse_paren_special(self, head_tk: Lexeme.Type) -> _ASTRes:
        if not isinstance(head_tk, Lexeme.ATOM):
            return None, []
        match head_tk.value:
            case "let":
                return self._binds.parse_let()
            case "locals":
                return self._binds.parse_locals()

            case "inherit":
                return self._binds.parse_lone_inherit()
            case "fn":
                return self._parse_fn()
            case "type":
                return self._todo()
            case "enum":
                return self._todo()
            case _:
                return None, []

    @override
    def _resolve_some_pos(self, pos: int) -> tuple[int, int]:
        tk = self._source_elems[pos]
        return tk.pos_start, tk.pos_end

    def _todo(self) -> _ASTRes:
        raise NotImplementedError
