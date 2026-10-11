lazy from typing import TYPE_CHECKING

lazy from attrs import define

lazy from winter.lexer.lexemes import Lexeme, StringLikeLex

lazy from .ast import AST, StringLikeAST

if TYPE_CHECKING:
    lazy from .parser import Parser

UNEXPECTED_TOKEN = "Unexpected token"


@define(slots=True)
class ScalarParser:
    p: Parser
    type _Res[X] = Parser._Res[X]
    type _ASTRes = Parser._ASTRes

    def parse_scalar(self) -> _ASTRes:
        if (tk := self.p._peek()) is None:
            return None, []
        pos_start = tk.pos_start
        pos_end = tk.pos_end
        match tk:
            case Lexeme.ATOM():
                return self._parse_atom(tk)
            case Lexeme.NUMBER(value=value):
                _ = self.p._next()
                return AST.NUMBER(
                    pos_start=pos_start,
                    pos_end=pos_end,
                    value=value,
                ), []
            case Lexeme.STRING_LIKE():
                out = self.parse_str(tk)
                _ = self.p._next()
                return out
            case _:
                return None, self.p._single_err(UNEXPECTED_TOKEN)

    def _parse_atom(self, tk: Lexeme.ATOM) -> _ASTRes:
        pos_end = tk.pos_end
        call_next = True
        out: AST.Type | None = None
        out_t: type[AST.Type] | None = None
        out_t_kwargs: dict[str, object] = {
            "pos_start": tk.pos_start,
        }
        match tk.value:
            case "nil":
                out_t = AST.NIL
            case "false":
                out_t = AST.BOOL
                out_t_kwargs["value"] = False
            case "true":
                out_t = AST.BOOL
                out_t_kwargs["value"] = True
            case _:
                out_t = None
        if out_t is None:
            call_next = False
            dot_path_res, errors = self.p._ref.parse_dotted_path(allow_eof=True)
            if dot_path_res is None:
                return None, errors
            dot_nodes, pos_end = dot_path_res
            match dot_nodes:
                case []:
                    raise RuntimeError("Atom was empty!")
                case [one_node]:
                    out = one_node
                case _:
                    out_t = AST.SELECTION
                    out_t_kwargs["items"] = dot_nodes
        if call_next:
            _ = self.p._next()
        if out is not None:
            return out, []
        if out_t is None:
            raise RuntimeError("`out_t` should never be empty here!")
        out_t_kwargs["pos_end"] = pos_end
        return out_t(
            **out_t_kwargs,  # pyright: ignore[reportArgumentType]
        ), []

    def parse_str(
        self,
        tk: Lexeme.STRING_LIKE,
    ) -> _Res[AST.STRING_LIKE | None]:
        parts: list[StringLikeAST.Type] = []
        for tk_part in tk.parts:
            part, errors = self._str_tk_to_ast(tk_part)
            if part is None:
                return None, errors
            parts.append(part)
        return AST.STRING_LIKE(
            pos_start=tk.pos_start,
            pos_end=tk.pos_end,
            parts=parts,
            is_path=tk.is_path,
        ), []

    def _str_tk_to_ast(
        self, tk_part: StringLikeLex.Type
    ) -> _Res[StringLikeAST.Type | None]:
        match tk_part:
            case StringLikeLex.TEXT():
                return StringLikeAST.TEXT(value=tk_part.value), []
            case StringLikeLex.EXPR():
                # we do this `__class__` trick to avoid circular imports!
                expr_parser = self.p.__class__(source=iter(tk_part.value))
                part_ast, errors = expr_parser._parse_one()
                if part_ast is None:
                    return None, errors
                return StringLikeAST.EXPR(value=part_ast), []
