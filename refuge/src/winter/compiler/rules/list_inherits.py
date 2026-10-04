from typing import override

from attrs import define, evolve, field
from loguru import logger

from winter.parser.ast import AST, Context
from winter.utils.ast_utils import WalkUpdate, walk_ast, walk_context

from .base import BaseRule, RuleError


class ListInheritsRuleError(RuleError): ...


WARN_INHERIT_ITEMLESS = (
    "at {}→{}, the list-based inherit doesn't have inherited items; "
    "thus defaulting to just replacing it with the inherit namespace"
    " as an identifier: `{}`"
)


@define
class ListInheritsRule(BaseRule[ListInheritsRuleError]):
    error_t: type[ListInheritsRuleError] = field(
        default=ListInheritsRuleError,
        init=False,
    )

    type _Res[X] = ListInheritsRule.Result[X]

    @override
    def run_context(self, ctx: Context) -> _Res[Context]:
        try:
            ctx = walk_context(
                ctx=ctx,
                ctx_cb=lambda x: WalkUpdate(updated_t=x),
                ast_cb=lambda x: WalkUpdate(
                    updated_t=self._raise_if_err(self._run_single(x))
                ),
            )
        except ListInheritsRuleError as err:
            return None, err
        return ctx, None

    @override
    def run(self, ast: AST.Type) -> _Res[AST.Type]:
        try:
            node = walk_ast(
                ast=ast,
                cb=lambda x: WalkUpdate(
                    updated_t=self._raise_if_err(self._run_single(x))
                ),
            )
        except ListInheritsRuleError as err:
            return None, err
        return node, None

    def _run_single(self, ast: AST.Type) -> _Res[AST.Type | None]:
        if not isinstance(ast, AST.LIST):
            return ast, None
        out_items: list[AST.Type] = []
        for item in ast.items:
            if not (
                isinstance(item, AST.INHERITS)
                and not isinstance(item, AST.LOCALS)
                and not isinstance(item, AST.LET)
            ):
                out_items.append(item)
                continue
            for inherit_ns, inherited_items in item.inherits:
                ns_id = AST.IDENTIFIER(
                    pos_start=item.pos_start,
                    pos_end=item.pos_end,
                    value=inherit_ns,
                )
                if not inherited_items:
                    logger.warning(
                        WARN_INHERIT_ITEMLESS,
                        item.pos_start,
                        item.pos_end,
                        inherit_ns,
                    )
                    out_items.append(ns_id)
                    continue
                item_ids = (
                    AST.IDENTIFIER(
                        pos_start=item.pos_start, pos_end=item.pos_end, value=x.name
                    )
                    for x in inherited_items
                )
                sels = (
                    AST.SELECTION(
                        pos_start=item.pos_start,
                        pos_end=item.pos_end,
                        items=[ns_id, id],
                    )
                    for id in item_ids
                )
                out_items.extend(sels)
        out_ast = evolve(ast, items=out_items)
        return out_ast, None
