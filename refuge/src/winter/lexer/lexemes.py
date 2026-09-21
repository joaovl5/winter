from typing import ClassVar, override

from attrs import define


class Lexeme:
    @define
    class _Base:
        pos_start: int
        pos_end: int

        _SYMBOL: ClassVar[str | None] = None

        def _repr(self) -> str | None:
            return None

        @override
        def __repr__(self) -> str:
            out = self._repr() or self._SYMBOL

            return f"«{self.pos_start} '{out}' {self.pos_end}»"

    @define(repr=False)
    class OPEN_PAREN(_Base):
        _SYMBOL: ClassVar[str | None] = "("

    @define(repr=False)
    class OPEN_BRACKET(_Base):
        _SYMBOL: ClassVar[str | None] = "["

    @define(repr=False)
    class OPEN_BRACE(_Base):
        _SYMBOL: ClassVar[str | None] = "{"

    @define(repr=False)
    class CLOSE_PAREN(_Base):
        _SYMBOL: ClassVar[str | None] = ")"

    @define(repr=False)
    class CLOSE_BRACKET(_Base):
        _SYMBOL: ClassVar[str | None] = "]"

    @define(repr=False)
    class CLOSE_BRACE(_Base):
        _SYMBOL: ClassVar[str | None] = "}"

    @define(repr=False)
    class HASH(_Base):
        _SYMBOL: ClassVar[str | None] = "#"

    @define(repr=False)
    class AT(_Base):
        _SYMBOL: ClassVar[str | None] = "@"

    @define(repr=False)
    class EXCLAMATION(_Base):
        _SYMBOL: ClassVar[str | None] = "!"

    @define(repr=False)
    class QUESTION_MARK(_Base):
        _SYMBOL: ClassVar[str | None] = "?"

    @define(repr=False)
    class BACKTICK(_Base):
        _SYMBOL: ClassVar[str | None] = "`"

    @define(repr=False)
    class COMMA(_Base):
        _SYMBOL: ClassVar[str | None] = ","

    @define(repr=False)
    class DOT(_Base):
        _SYMBOL: ClassVar[str | None] = "."

    @define(repr=False)
    class ELLIPSIS(_Base):
        _SYMBOL: ClassVar[str | None] = "…"

    @define(repr=False)
    class ATOM(_Base):
        value: str

        @override
        def _repr(self) -> str | None:
            return self.value

    @define(repr=False)
    class NUMBER(_Base):
        value: str

        @override
        def _repr(self) -> str | None:
            return self.value

    @define(repr=False)
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
    @define(repr=False)
    class TEXT:
        value: str

        @override
        def __repr__(self) -> str:
            return self.value

    @define(repr=False)
    class EXPR:
        value: list[Lexeme.Type]

        @override
        def __repr__(self) -> str:
            out = " | ".join(x._repr() or x._SYMBOL or "_" for x in self.value)  # pyright: ignore[reportPrivateUsage]
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
}
