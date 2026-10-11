lazy from typing import override

lazy from attrs import Factory, define, field
lazy from loguru import logger

lazy from winter.lexer.lexemes import Lexeme
lazy from winter.utils.voyager import Voyager, VoyagerError

lazy from .ast import AST
lazy from .parser_binds import BindsParser
lazy from .parser_collections import CollectionParser
lazy from .parser_conds import CondParser
lazy from .parser_func import FuncParser
lazy from .parser_paren import ParenParser
lazy from .parser_ref import RefParser
lazy from .parser_scalar import ScalarParser


@define(slots=True, frozen=True)
class ParserError(VoyagerError): ...


@define(slots=True)
class Parser(Voyager[Lexeme.Type, AST.Type, ParserError]):
    error_t: type[ParserError] = field(default=ParserError, init=False)
    _binds: BindsParser = Factory(BindsParser, takes_self=True)
    _func: FuncParser = Factory(FuncParser, takes_self=True)
    _ref: RefParser = Factory(RefParser, takes_self=True)
    _scalar: ScalarParser = Factory(ScalarParser, takes_self=True)
    _paren: ParenParser = Factory(ParenParser, takes_self=True)
    _col: CollectionParser = Factory(CollectionParser, takes_self=True)
    _cond: CondParser = Factory(CondParser, takes_self=True)

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
                logger.warning("Had to fallback to (0, 0) - this may be a bug!")
                return 0, 0
        return tk.pos_start, tk.pos_end

    def _todo(self) -> _ASTRes:
        raise NotImplementedError
