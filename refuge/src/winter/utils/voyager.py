from abc import ABC, abstractmethod
from collections.abc import Callable, Generator, Iterator, Sequence
from functools import cached_property

from attrs import define, field
from more_itertools import SequenceView, loops, seekable

from winter.utils.error_box import ErrorBox


@define
class Voyager[In, Target, Err: Exception](ABC):
    source: Iterator[In]
    error_t: type[Err]
    pos: int = 0
    pos_stack: list[int] = field(factory=list)
    source_iter: seekable[In] = field(init=False)

    type Result[X] = tuple[X, list[Err]]

    def __attrs_post_init__(self) -> None:
        self.source_iter = seekable(self.source)

    @cached_property
    def _source_elems(self) -> SequenceView[In]:
        return self.source_iter.elements()

    # like 'parse' but will collect from the generator
    def parse_total(self) -> Result[list[Target]]:
        out: list[Target] = []
        errors: list[Err] = []

        for x, x_errs in self.parse():
            if x is not None:
                out.append(x)
            errors.extend(x_errs)
        return out, errors

    # continuosly yields Ts and Es
    # sends a 'None' value only at the end
    def parse(
        self,
    ) -> Generator[Result[Target | None],]:
        while True:
            obj, errors = self._parse_one()
            yield obj, errors
            if obj is None:
                break

    @abstractmethod
    def _parse_one(self) -> Result[Target | None]: ...

    # in case the raw source "pos" is not the original pos, this function may be
    # overriden. we use two integers so that ranges may be used later
    # we expect the first to be smaller or equal than the second
    def _resolve_some_pos(
        self,
        pos: int,
    ) -> tuple[int, int]:
        return pos, pos

    # convenience method for using current pos
    def _get_resolved_pos(self, offset: int = 0) -> tuple[int, int]:
        return self._resolve_some_pos(self.pos + offset)

    def _error(self, msg: str, *, error_t: type[Err]) -> Err:
        pos_a, pos_b = self._get_resolved_pos()
        pos_str = f"({pos_a}→{pos_b})"
        return error_t(f"at {pos_str} : {msg}")

    def _mk_err_box(self) -> ErrorBox[Err]:
        return ErrorBox(
            error_t=self.error_t,
            error_factory=self._error,
        )

    def _single_err(self, msg: str) -> ErrorBox[Err]:
        box = self._mk_err_box()
        box.push(msg)
        return box

    def _peek(self) -> In | None:
        try:
            return self.source_iter.peek()
        except StopIteration:
            return None

    def _next(self) -> In | None:
        try:
            out = next(self.source_iter)
            self.pos += 1
            return out
        except StopIteration:
            return None

    def _peek_n(self, n: int = 1) -> list[In]:
        move_back = 0
        out: list[In] = []
        if (x := self._peek()) is None:
            return out
        out.append(x)
        for _ in loops(n):
            self.source_iter.relative_seek(1)
            move_back -= 1
            if (x := self._peek()) is None:
                break
            out.append(x)
        self.source_iter.relative_seek(move_back)
        return out

    def _peek_nth(self, nth: int = 0) -> In | None:
        lst = self._peek_n(nth)
        try:
            return lst[nth]
        except IndexError:
            return None

    # FIXME: currently this isnt working properly because __eq__ does pos_start
    # we'll need something to deal in type-level sequences without initializing
    def _starts_with(self, x: Sequence[In]) -> bool:
        y = self._peek_n(len(x))
        return tuple(x) == tuple(y)

    def _get_range(self, a: int, b: int) -> list[In]:
        self._pos_checkpoint()
        out: list[In] = []
        idx = a
        diff = b - a
        if diff < 0:
            raise IndexError("Range doesn't make sense!")
        for _ in loops(diff):
            self.source_iter.seek(idx)
            x = self.source_iter.peek()
            idx += 1
            if x is None:
                break
            out.append(x)

        self._pos_rollback()
        return out

    def _pos_checkpoint(self) -> None:
        self.pos_stack.insert(0, self.pos)

    def _pos_rollback(self) -> None:
        self.pos = self.pos_stack.pop(0)
        self.source_iter.seek(self.pos)

    # python's typesystem can't handle this, but
    # T_sub must be a subtype of T, please
    def _bake_t[T_sub](
        self,
        t: Callable[..., T_sub],
        pos_start: int | None = None,
        pos_end: int | None = None,
        *args: object,
        **kwargs: object,
    ) -> T_sub:
        pos_start = pos_start or self.pos
        pos_end = pos_end or self.pos
        kwargs["pos_start"] = pos_start
        kwargs["pos_end"] = pos_end
        return t(*args, **kwargs)
