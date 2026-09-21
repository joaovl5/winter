from abc import ABC, abstractmethod
from collections.abc import Generator, Iterator, Sequence
from typing import Protocol

from attrs import define, field
from more_itertools import seekable

from winter.utils.error_box import ErrorBox


class Span(Protocol):
    pos_start: int
    pos_end: int

    def __init__(
        self,
        pos_start: int,
        pos_end: int,
        *args: object,
        **kwargs: object,
    ) -> None: ...


@define
class Voyager[T: Span, E: Exception](ABC):
    source: Iterator[T]
    error_t: type[E]
    pos: int = 0
    pos_stack: list[int] = field(factory=list)
    source_iter: seekable[T] = field(init=False)

    def __attrs_post_init__(self) -> None:
        self.source_iter = seekable(self.source)

    @abstractmethod
    def parse(
        self,
    ) -> Generator[
        # continuosly yields Ts and Es
        tuple[list[T], list[E]],
        # must keep sending "True" otherwise stops
        bool,
    ]: ...

    # in case the raw source "pos" is not the original pos, this function may be
    # overriden
    def _resolve_pos(
        self,
        pos: int | None = None,
    ) -> int:
        pos = pos or self.pos
        return pos

    def _error(self, msg: str, *, error_t: type[E]) -> E:
        return error_t(f"at {self._resolve_pos()} : {msg}")

    def _mk_err_box(self) -> ErrorBox[E]:
        return ErrorBox(
            error_t=self.error_t,
            error_factory=self._error,
        )

    def _peek(self) -> T | None:
        try:
            return self.source_iter.peek()
        except StopIteration:
            return None

    def _next(self) -> T | None:
        try:
            out = next(self.source_iter)
            self.pos += 1
            return out
        except StopIteration:
            return None

    def _peek_n(self, n: int = 1) -> list[T]:
        move_back = 0
        out: list[T] = []
        for _ in range(n):
            self.source_iter.relative_seek(1)
            move_back -= 1
            if (x := self._peek()) is None:
                break
            out.append(x)
        self.source_iter.relative_seek(move_back)
        return out

    def _starts_with(self, x: Sequence[T]) -> bool:
        y = self._peek_n(len(x))
        return tuple(x) == tuple(y)

    def _pos_checkpoint(self) -> None:
        self.pos_stack.insert(0, self.pos)

    def _pos_rollback(self) -> None:
        self.pos = self.pos_stack.pop(0)
        self.source_iter.seek(self.pos)

    def _bake_t(
        self,
        voyage_t: type[T],
        pos_start: int | None = None,
        pos_end: int | None = None,
        **kwargs: object,
    ) -> T:
        pos_start = pos_start or self.pos
        pos_end = pos_end or self.pos
        return voyage_t(
            pos_start=pos_start,
            pos_end=pos_end,
            **kwargs,
        )
