from typing import Protocol, Self

from attrs import define


class ErrorFactoryFn[T](Protocol):
    @staticmethod
    def __call__(msg: str, *, error_t: type[T]) -> T: ...


@define
class ErrorBox[T](list[T]):
    """Box made for a certain type of error."""

    error_t: type[T]
    error_factory: ErrorFactoryFn[T]

    def push(self, msg: str, *, error_t: type[T] | None = None) -> None:
        """Push a new error, made from `error_factory`, into the box."""
        err = self.error_factory(msg, error_t=error_t or self.error_t)
        self.append(err)

    def push_ret(self, msg: str, *, error_t: type[T] | None = None) -> Self:
        """Like `push` but also returns itself for convenience."""
        self.push(msg, error_t=error_t)
        return self
