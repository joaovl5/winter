from collections.abc import Callable

from attrs import define, evolve

from winter.parser.ast import AST, CallableHead, Context, FuncParam, StringLikeAST


@define
class WalkUpdate[T]:
    updated_t: T


type _Callback[T] = Callable[[T], WalkUpdate[T]]


def _evolve_items(
    parent: AST.Type,
    items_key: str,
    cb: _Callback[AST.Type],
    **kwargs: object,
) -> AST.Type:
    items = getattr(parent, items_key)
    out_items = [walk_ast(i, cb) for i in items]
    return evolve(
        parent,
        **{items_key: out_items},
        **kwargs,
    )


def walk_ast[T: AST.Type](
    ast: AST.Type,
    cb: _Callback[AST.Type],
) -> AST.Type:
    match ast:
        case AST.STRING_LIKE(parts=parts):
            out_parts = [
                evolve(p, value=walk_ast(p.value, cb))
                if isinstance(p, StringLikeAST.EXPR)
                else p
                for p in parts
            ]
            ast = evolve(ast, parts=out_parts)
        case AST.LIST():
            ast = _evolve_items(ast, "items", cb)
        case AST.SELECTION():
            ast = _evolve_items(ast, "items", cb)
        case AST.MAP(binds=binds):
            # we don't treat the bind_path as a list of items, since its only
            # an impl detail plus `(locals bla bla).(thing)` is weird
            out_binds = [
                (bind_path, walk_ast(bind_value, cb)) for bind_path, bind_value in binds
            ]
            ast = evolve(ast, binds=out_binds)
        case AST.CALLABLE(head=head):
            out_head = head
            if not isinstance(head, CallableHead.OPERATOR):
                out_head = walk_ast(head, cb)
            ast = _evolve_items(ast, "tail", cb, head=out_head)
        case AST.LET(binds=binds, body=body):
            out_binds = [(k, walk_ast(v, cb)) for k, v in binds]
            out_body = walk_ast(body, cb)
            ast = evolve(ast, binds=out_binds, body=out_body)
        case AST.LOCALS(binds=binds):
            out_binds = [(k, walk_ast(v, cb)) for k, v in binds]
            ast = evolve(ast, binds=out_binds)
        case AST.FUNCTION(params=params, body=body):
            out_params = [
                (
                    evolve(
                        param,
                        params=[
                            (p_name, walk_ast(p_value, cb) if p_value else None)
                            for p_name, p_value in param.params
                        ],
                    )
                    if isinstance(param, FuncParam.UNWRAPPED)
                    else param
                )
                for param in params
            ]
            ast = _evolve_items(ast, "body", cb, params=out_params)
        case AST.IF(
            condition=condition,
            case_true=case_true,
            case_false=case_false,
        ):
            ast = evolve(
                ast,
                condition=walk_ast(condition, cb),
                case_true=walk_ast(case_true, cb),
                case_false=walk_ast(case_false, cb),
            )
        case (
            AST.NIL()
            | AST.BOOL()
            | AST.NUMBER()
            | AST.IDENTIFIER()
            | AST.DECLARED_TYPE()
            | AST.INHERITS()
        ):
            pass

    update = cb(ast)
    return update.updated_t


def walk_context(
    ctx: Context,
    ctx_cb: _Callback[Context],
    ast_cb: _Callback[AST.Type],
) -> Context:
    out_items = [walk_ast(i, ast_cb) for i in ctx.body]
    ctx = evolve(ctx, body=out_items)
    ctx_update = ctx_cb(ctx)
    return ctx_update.updated_t
