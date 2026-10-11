lazy import inspect
lazy from enum import Enum
lazy from pathlib import Path
lazy from typing import Self, override

lazy import rich.repr as r
lazy from attrs import Factory

lazy from winter.utils.types import Symbol, def_strict, def_sym

type OneBind = tuple[TypedReference, AST.Type]
type ManyBinds = list[OneBind]
type ManyInherits = list[InheritNode]


# we don't call these "modules" because of nixpkgs modules
# neither "namespaces" because its already used elsewhere
# for other purposes through-out the codebase
@def_strict
class Context:
    file: Path
    body: list[AST.Type]
    name: str = Factory(lambda s: s.file.name, takes_self=True)


@def_strict
class Project:
    contexts: list[Context]
    dir: Path = Factory(lambda: Path("."))
    name: str = Factory(lambda s: s.dir.name, takes_self=True)


class AST:
    @r.auto(angular=False)
    class _Base(Symbol):
        def __rich_repr__(self) -> r.Result:
            sig = inspect.signature(self.__init__)
            params = sig.parameters
            is_single_param = len(params) - 2 == 1
            for name, param in params.items():
                if name in {"pos_start", "pos_end"}:
                    continue
                if is_single_param:
                    yield getattr(self, param.name)
                else:
                    yield param.name, getattr(self, param.name)

    @def_sym
    class NIL(_Base):
        _SYMBOL = "∅"

    @def_sym
    class BOOL(_Base):
        value: bool

        @override
        def _repr(self) -> str | None:
            return "TRUE" if self.value else "FALSE"

    @def_sym
    class NUMBER(_Base):
        value: str

        @override
        def _repr(self) -> str | None:
            return self.value

    @def_sym
    class IDENTIFIER(_Base):
        value: str

        @override
        def _repr(self) -> str | None:
            return self.value

    @def_sym
    class STRING_LIKE(_Base):
        parts: list[StringLikeAST.Type]
        is_path: bool

        @override
        def _repr(self) -> str | None:
            return (
                ("path" if self.is_path else "str")
                + "' "
                + "' ".join((f"'{part.__repr__()}") for part in self.parts)
            )

    @def_sym
    class LIST(_Base):
        items: list[AST.Type]

        @override
        def _repr(self) -> str | None:
            return "list' " + ("' ".join(f"'{x._repr()}" for x in self.items))

    @def_sym
    class SELECTION(_Base):
        items: list[AST.Type]

        @override
        def _repr(self) -> str | None:
            return "sel' " + ("' ".join(f"'{x._repr()}" for x in self.items))

    @def_sym
    class MAP(_Base):
        type MapBind = tuple[  # one tuple is one bind
            list[AST.Type],  # one or more path components
            AST.Type,  # bound value
        ]
        binds: list[MapBind]
        inherits: ManyInherits

        @override
        def _repr(self) -> str | None:
            binds = "' ".join(
                f"{'.'.join(kx.__repr__() for kx in k)} ⇒ {v}" for k, v in self.binds
            )
            inherits = "' ".join(
                f"{ns}-({(' '.join(lstx.__repr__() for lstx in lst)) if lst else ''})"
                for ns, lst in self.inherits
            )
            return "map' " + binds + inherits

    @def_sym
    class CALLABLE(_Base):
        head: CallableHead.Type
        tail: list[AST.Type]

        @override
        def _repr(self) -> str | None:

            return f"{self.head.__repr__()}' " + "' ".join(
                f"'{x._repr()}" for x in self.tail
            )

    @def_sym
    class INHERITS(_Base):
        inherits: ManyInherits

        def _repr_inherit(self) -> str:
            return "' ".join(
                f"{ns}-({(' '.join(lstx.__repr__() for lstx in lst)) if lst else ''})"
                for ns, lst in self.inherits
            )

        @override
        def _repr(self) -> str | None:
            return "inherit' " + self._repr_inherit()

    @def_sym
    class LOCALS(INHERITS):
        binds: ManyBinds

        def _repr_bindings(self) -> str:
            binds = "' ".join(f"{k.__repr__()} ⇒ {v}" for k, v in self.binds)
            return binds + self._repr_inherit()

        @override
        def _repr(self) -> str | None:
            return "locals' " + self._repr_bindings()

    @def_sym
    class LET(LOCALS):
        body: AST.Type

        @override
        def _repr(self) -> str | None:
            return "let' " + self._repr_bindings() + " body '" + self.body.__repr__()

    @def_sym
    class FUNCTION(_Base):
        name: TypedReference | None
        params: list[FuncParam.Type]
        body: list[AST.Type]

        @override
        def _repr(self) -> str | None:
            return (
                (self.name.__repr__() if self.name else "")
                + "' "
                + " ! ".join(p.__repr__() for p in self.params)
                + " : "
                + " ¡ ".join(x.__repr__() for x in self.body)
            )

    @def_sym
    class DECLARED_TYPE(_Base):
        name: TypedReference | None
        value: DeclaredType.Type

        @override
        def _repr(self) -> str | None:
            return (
                "type "
                + ("`" + self.name.__repr__() + "`" if self.name else "")
                + " : "
                + self.value.__repr__()
            )

    @def_sym
    class IF(_Base):
        condition: AST.Type
        case_true: AST.Type
        case_false: AST.Type

    type Type = (
        NIL
        | BOOL
        | NUMBER
        | STRING_LIKE
        | IDENTIFIER
        | LIST
        | SELECTION
        | MAP
        | CALLABLE
        | INHERITS
        | LOCALS
        | LET
        | FUNCTION
        | DECLARED_TYPE
        | IF
    )


