from typing import TYPE_CHECKING

from attrs import define

from winter.lexer.lexemes import Lexeme
from winter.parser.ast import AST, ManyInherits

if TYPE_CHECKING:
    from .parser import Parser


EXPECTED_BRACKET = "Expected opening bracket"
EXPECTED_BRACE = "Expected opening brace"
UNTERMINATED_LIST = "Unterminated list"
UNTERMINATED_MAP = "Unterminated map"
EXCLAM_SYNTAX_TODO = "The `!` syntax for map keys wasn't yet implemented"


@define
class CollectionParser:
    p: Parser
    type _Res[X] = Parser._Res[X]
    type _ASTRes = Parser._ASTRes

    def parse_list(self) -> _ASTRes:
        if (open_tk := self.p._next()) is None:
            return None, self.p._single_err(EXPECTED_BRACKET)
        items: list[AST.Type] = []
        while True:
            if (tk := self.p._peek()) is None:
                return None, self.p._single_err(UNTERMINATED_LIST)
            if isinstance(tk, Lexeme.CLOSE_BRACKET):
                break
            ast_val, errors = self.p._parse_one()
            if ast_val is None:
                return None, errors
            items.append(ast_val)
        if (close_tk := self.p._next()) is None:
            return None, self.p._single_err(UNTERMINATED_LIST)
        return AST.LIST(
            pos_start=open_tk.pos_start,
            pos_end=close_tk.pos_end,
            items=items,
        ), []

    def parse_map(self) -> _ASTRes:
        if (open_tk := self.p._next()) is None:
            return None, self.p._single_err(EXPECTED_BRACE)
        items: list[AST.MAP.MapBind] = []
        inherits: ManyInherits = []

        while True:
            if (tk := self.p._peek()) is None:
                return None, self.p._single_err(UNTERMINATED_MAP)
            if isinstance(tk, Lexeme.EXCLAMATION):
                return None, self.p._single_err(EXCLAM_SYNTAX_TODO)
            if isinstance(tk, Lexeme.CLOSE_BRACE):
                break
            if (
                isinstance(tk, Lexeme.OPEN_PAREN)
                and (next_tk := self.p._peek_nth(1))
                and isinstance(next_tk, Lexeme.ATOM)
                and next_tk.value == "inherit"
            ):
                _ = self.p._next()
                _ = self.p._next()
                new_inherits, errors = self.p._binds.parse_inherit()
                if new_inherits is None:
                    return None, errors
                inherits.extend(new_inherits)
                continue

            dot_path, errors = self.p._ref.parse_dotted_path(allow_eof=False)
            if dot_path is None:
                return None, errors
            item_key, _ = dot_path
            item_val, errors = self.p._parse_one()
            if item_val is None:
                return None, errors
            items.append((item_key, item_val))

        if (close_tk := self.p._next()) is None:
            return None, self.p._single_err(UNTERMINATED_MAP)

        return AST.MAP(
            pos_start=open_tk.pos_start,
            pos_end=close_tk.pos_end,
            binds=items,
            inherits=inherits,
        ), []
