from typing import override

from attrs import Factory, define, field
from loguru import logger

from winter.lexer.lexemes import Lexeme
from winter.utils.voyager import Voyager

from .ast import AST
from .parser_binds import BindsParser
from .parser_collections import CollectionParser
from .parser_func import FuncParser
from .parser_paren import ParenParser
from .parser_ref import RefParser
from .parser_scalar import ScalarParser


class ParserError(RuntimeError): ...


@define
class Parser(Voyager[Lexeme.Type, AST.Type, ParserError]):
    error_t: type[ParserError] = field(default=ParserError, init=False)
    _binds: BindsParser = Factory(BindsParser, takes_self=True)
    _func: FuncParser = Factory(FuncParser, takes_self=True)
    _ref: RefParser = Factory(RefParser, takes_self=True)
    _scalar: ScalarParser = Factory(ScalarParser, takes_self=True)
    _paren: ParenParser = Factory(ParenParser, takes_self=True)
    _col: CollectionParser = Factory(CollectionParser, takes_self=True)

    type _Res[X] = Parser.Result[X]
    type _ASTRes = _Res[AST.Type | None]

    @override
    def _parse_one(self) -> _ASTRes:
        all_errors = self._mk_err_box()

        if (tk := self._peek()) is None:
            return None, all_errors
        match tk:
            case Lexeme.OPEN_PAREN():
                return self._paren.parse_paren()
            case Lexeme.OPEN_BRACKET():
                return self._col.parse_list()
            case Lexeme.OPEN_BRACE():
                return self._col.parse_map()
            case _:
                return self._scalar.parse_scalar()

    @override
    def _resolve_some_pos(self, pos: int) -> tuple[int, int]:
        tk = self._get_source_elem(pos)
        if tk is None:
            if pos > 0:
                return self._resolve_some_pos(pos - 1)
            else:
                logger.warning("Had to fallback to (0, 0)")
                return 0, 0
        return tk.pos_start, tk.pos_end

    def _todo(self) -> _ASTRes:
        raise NotImplementedError
