from collections.abc import Generator
from typing import override

from attrs import define, field
from more_itertools import loops

from winter.utils.voyager import Voyager

from .lexemes import SINGLE_CHAR_LEXEMES, Lexeme, StringLikeLex


class LexerError(RuntimeError): ...


BACKSLASH_VALUES = {
    "n": "\n",
    "r": "\r",
    "t": "\t",
    '"': '"',
    "\\": "\\",
}


@define
class Lexer(Voyager[str, Lexeme.Type, LexerError]):
    error_t: type[LexerError] = field(default=LexerError, init=False)
    type _Res[X] = Lexer.Result[X]

    @override
    def _parse_one(self) -> _Res[Lexeme.Type | None]:
        self._skip_ignored()
        all_errors: list[LexerError] = []
        pos_start = self.pos

        if (ch := self._peek()) is None:
            return None, all_errors
        if self._starts_with("..."):
            for _ in loops(3):
                _ = self._next()
            return Lexeme.ELLIPSIS(
                pos_start=pos_start,
                pos_end=self.pos,
            ), all_errors
        if (tk := self._parse_single_char(ch)) is not None:
            _ = self._next()
            return tk, all_errors
        for tk, errors in self._parse_one_options():
            all_errors.extend(errors)
            if tk:
                return tk, all_errors
        return None, all_errors

    def _parse_one_options(self) -> Generator[_Res[Lexeme.Type | None]]:
        funcs = (
            self._parse_string_like,
            self._parse_number,
            self._parse_atom,
        )
        for func in funcs:
            yield func()

    def _parse_atom(self) -> _Res[Lexeme.ATOM | None]:
        pos_start = self.pos
        while (ch := self._peek()) is not None:
            if not self._is_atom_char(ch):
                break
            _ = self._next()
        pos_end = self.pos
        if pos_start == pos_end:
            return None, [self._error("Expected atom", error_t=LexerError)]
        atom_range = "".join(self._get_range(pos_start, pos_end))
        return self._bake_t(
            Lexeme.ATOM,
            pos_start=pos_start,
            pos_end=pos_end,
            value=atom_range,
        ), []

    def _parse_number(self) -> _Res[Lexeme.NUMBER | None]:
        self._pos_checkpoint()
        pos_start = self.pos
        out: Lexeme.NUMBER | None = None
        invalid: bool = False
        all_errors = self._mk_err_box()
        rollbacks_needed = 1

        if (ch := self._peek()) is None:
            return out, all_errors
        if ch == "-":
            _ = self._next()
        digit_start = self.pos
        self._traverse_digits()
        if digit_start == self.pos:
            for _ in loops(rollbacks_needed):
                self._pos_rollback()
            return out, all_errors
        if (ch := self._peek()) == ".":
            self._pos_checkpoint()
            rollbacks_needed += 1
            _ = self._next()
            decimal_start = self.pos
            self._traverse_digits()
            if decimal_start == self.pos:
                invalid = True
                self._pos_rollback()
                rollbacks_needed -= 1
        if (ch := self._peek()) is not None and self._is_atom_char(ch):
            all_errors.push("Invalid number literal")
            for _ in loops(rollbacks_needed):
                self._pos_rollback()
        if not invalid:
            num_range = "".join(self._get_range(pos_start, self.pos))
            out = self._bake_t(
                Lexeme.NUMBER,
                pos_start=pos_start,
                pos_end=self.pos,
                value=num_range,
            )

        return out, all_errors

    def _is_atom_char(self, ch: str) -> bool:
        return (
            ch.isascii()
            and not ch.isspace()
            and ch not in {";", '"'}
            and self._parse_single_char(ch) is None
        )

    def _traverse_digits(self) -> None:
        while (ch := self._peek()) is not None and ch.isdigit():
            _ = self._next()

    def _parse_single_char(self, ch: str) -> Lexeme.Type | None:
        out_t: type[Lexeme.Type] | None = SINGLE_CHAR_LEXEMES.get(ch)
        if out_t is not None:
            return self._bake_t(out_t)
        return None

    def _parse_string_like(self) -> _Res[Lexeme.STRING_LIKE | None]:
        self._pos_checkpoint()
        pos_start = self.pos
        out: Lexeme.STRING_LIKE | None = None
        all_errors = self._mk_err_box()

        has_opened = False
        is_interp = False
        is_path = False
        parts: list[StringLikeLex.Type] = []
        current_value: list[str] = []

        while True:
            if (ch := self._next()) is None:
                if has_opened:
                    all_errors.push("Unterminated string")
                break
            if not has_opened:
                match ch:
                    case '"':
                        has_opened = True
                    case "$" if self._peek() == '"':
                        is_interp = True
                    case "p" if not is_path:
                        is_path = True
                    case _:
                        break
                continue
            if ch == '"':
                parts.append(StringLikeLex.TEXT("".join(current_value)))
                break
            elif ch == "$":
                current_value.append("\\$")
            elif ch == "\\":
                if (ch_next := self._next()) is None:
                    all_errors.push("Unterminated backslash")
                    break
                if (bs_value := BACKSLASH_VALUES.get(ch_next)) is None:
                    all_errors.push(f"Unsupported escape sequence of '\\{ch_next}'")
                    break
                current_value.append(bs_value)
            elif ch == "{" and is_interp:  # }
                if (ch_next := self._peek()) is None:
                    all_errors.push("Unterminated interpolation")
                    break
                if ch_next == "{":  # }
                    current_value.append("{")  # }
                    _ = self._next()
                    continue

                parts.append(StringLikeLex.TEXT("".join(current_value)))
                current_value.clear()
                interp, errors = self._parse_string_like_interp()
                all_errors.extend(errors)
                if interp:
                    parts.append(StringLikeLex.EXPR(interp))
                else:
                    break
            elif ch == "}" and is_interp:
                if (ch_next := self._next()) is None:
                    all_errors.push("Unterminated string")
                    break
                if ch_next == "}":
                    current_value.append("}")
                    continue
                all_errors.push("Unexpected '}' in interpolated string")
                break
            else:
                current_value.append(ch)

        pos_end = self.pos
        if len(parts) > 0:
            out = self._bake_t(
                Lexeme.STRING_LIKE,
                pos_start=pos_start,
                pos_end=pos_end,
                parts=parts,
                is_path=is_path,
            )

        if out is None:
            self._pos_rollback()
        return out, all_errors

    def _parse_string_like_interp(self) -> _Res[list[Lexeme.Type] | None]:
        out: list[Lexeme.Type] = []
        brace_depth: int = 1
        all_errors = self._mk_err_box()
        while True:
            tk, errors = self._parse_one()
            all_errors.extend(errors)
            if tk is None:
                all_errors.push("Unterminated string interpolation")
                break
            if isinstance(tk, Lexeme.OPEN_BRACE):
                brace_depth += 1
            elif isinstance(tk, Lexeme.CLOSE_BRACE):
                brace_depth -= 1

            if brace_depth == 0:
                break
            out.append(tk)

        return (None if len(out) == 0 else out), all_errors

    def _skip_ignored(self) -> None:
        while True:
            match self._peek():
                case str(x) if x.isspace():
                    _ = self._next()
                case str(";"):
                    while (x := self._next()) not in (None, "\n"):
                        ...
                case _:
                    break
