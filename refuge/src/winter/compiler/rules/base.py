from abc import ABC, abstractmethod

from attrs import define

from winter.parser.ast import AST


class RuleError(BaseException): ...


@define
class BaseRule[
    Err: RuleError = RuleError,
](ABC):
    """Abstract compiler rule."""

    error_t: type[Err]

    type Result[X] = tuple[X, Err | None]

    @abstractmethod
    def run[T: AST.Type](self, ast: T) -> Result[T | None]:
        """Execute the rule on an AST node.

        Returns tuple with a possibly modified or null AST node, and a potential error.
        """