class DeclaredType:
    type INTERSECTION = list[TypeAnnotation]
    type ATTRIBUTES = list[TypedReference]

    @def_sym
    class ENUM:
        type_value: TypeAnnotation
        items: list[tuple[str, AST.Type]]

        @override
        def __repr__(self) -> str:
            return f"enum ({self.type_value.__repr__}) : " + "; ".join(
                f"{k}⇒{v.__repr__()}" for k, v in self.items
            )

    type Type = INTERSECTION | ATTRIBUTES | ENUM


class CallableHead:
    class OPERATOR(Enum):
        ADD = "+"
        SUB = "-"
        MUL = "*"
        DIV = "/"
        GT = ">"
        LT = "<"
        GTE = ">="
        LTE = "<="
        EQ = "="
        NEQ = "not="
        NOT = "not"
        ALL = "all"
        ANY = "any"
        THUS = "thus"
        SEL = "."
        OR = "or"
        HAS = "has"
        CONC = "++"
        UPDT = "//"
        LPIPE = "<|"
        RPIPE = "|>"

    type EXPR = AST.Type
    type Type = OPERATOR | EXPR


@def_sym
class TypedReference:
    name: str
    annotation: TypeAnnotation | None = None

    @override
    def __repr__(self) -> str:
        return f"{self.name}"


type UnwrappedParam = tuple[TypedReference, AST.Type | None]


class FuncParam:
    type PLAIN = TypedReference

    @def_sym
    class UNWRAPPED:
        # name for arg if named
        name: TypedReference | None
        # unpacked params, optionally with defaults (AST.Type)
        params: list[UnwrappedParam]
        # when ellipsis is detected
        allow_extra: bool

        @override
        def __repr__(self) -> str:
            return (
                (self.name.__repr__() + "@" if self.name else "")
                + " "
                + " | ".join(
                    typed_ref.__repr__()
                    + (f"={def_value}" if def_value is not None else "")
                    for typed_ref, def_value in self.params
                )
                + ("…" if self.allow_extra else "")
            )

    type Type = PLAIN | UNWRAPPED


class TypeAnnotation:
    @def_sym
    class ANY: ...

    @def_sym
    class NIL: ...

    @def_sym
    class STRING: ...

    @def_sym
    class PATH: ...

    @def_sym
    class NUMBER: ...

    @def_sym
    class TYPE_NAME:
        value: str

    @def_sym
    class LIST:
        t: TypeAnnotation.Type

    @def_sym
    class MAP:
        key_t: TypeAnnotation.Type
        value_t: TypeAnnotation.Type

    @def_sym
    class FUNCTION[OUT_T: TypeAnnotation.Type]:
        args_t: list[TypeAnnotation.Type]
        out_t: OUT_T

        def apply_once(self) -> Self | OUT_T:
            """Represent function type after one argument was consumed/curried."""
            if len(self.args_t) == 0:
                raise RuntimeError("Function can't have zero args")
            if len(self.args_t) == 1:
                return self.out_t
            new_args_t = self.args_t[1:]
            return self.__class__(
                args_t=new_args_t,
                out_t=self.out_t,
            )

    @def_sym
    class UNION:
        items: list[TypeAnnotation.Type]

    type Type = None


type InheritNode = tuple[
    str,
    # if 'None' is just a single-thing inherit
    # otherwise its a list of the things
    # further explanation at a later date, I'm
    # currently tired to explain now
    list[TypedReference] | None,
]


class StringLikeAST:
    @def_sym
    class TEXT:
        value: str

        @override
        def __repr__(self) -> str:
            return self.value

    @def_sym
    class EXPR:
        value: AST.Type

        @override
        def __repr__(self) -> str:
            return f"«{self.value._repr() or self.value._SYMBOL}»"

    type Type = TEXT | EXPR
