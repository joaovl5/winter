lazy from typing import TYPE_CHECKING

lazy from attrs import define

lazy from winter.lexer.lexemes import Lexeme

lazy from .ast import AST, TypedReference

if TYPE_CHECKING:
    lazy from .ast import ManyBinds, ManyInherits, OneBind
    lazy from .parser import Parser, ParserError

EXPECTED_REFERENCE = "Expected a reference"
EXPECTED_REFERENCE_ATOM = "Expected an identifier atom within a reference"
EXPECTED_BIND_ATOM = "Expected an identifier atom within a bind declaration"
EXPECTED_BIND_AST = "Expected value for bind declaration"
EXPECTED_LET_BRACKET = "Expected bracket after `let` call"
EXPECTED_LET_PAREN = "Expected parenthesis after `let` body"
EXPECTED_LET_BODY = "Expected a body in `let` block"
UNTERMINATED_LET = "Unterminated `let` block"
UNTERMINATED_LOCALS = "Unterminated `locals` block"
UNTERMINATED_BINDS = "Unterminated binds block"
UNTERMINATED_INHERIT = "Unterminated `inherit` block"
UNTERMINATED_INHERIT_ITEMS = "Unterminated `inherit` bind items"
BAD_TYPE_INHERIT_BIND = "Inherit item must only be an identifier"


@define(frozen=True, kw_only=True, slots=True)
class _BindsLikeRes:
    pos_start: int
    binds: ManyBinds
    inherits: ManyInherits
    errors: list[ParserError]


