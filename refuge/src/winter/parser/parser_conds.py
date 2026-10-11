lazy from typing import TYPE_CHECKING

lazy from attrs import define

lazy from winter.lexer.lexemes import Lexeme

lazy from .ast import AST

if TYPE_CHECKING:
    lazy from .parser import Parser

UNTERMINATED_IF = "Unterminated `if`"
EXPECTED_CASE_FALSE = "Expected `if` to have both 'true' then 'false' case handling"
EXPECTED_RPAREN = "Expected a closing parenthesis after both `if` cases"


@define(slots=True)
class CondParser:
    p: Parser
    type _Res[X] = Parser._Res[X]
    type _ASTRes = Parser._ASTRes

    def parse_cond(self) -> _ASTRes:
        _ = self.p._next()  # in `if`

        pos_start, _ = self.p._get_resolved_pos()

        if self.p._peek() is None:
            return None, self.p._single_err(UNTERMINATED_IF)

        cond_value, errors = self.p._parse_one()
        if cond_value is None:
            return None, errors
        case_true, errors = self.p._parse_one()
        if case_true is None:
            return None, errors
        if (tk := self.p._peek()) and isinstance(tk, Lexeme.CLOSE_PAREN):
            return None, self.p._single_err(EXPECTED_CASE_FALSE)
        case_false, errors = self.p._parse_one()
        if case_false is None:
            return None, errors
        if (tk := self.p._next()) is None:
            return None, self.p._single_err(UNTERMINATED_IF)
        if not isinstance(tk, Lexeme.CLOSE_PAREN):
            return None, self.p._single_err(EXPECTED_RPAREN)
        _, pos_end = self.p._get_resolved_pos(-1)
        return AST.IF(
            pos_start=pos_start,
            pos_end=pos_end,
            condition=cond_value,
            case_true=case_true,
            case_false=case_false,
        ), []
