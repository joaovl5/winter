from typing import override

from attrs import define, evolve, field

from winter.parser.ast import AST
from winter.utils.ast_utils import WalkUpdate, walk_ast

from .base import BaseRule, RuleError


class LocalsRuleError(RuleError): ...


SINGLE_NODE_ERR = "Couldn't lower `locals` into single node"


def get_ast_items(ast: AST.Type) -> list[AST.Type] | None:
    """Returns items list if node is of applicable type."""

    match ast:
        case AST.LIST(items=x):
            return x
        case AST.FUNCTION(body=x):
            return x
        case _:
            return None


def set_ast_items(ast: AST.Type, items: list[AST.Type]) -> AST.Type:
    """Sets items and returns a new node."""

    match ast:
        case AST.LIST():
            return evolve(ast, items=items)
        case AST.FUNCTION():
            return evolve(ast, body=items)
        case _:
            raise RuntimeError("Cannot set items here")


@define
class LocalsRule(BaseRule[LocalsRuleError]):
    error_t: type[LocalsRuleError] = field(default=LocalsRuleError, init=False)

    type _Res[X] = LocalsRule.Result[X]

    @override
    def run(self, ast: AST.Type) -> _Res[AST.Type | None]:
        try:
            node = walk_ast(
                ast=ast,
                cb=lambda x: WalkUpdate(
                    updated_node=self._raise_if_err(self._run_single(x))
                ),
            )
        except LocalsRuleError as err:
            return None, err
        return node, None

    def _run_single(self, ast: AST.Type) -> _Res[AST.Type | None]:

        if (ast_items := get_ast_items(ast)) is None or len(
            _locals := [
                x
                for x in ast_items
                if isinstance(x, AST.LOCALS) and not isinstance(x, AST.LET)
            ]
        ) == 0:
            return ast, None
        new_items: list[AST.Type] = []
        i = 0
        while True:
            try:
                x = ast_items[i]
            except IndexError:
                break
            if not (isinstance(x, AST.LOCALS) and not isinstance(x, AST.LET)):
                new_items.append(x)
                i += 1
                continue
            rest = ast_items[i + 1 :]
            if len(rest) == 0:
                break
            if len(rest) > 1:
                return None, LocalsRuleError(SINGLE_NODE_ERR)
            let_ast = AST.LET(
                pos_start=x.pos_start,
                pos_end=rest[0].pos_end,
                inherits=x.inherits,
                binds=x.binds,
                body=rest[0],
            )
            new_items.append(let_ast)
            break
        return set_ast_items(ast=ast, items=new_items), None
