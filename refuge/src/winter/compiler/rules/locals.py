from typing import override

from attrs import define, evolve, field

from winter.parser.ast import AST, Context
from winter.utils.ast_utils import WalkUpdate, walk_ast, walk_context

from .base import BaseRule, RuleError


class LocalsRuleError(RuleError): ...


SINGLE_NODE_ERR = "Couldn't lower `locals` into single node"


def get_ast_items(ast: AST.Type) -> list[AST.Type] | None:
    """Returns items list if node is of applicable type."""

    match ast:
        case AST.FUNCTION(body=x):
            return x
        case _:
            return None


def set_ast_items(ast: AST.Type, items: list[AST.Type]) -> AST.Type:
    """Sets items and returns a new node."""

    match ast:
        case AST.FUNCTION():
            return evolve(ast, body=items)
        case _:
            raise RuntimeError("Cannot set items here")


@define
class LocalsRule(BaseRule[LocalsRuleError]):
    error_t: type[LocalsRuleError] = field(default=LocalsRuleError, init=False)

    type _Res[X] = LocalsRule.Result[X]

    @override
    def run_context(self, ctx: Context) -> _Res[Context]:
        try:
            ctx = walk_context(
                ctx=ctx,
                ctx_cb=lambda x: WalkUpdate(
                    updated_t=self._raise_if_err(self._run_shallow_ctx(x))
                ),
                ast_cb=lambda x: WalkUpdate(
                    updated_t=self._raise_if_err(self._run_shallow(x))
                ),
            )
        except LocalsRuleError as err:
            return None, err
        return ctx, None

    @override
    def run(self, ast: AST.Type) -> _Res[AST.Type]:
        try:
            node = walk_ast(
                ast=ast,
                cb=lambda x: WalkUpdate(
                    updated_t=self._raise_if_err(self._run_shallow(x))
                ),
            )
        except LocalsRuleError as err:
            return None, err
        return node, None

    def _process_list(self, lst: list[AST.Type]) -> _Res[list[AST.Type]]:
        new_items: list[AST.Type] = []
        i = 0
        while True:
            try:
                x = lst[i]
            except IndexError:
                break
            if not (isinstance(x, AST.LOCALS) and not isinstance(x, AST.LET)):
                new_items.append(x)
                i += 1
                continue
            rest = lst[i + 1 :]
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
        return new_items, None

    def _run_shallow(self, ast: AST.Type) -> _Res[AST.Type]:
        if (ast_items := get_ast_items(ast)) is None or len(
            _locals := [
                x
                for x in ast_items
                if isinstance(x, AST.LOCALS) and not isinstance(x, AST.LET)
            ]
        ) == 0:
            return ast, None
        new_items, err = self._process_list(ast_items)
        if new_items is None or err:
            return None, err or LocalsRuleError(
                "`new_items` was none",
            )

        return set_ast_items(ast=ast, items=new_items), None

    def _run_shallow_ctx(self, ctx: Context) -> _Res[Context]:
        if (
            len(
                [
                    x
                    for x in ctx.body
                    if isinstance(x, AST.LOCALS) and not isinstance(x, AST.LET)
                ]
            )
            == 0
        ):
            return ctx, None
        new_items, err = self._process_list(ctx.body)
        if new_items is None or err:
            return None, err or LocalsRuleError(
                "`new_items` was none",
            )
        return evolve(ctx, body=new_items), None
