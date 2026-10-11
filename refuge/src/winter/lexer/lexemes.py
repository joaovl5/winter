lazy from typing import override

from winter.utils.types import Symbol, def_sym


class Lexeme:
    _Base: type[Symbol] = Symbol

    @def_sym
    class OPEN_PAREN(_Base):
        _SYMBOL = "("

    @def_sym
    class OPEN_BRACKET(_Base):
        _SYMBOL = "["

    @def_sym
    class OPEN_BRACE(_Base):
        _SYMBOL = "{"

    @def_sym
    class CLOSE_PAREN(_Base):
        _SYMBOL = ")"

    @def_sym
    class CLOSE_BRACKET(_Base):
        _SYMBOL = "]"

    @def_sym
    class CLOSE_BRACE(_Base):
        _SYMBOL = "}"

    @def_sym
    class HASH(_Base):
        _SYMBOL = "#"

    @def_sym
    class AT(_Base):
        _SYMBOL = "@"

    @def_sym
    class EXCLAMATION(_Base):
        _SYMBOL = "!"

    @def_sym
    class QUESTION_MARK(_Base):
        _SYMBOL = "?"

    @def_sym
    class BACKTICK(_Base):
        _SYMBOL = "`"

    @def_sym
    class COMMA(_Base):
        _SYMBOL = ","

    @def_sym
    class DOT(_Base):
        _SYMBOL = "."

    @def_sym
    class ELLIPSIS(_Base):
        _SYMBOL = "…"

    @def_sym
    class ATOM(_Base):
        value: str

        @override
        def _repr(self) -> str | None:
            return self.value

    @def_sym
    class NUMBER(_Base):
        value: str

        @override
        def _repr(self) -> str | None:
            return self.value

    @def_sym
    class STRING_LIKE(_Base):
        parts: list[StringLikeLex.Type]
        is_path: bool

        @override
        def _repr(self) -> str | None:
            return (
                ("path" if self.is_path else "str")
                + "' "
                + "' ".join((f"'{part.__repr__()}") for part in self.parts)
            )

    type Type = (
        OPEN_PAREN
        | OPEN_BRACKET
        | OPEN_BRACE
        | CLOSE_PAREN
        | CLOSE_BRACKET
        | CLOSE_BRACE
        | HASH
        | AT
        | EXCLAMATION
        | QUESTION_MARK
        | BACKTICK
        | COMMA
        | DOT
        | ELLIPSIS
        | ATOM
        | NUMBER
        | STRING_LIKE
    )


class StringLikeLex:
    @def_sym
    class TEXT:
        value: str

        @override
        def __repr__(self) -> str:
            return self.value

    @def_sym
    class EXPR:
        value: list[Lexeme.Type]

        @override
        def __repr__(self) -> str:
            out = " | ".join(x._repr() or x._SYMBOL or "_" for x in self.value)
            return f"«{out}»"

    type Type = TEXT | EXPR


SINGLE_CHAR_LEXEMES: dict[str, type[Lexeme.Type]] = {
    "(": Lexeme.OPEN_PAREN,
    "[": Lexeme.OPEN_BRACKET,
    "{": Lexeme.OPEN_BRACE,
    ")": Lexeme.CLOSE_PAREN,
    "]": Lexeme.CLOSE_BRACKET,
    "}": Lexeme.CLOSE_BRACE,
    "#": Lexeme.HASH,
    "!": Lexeme.EXCLAMATION,
    "?": Lexeme.QUESTION_MARK,
    ",": Lexeme.COMMA,
    ".": Lexeme.DOT,
    "@": Lexeme.AT,
}
