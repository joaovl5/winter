lazy from typing import ClassVar, dataclass_transform, override

lazy from attrs import define, field


# wrapper so pyright behaves
@dataclass_transform(
    field_specifiers=(field,),
    eq_default=True,
    order_default=False,
    kw_only_default=False,
    frozen_default=True,
)
def def_sym[T](cls: type[T]) -> type[T]:
    return define(
        repr=False,
        frozen=True,
        eq=True,
    )(cls)


# another wrapper so pyright behaves
@dataclass_transform(
    field_specifiers=(field,),
    eq_default=True,
    order_default=False,
    kw_only_default=True,
    frozen_default=True,
)
def def_strict[T](cls: type[T]) -> type[T]:
    return define(
        repr=False,
        kw_only=True,
        frozen=True,
        slots=True,
        eq=True,
    )(cls)


@def_sym
class Symbol:
    pos_start: int
    pos_end: int

    _SYMBOL: ClassVar[str | None] = None

    def _repr(self) -> str | None:
        return None

    @override
    def __repr__(self) -> str:
        out = self._repr() or self._SYMBOL

        return f"«{self.pos_start} '{out}' {self.pos_end}»"
