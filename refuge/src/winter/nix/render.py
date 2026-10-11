lazy from .ast import AST, Binding, LambdaArg, StringLikePart


def render_nix_ast(node: AST.Type) -> str:
    match node:
        case AST.NULL():
            return "null"
        case AST.BOOL(value=value):
            return "true" if value else "false"
        case AST.IDENTIFIER(value=value) | AST.NUMBER(value=value):
            return value
        case AST.STRING_LIKE():
            return _str_like(node)
        case AST.LIST():
            return _list(node)
        case AST.ATTRSET():
            return _attrset(node)
        case AST.LET():
            return _let(node)
        case AST.N_ARY_OP(items=items, op=op):
            return f" {op} ".join(_render_as_arg(x) for x in items)
        case AST.BINARY_OP(lhs=lhs, rhs=rhs, op=op):
            lhs = _render_as_arg(lhs)
            rhs = _render_as_arg(rhs)
            return lhs + f" {op} " + rhs
        case AST.UNARY_OP(value=value, op=op):
            value = _render_as_arg(value)
            return f"{op} " + value
        case AST.ATTRSET_HAS(
            attrset=attrset,
            has_expr=has_expr,
        ):
            attrset_nix = render_nix_ast(attrset)
            has_expr_nix = render_nix_ast(has_expr)
            return f"{attrset_nix} ? {has_expr_nix}"
        case AST.ATTRSET_OR(
            attrset=attrset,
            maybe_expr=maybe_expr,
            else_expr=else_expr,
        ):
            attrset_nix = render_nix_ast(attrset)
            maybe_expr_nix = render_nix_ast(maybe_expr)
            else_expr_nix = render_nix_ast(else_expr)
            return f"{attrset_nix}.{maybe_expr_nix} or {else_expr_nix}"
        case AST.LAMBDA(arg=arg, body=body):
            arg = _render_lambda_arg(arg)
            body = render_nix_ast(body)
            return f"{arg}: {body}"
        case AST.APPLY(function=f, args=args):
            parts: list[str] = [f"({render_nix_ast(f)})"]
            parts.extend(_render_as_arg(x) for x in args)
            return " ".join(parts)
        case AST.SELECTION(base=x, path=path):
            out: list[str] = [f"({render_nix_ast(x)})"]
            for k in path:
                out.append(".")
                out.append(_dot_path_part(k))
            return "".join(out)
        case AST.IF(
            condition=condition,
            case_true=case_true,
            case_false=case_false,
        ):
            return (
                "if "
                + _render_as_arg(condition)
                + " then "
                + _render_as_arg(case_true)
                + " else "
                + _render_as_arg(case_false)
            )


def _render_lambda_arg(arg: LambdaArg.Type):
    if isinstance(arg, LambdaArg.PLAIN):
        return arg.value
    # unwrapped case
    parts = [
        name
        + (
            " ? " + default_str
            if default is not None and (default_str := render_nix_ast(default))
            else ""
        )
        for name, default in arg.params
    ]
    if arg.allow_extra:
        parts.append("...")
    pat = "{}" if len(parts) == 0 else (f"{{ {', '.join(parts)} }}")
    return pat if arg.name is None else f"{arg.name}@{pat}"


def _let(node: AST.LET) -> str:
    entries_str = _render_bindings(node.entries)
    body_str = render_nix_ast(node.body)
    return "let " + entries_str + " in " + body_str


def _attrset(node: AST.ATTRSET) -> str:
    if len(node.entries) == 0:
        return "{}"
    return "{ " + _render_bindings(node.entries) + " }"


def _render_bindings(entries: list[Binding.Type]) -> str:
    return " ".join(_render_binding(x) for x in entries)


def _render_binding(entry: Binding.Type) -> str:
    match entry:
        case (str(inherit_ns), inherit_items):
            if inherit_items is not None:
                items = " ".join(inherit_items)
                return f"inherit ({inherit_ns}) {items};"
            else:
                return f"inherit {inherit_ns}; "
        case (list(dot_path), bound_value):
            dot_path_str = _dot_path(dot_path)
            return f"{dot_path_str} = {render_nix_ast(bound_value)};"


def _dot_path(lst: list[AST.Type]):
    return ".".join(_dot_path_part(x) for x in lst)


def _dot_path_part(x: AST.Type):
    match x:
        case AST.IDENTIFIER():
            return x.value
        case AST.STRING_LIKE():
            return _str_like(x)
        case _:
            raise RuntimeError("Unexpected AST for dotted path")


def _list(node: AST.LIST) -> str:
    if len(node.entries) == 0:
        return "[]"
    out = " ".join(_render_as_arg(x) for x in node.entries)
    return f"[ {out} ]"


def _render_as_arg(node: AST.Type) -> str:
    x = render_nix_ast(node)
    return f"({x})"


def _str_like(node: AST.STRING_LIKE) -> str:
    out: list[str] = []
    for part in node.value:
        part_str = _str_like_part(part, is_path=node.is_path)
        out.append(part_str)
    out_str = "".join(out)
    return out_str if node.is_path else f'"{out_str}"'


def _str_like_part(part: StringLikePart.Type, *, is_path: bool) -> str:
    match x := part.value:
        case str():
            return x if is_path else _escape_str(x)
        case _:  # expr
            return "${" + render_nix_ast(x) + "}"


_ESCAPE_LITERALS = (
    ('"', '\\"'),
    ("\\", "\\\\"),
    ("\n", "\\n"),
    ("\r", "\\r"),
    ("\t", "\\t"),
    ("${", "\\${"),
)


def _escape_str(x: str) -> str:
    out = x
    for before, after in _ESCAPE_LITERALS:
        out = out.replace(before, after)
    return out
