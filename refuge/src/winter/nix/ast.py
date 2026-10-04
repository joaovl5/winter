import rich.repr as r

from winter.utils.types import def_strict

# this file is just for nix's ast, no winter shenanigans


class AST:
    @r.auto(angular=False)
    @def_strict
    class _Base: ...

    @def_strict
    class NULL(_Base): ...

    @def_strict
    class BOOL(_Base):
        value: bool

    @def_strict
    class NUMBER(_Base):
        value: str

    @def_strict
    class IDENTIFIER(_Base):
        value: str

    @def_strict
    class STRING_LIKE(_Base):
        value: list[StringLikePart.Type]  # todo
        is_path: bool

    @def_strict
    class LIST(_Base):
        entries: list[AST.Type]

    @def_strict
    class ATTRSET(_Base):
        entries: list[Binding.Type]

    @def_strict
    class BINARY_OP(_Base):
        lhs: AST.Type
        rhs: AST.Type
        op: str

    @def_strict
    class UNARY_OP(_Base):
        value: AST.Type
        op: str

    @def_strict
    class LAMBDA(_Base):
        arg: LambdaArg.Type
        body: AST.Type

    @def_strict
    class LET(_Base):
        entries: list[Binding.Type]
        body: AST.Type

    @def_strict
    class APPLY(_Base):
        function: AST.Type
        args: list[AST.Type]

    @def_strict
    class SELECTION(_Base):
        base: AST.Type
        path: list[AST.Type]

    @def_strict
    class IF(_Base):
        condition: AST.Type
        case_true: AST.Type
        case_false: AST.Type

    type Type = (
        NULL
        | BOOL
        | NUMBER
        | IDENTIFIER
        | STRING_LIKE
        | LIST
        | ATTRSET
        | BINARY_OP
        | UNARY_OP
        | LAMBDA
        | LET
        | APPLY
        | SELECTION
        | IF
    )


class LambdaArg:
    @def_strict
    class PLAIN:
        value: str

    @def_strict
    class UNWRAPPED:
        name: str | None
        params: list[tuple[str, AST.Type | None]]
        allow_extra: bool

    type Type = PLAIN | UNWRAPPED


class Binding:
    type ASSIGNMENT = tuple[list[AST.Type], AST.Type]
    type INHERIT = tuple[str, list[str] | None]

    type Type = ASSIGNMENT | INHERIT


class StringLikePart:
    @def_strict
    class TEXT:
        value: str

    @def_strict
    class EXPR:
        value: AST.Type

    type Type = TEXT | EXPR