@define(slots=True)
class BindsParser:
    p: Parser
    type _Res[X] = Parser._Res[X]
    type _ASTRes = Parser._ASTRes

    # partially handles a binds call like `let` or `locals`
    # used because lots of code may be shared between them
    def _parse_binds_like(
        self,
        *,
        unterminated_error: str,
        needs_bracket: bool,
        needs_bracket_error: str | None,
        binds_stop_tk: type[Lexeme.Type],
    ) -> _BindsLikeRes | list[ParserError]:
        _ = self.p._next()
        all_errors = self.p._mk_err_box()
        pos_start, _ = self.p._get_resolved_pos()

        if needs_bracket:
            if (binds_start_tk := self.p._next()) is None:
                all_errors.push(unterminated_error)
                return all_errors
            if not isinstance(binds_start_tk, Lexeme.OPEN_BRACKET):
                if not needs_bracket_error:
                    raise RuntimeError("Needed `needs_bracket_error`")
                all_errors.push(needs_bracket_error)
                return all_errors
        binds_res, errors = self._parse_binds_list(stop_tk=binds_stop_tk)
        all_errors.extend(errors)
        if binds_res is None:
            return all_errors
        binds, inherits = binds_res
        return _BindsLikeRes(
            pos_start=pos_start,
            binds=binds,
            inherits=inherits,
            errors=all_errors,
        )

    def parse_let(self) -> _ASTRes:
        res = self._parse_binds_like(
            unterminated_error=UNTERMINATED_LET,
            needs_bracket=True,
            needs_bracket_error=EXPECTED_LET_BRACKET,
            binds_stop_tk=Lexeme.CLOSE_BRACKET,
        )
        if not isinstance(res, _BindsLikeRes):
            return None, res
        all_errors = self.p._mk_err_box()
        body_ast, errors = self.p._parse_one()
        all_errors.extend(errors)
        if body_ast is None:
            all_errors.push(EXPECTED_LET_BODY)
            return None, all_errors
        if (let_end_tk := self.p._next()) is None:
            all_errors.push(UNTERMINATED_LET)
            return None, all_errors
        if not isinstance(let_end_tk, Lexeme.CLOSE_PAREN):
            all_errors.push(EXPECTED_LET_PAREN)
            return None, all_errors
        _, pos_end = self.p._get_resolved_pos(-1)

        return AST.LET(
            pos_start=res.pos_start,
            pos_end=pos_end,
            body=body_ast,
            binds=res.binds,
            inherits=res.inherits,
        ), all_errors

    def parse_locals(self) -> _ASTRes:
        res = self._parse_binds_like(
            unterminated_error=UNTERMINATED_LOCALS,
            needs_bracket=False,
            needs_bracket_error=None,
            binds_stop_tk=Lexeme.CLOSE_PAREN,
        )
        if not isinstance(res, _BindsLikeRes):
            return None, res
        _, pos_end = self.p._get_resolved_pos(-1)
        return AST.LOCALS(
            pos_start=res.pos_start,
            pos_end=pos_end,
            binds=res.binds,
            inherits=res.inherits,
        ), res.errors

    def _parse_binds_list(
        self, *, stop_tk: type[Lexeme.Type]
    ) -> _Res[tuple[ManyBinds, ManyInherits] | None]:
        binds: ManyBinds = []
        inherits: ManyInherits = []
        all_errors = self.p._mk_err_box()

        while True:
            if (tk_0 := self.p._peek()) is None:
                all_errors.push("Unterminated binds list")
                return None, all_errors
            if isinstance(tk_0, stop_tk):
                break
            if (
                (isinstance(tk_0, Lexeme.OPEN_PAREN))
                and (tk_1 := self.p._peek_nth(1))
                and (isinstance(tk_1, Lexeme.ATOM))
                and tk_1.value == "inherit"
            ):
                _ = self.p._next()
                _ = self.p._next()
                new_inherits, new_errors = self.parse_inherit()
                all_errors.extend(new_errors)
                if new_inherits:
                    inherits.extend(new_inherits)
                continue
            bind, new_errors = self._parse_one_bind()
            all_errors.extend(new_errors)
            if bind is None:
                all_errors.push("Expected some bind")
                return None, all_errors
            else:
                binds.append(bind)
        _ = self.p._next()

        return (binds, inherits), all_errors

    def parse_lone_inherit(self) -> _ASTRes:
        _ = self.p._next()
        all_errors = self.p._mk_err_box()
        pos_start, _ = self.p._get_resolved_pos()
        inherits, errors = self.parse_inherit()
        all_errors.extend(errors)
        if inherits is None:
            return None, all_errors
        _, pos_end = self.p._resolve_some_pos(self.p.pos - 1)
        return AST.INHERITS(
            pos_start=pos_start,
            pos_end=pos_end,
            inherits=inherits,
        ), all_errors

    def parse_inherit(self) -> _Res[ManyInherits | None]:
        out: ManyInherits = []
        all_errors = self.p._mk_err_box()

        while True:
            if (bind_tk := self.p._next()) is None:
                all_errors.push(UNTERMINATED_INHERIT)
                return None, all_errors
            if isinstance(bind_tk, Lexeme.CLOSE_PAREN):
                break
            if not isinstance(bind_tk, Lexeme.ATOM):
                all_errors.push(BAD_TYPE_INHERIT_BIND)
                return None, all_errors
            if (next_tk := self.p._peek()) is None:
                all_errors.push(UNTERMINATED_INHERIT)
            if isinstance(next_tk, Lexeme.OPEN_PAREN):
                _ = self.p._next()
                items, errors = self._parse_inherit_items()
                all_errors.extend(errors)
                if items is not None:
                    out.append((bind_tk.value, items))
                    continue
                return None, all_errors
            out.append((bind_tk.value, None))

        return out, all_errors

    def _parse_inherit_items(self) -> _Res[list[TypedReference] | None]:
        items: list[TypedReference] = []
        all_errors = self.p._mk_err_box()

        while True:
            if (item_tk := self.p._peek()) is None:
                all_errors.push(UNTERMINATED_INHERIT_ITEMS)
                return None, all_errors
            if isinstance(item_tk, Lexeme.CLOSE_PAREN):
                break
            typed_ref, errors = self.p._ref.parse_typed_ref()
            all_errors.extend(errors)
            if typed_ref is None:
                return None, all_errors
            items.append(typed_ref)
        _ = self.p._next()
        return items, all_errors

    def _parse_one_bind(self) -> _Res[OneBind | None]:
        if (bind_tk := self.p._next()) is None:
            return None, []
        all_errors = self.p._mk_err_box()
        if not isinstance(bind_tk, Lexeme.ATOM):
            all_errors.push(EXPECTED_BIND_ATOM)
            return None, all_errors
        bound_ast, errors = self.p._parse_one()
        all_errors.extend(errors)
        if bound_ast is None:
            all_errors.push(EXPECTED_BIND_AST)
            return None, all_errors
        return (
            TypedReference(
                name=bind_tk.value,
                # do types later !!!
                annotation=None,
            ),
            bound_ast,
        ), all_errors
