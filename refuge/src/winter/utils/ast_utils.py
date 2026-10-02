from collections.abc import Callable, Generator, Iterable

from attrs import define, evolve

from winter.parser.ast import AST, CallableHead, FuncParam, StringLikeAST


def _walk_asts(asts: Iterable[AST.Type]) -> Generator[AST.Type]:
    yield from (x for ast in asts for x in walk_ast(ast, cb))


@define
class WalkUpdate:
    updated_node: AST.Type


type _Callback = Callable[[AST.Type], WalkUpdate]


def _evolve_items(parent: AST.Type, items_key: str, cb: _Callback) -> AST.Type:
    items = getattr(parent, items_key)
    out_items = [walk_ast(i, cb) for i in items]
    return evolve(parent, **{items_key: out_items})


def walk_ast(
    ast: AST.Type,
    cb: _Callback,
) -> AST.Type:
    match ast:
        case AST.STRING_LIKE(parts=parts):
            yield from (
                w
                for p in parts
                if isinstance(p, StringLikeAST.EXPR)
                for w in walk_ast(p.value)
            )
        case AST.LIST():
            ast = _evolve_items(ast, "items", cb)
        case AST.SELECTION():
            ast = _evolve_items(ast, "items", cb)
        case AST.MAP(binds=binds):
            # TODO: :??? ? ??
            for bind_path, bound_value in binds:
                yield from _walk_asts(bind_path)
                yield from walk_ast(bound_value)
        case AST.CALLABLE(head=head, tail=tail):
            if not isinstance(head, CallableHead.OPERATOR):
                yield from walk_ast(head)
            yield from _walk_asts(tail)
        case AST.LET(binds=binds, body=body):
            yield from _walk_asts(x for _, x in binds)
            yield from walk_ast(body)
        case AST.LOCALS(binds=binds):
            yield from _walk_asts(x for _, x in binds)
        case AST.FUNCTION(params=params, body=body):
            yield from _walk_asts(
                param_def_value
                for unw in params
                if isinstance(unw, FuncParam.UNWRAPPED)
                for (_, param_def_value) in unw.params
                if param_def_value
            )
            yield from _walk_asts(body)
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
    return update.updated_node
