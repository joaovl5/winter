from typing import TYPE_CHECKING

from attrs import define

from winter.lexer.lexemes import Lexeme

from .ast import AST, TypedReference

if TYPE_CHECKING:
    from .parser import Parser

EXPECTED_REFERENCE = "Expected a reference"
EXPECTED_REFERENCE_ATOM = "Expected an identifier atom within a reference"
UNTERMINATED_DOTTED_PATH = "Unterminated dotted path"
UNEXPECTED_DOTTED_PATH_TOKEN = (
    "Dotted path only supports identifier or string attributes"
)


@define
class RefParser:
    p: Parser
    type _Res[X] = Parser._Res[X]
    type _ASTRes = Parser._ASTRes

    def parse_typed_ref(self) -> _Res[TypedReference | None]:
        all_errors = self.p._mk_err_box()
        if (tk := self.p._next()) is None:
            all_errors.push(EXPECTED_REFERENCE)
            return None, all_errors
        if not isinstance(tk, Lexeme.ATOM):
            all_errors.push(EXPECTED_REFERENCE_ATOM)
            return None, all_errors

        typed_ref = TypedReference(
            name=tk.value,
            # do types later!!!
            annotation=None,
        )
        return typed_ref, all_errors

    def parse_dotted_path(
        self,
        *,
        allow_eof: bool = False,
    ) -> _Res[tuple[list[AST.Type], int] | None]:
        out: list[AST.Type] = []
        pos_end: int
        while True:
            if (tk := self.p._next()) is None:
                return None, self.p._single_err(UNTERMINATED_DOTTED_PATH)
            pos_end = tk.pos_end

            ast_val: AST.Type
            if isinstance(tk, Lexeme.ATOM):
                ast_val = AST.IDENTIFIER(
                    pos_start=tk.pos_start,
                    pos_end=tk.pos_end,
                    value=tk.value,
                )
            elif isinstance(tk, Lexeme.STRING_LIKE):
                str_val, errors = self.p._scalar.parse_str(tk)
                if str_val is None:
                    return None, errors
                ast_val = str_val
            else:
                return None, self.p._single_err(UNEXPECTED_DOTTED_PATH_TOKEN)
            out.append(ast_val)

            if (next_tk := self.p._peek()) is None:
                if allow_eof:
                    break
                return None, self.p._single_err(UNTERMINATED_DOTTED_PATH)
            if not isinstance(next_tk, Lexeme.DOT):
                break
            _ = self.p._next()
        return (out, pos_end), []
