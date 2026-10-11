lazy from typing import TYPE_CHECKING

lazy from attrs import define

lazy from winter.lexer.lexemes import Lexeme
lazy from winter.parser.ast import AST, CallableHead

if TYPE_CHECKING:
    lazy from .parser import Parser


EXPECTED_TOKEN = "Expected a token"
EXPECTED_PAREN = "Expected opening parenthesis"
UNTERMINATED_BLOCK = "Unterminated block"


@define(slots=True)
class ParenParser:
    p: Parser
    type _Res[X] = Parser._Res[X]
    type _ASTRes = Parser._ASTRes

    def parse_paren(self) -> _ASTRes:
        if (open_tk := self.p._next()) is None:
            return None, self.p._single_err(EXPECTED_PAREN)
        if (head_tk := self.p._peek()) is None:
            return None, self.p._single_err(UNTERMINATED_BLOCK)
        if (out := self._parse_paren_special(head_tk=head_tk)) is not None:
            return out
        c_head: CallableHead.Type
        if (op := self._parse_op(head_tk=head_tk)) is not None:
            c_head = op
            _ = self.p._next()
        else:
            res, errors = self._paren_head_expr()
            if res is None:
                return None, errors
            c_head = res
        c_tail: list[AST.Type] = []
        while True:
            if (tk := self.p._peek()) is None:
                return None, self.p._single_err(UNTERMINATED_BLOCK)
            if isinstance(tk, Lexeme.CLOSE_PAREN):
                break
            ast_val, errors = self.p._parse_one()
            if ast_val is None:
                return None, errors
            c_tail.append(ast_val)

        if (close_tk := self.p._next()) is None:
            return None, self.p._single_err(UNTERMINATED_BLOCK)

        return AST.CALLABLE(
            pos_start=open_tk.pos_start,
            pos_end=close_tk.pos_end,
            head=c_head,
            tail=c_tail,
        ), []

    def _paren_head_expr(self) -> _Res[CallableHead.EXPR | None]:
        if (tk := self.p._peek()) is None:
            return None, self.p._single_err(EXPECTED_TOKEN)
        if not isinstance(tk, Lexeme.ATOM):
            return self.p._parse_one()
        pos_start, _ = self.p._get_resolved_pos()
        dot_path, errors = self.p._ref.parse_dotted_path(allow_eof=False)
        if dot_path is None:
            return None, errors
        head_nodes, pos_end = dot_path
        ast_val: AST.Type
        match len(head_nodes):
            case 0:
                raise RuntimeError("`head_nodes` should never be zero here")
            case 1:
                ast_val = head_nodes[0]
            case _:
                ast_val = AST.SELECTION(
                    pos_start=pos_start,
                    pos_end=pos_end,
                    items=head_nodes,
                )
        return ast_val, []

    def _parse_op(self, *, head_tk: Lexeme.Type) -> CallableHead.OPERATOR | None:
        if isinstance(head_tk, Lexeme.DOT):
            return CallableHead.OPERATOR.SEL
        if not isinstance(head_tk, Lexeme.ATOM):
            return None
        try:
            return CallableHead.OPERATOR(head_tk.value)
        except ValueError:
            return None

    def _parse_paren_special(self, *, head_tk: Lexeme.Type) -> _ASTRes | None:
        if not isinstance(head_tk, Lexeme.ATOM):
            return None
        match head_tk.value:
            case "let":
                return self.p._binds.parse_let()
            case "locals":
                return self.p._binds.parse_locals()

            case "inherit":
                return self.p._binds.parse_lone_inherit()
            case "fn":
                return self.p._func.parse_fn()
            case "if":
                return self.p._cond.parse_cond()
            case "type":
                return self.p._todo()
            case "enum":
                return self.p._todo()
            case _:
                return None
