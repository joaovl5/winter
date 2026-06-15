pub type OneNixVal<'x> = Box<NixVal<'x>>;
pub type ManyNixVal<'x> = Vec<NixVal<'x>>;

pub type NixBindings<'x> = Vec<NixBinding<'x>>;
pub type NixStringLikeParts<'x> = Vec<NixStringLikePart<'x>>;
pub type NixDotPathParts<'x> = Vec<NixDotPathPart<'x>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NixVal<'src> {
    Null,
    Bool(bool),
    Number(&'src str),
    Identifier(&'src str),
    String(NixStringLikeParts<'src>),
    Path(NixStringLikeParts<'src>),
    List(ManyNixVal<'src>),
    AttrSet(NixBindings<'src>),
    /// Application of an operator `op` amongst a left side `lhs` and a right side `rhs`
    BinaryOperator {
        lhs: OneNixVal<'src>,
        rhs: OneNixVal<'src>,
        op: String,
    },
    /// Application of an operator `op` amongst a sole value `val`
    UnaryOperator {
        op: String,
        val: OneNixVal<'src>,
    },
    /// Lambda, a single argument function, which may be derived from a larger
    /// multi-argument function, which in that case will yield nested lambdas
    Lambda {
        arg: NixLambdaArg<'src>,
        body: OneNixVal<'src>,
    },
    /// Let bind, derived either from a Winter `let` bind or through scope-inference of
    /// `locals` and/or `inherit` forms
    Let {
        // Although the same type from `AttrSet` is used, which allows for dotted
        // paths, it is assumed that no dotted paths will take place
        entries: NixBindings<'src>,
        body: OneNixVal<'src>,
    },
    /// Application of some expression (assumedly a function) with some non-zero amount
    /// of arguments
    Apply {
        function: OneNixVal<'src>,
        args: ManyNixVal<'src>,
    },
    /// Selection of a `path` from a `base` value, using the dot syntax
    Select {
        base: OneNixVal<'src>,
        path: Vec<NixDotPathPart<'src>>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NixBinding<'src> {
    Assign(NixDotPathParts<'src>, OneNixVal<'src>),
    Inherit {
        from: Option<OneNixVal<'src>>,
        names: Vec<&'src str>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NixLambdaArg<'src> {
    Plain(&'src str),
    Unwrapped {
        /// name for arg, if it is named
        name: Option<&'src str>,
        /// list of unpacked parameters that
        /// - have names
        /// - may have optional defaults - rendered with `?` operator
        params: Vec<(&'src str, Option<OneNixVal<'src>>)>,
        /// If set, will render with ellipsis
        allow_extra: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NixStringLikePart<'src> {
    Text(String),
    Expr(OneNixVal<'src>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NixDotPathPart<'src> {
    Identifier(&'src str),
    String(NixStringLikeParts<'src>),
}

impl<'src> NixVal<'src> {
    pub fn render(&self) -> String {
        match self {
            NixVal::Null => "null".to_string(),
            NixVal::Bool(x) => x.to_string(),
            NixVal::Number(x) => x.to_string(),
            NixVal::Identifier(x) => x.to_string(),
            NixVal::String(x) => render_nix_string(x),
            NixVal::Path(x) => render_nix_path(x),
            NixVal::List(x) => render_list(x),
            NixVal::AttrSet(x) => render_attrset(x),
            NixVal::BinaryOperator { lhs, rhs, op } => {
                format!("{} {} {}", lhs.render_as_arg(), op, rhs.render_as_arg())
            }
            NixVal::UnaryOperator { op, val } => format!("{op} {}", val.render_as_arg()),
            NixVal::Lambda { arg, body } => {
                format!("{}: {}", render_lambda_arg(arg), body.render())
            }
            NixVal::Let { entries, body } => render_let(entries, body),
            NixVal::Apply { function, args } => render_apply(function, args),
            NixVal::Select { base, path } => render_select(base, path),
        }
    }

    fn render_as_arg(&self) -> String {
        let rendered = self.render();
        match self {
            NixVal::Lambda { .. }
            | NixVal::Let { .. }
            | NixVal::Apply { .. }
            | NixVal::BinaryOperator { .. }
            | NixVal::UnaryOperator { .. } => format!("({rendered})"),
            _ => rendered,
        }
    }
}

fn render_list(lst: &[NixVal]) -> String {
    if lst.is_empty() {
        return "[]".to_string();
    }

    let lst = lst
        .iter()
        .map(|x| x.render_as_arg())
        .collect::<Vec<_>>()
        .join(" ");

    format!("[ {lst} ]")
}

fn render_attrset(entries: &[NixBinding]) -> String {
    if entries.is_empty() {
        return "{}".to_string();
    }

    format!("{{ {} }}", render_bindings(entries))
}

fn render_let(entries: &[NixBinding], body: &NixVal) -> String {
    format!("let {} in {}", render_bindings(entries), body.render())
}

fn render_bindings(entries: &[NixBinding]) -> String {
    entries
        .iter()
        .map(render_binding)
        .collect::<Vec<_>>()
        .join(" ")
}

fn render_binding(binding: &NixBinding) -> String {
    match binding {
        NixBinding::Assign(path, value) => {
            format!("{} = {};", render_dot_path(path), value.render())
        }
        NixBinding::Inherit { from, names } => {
            let from = from
                .as_ref()
                .map(|x| format!(" ({})", x.render()))
                .unwrap_or_default();
            format!("inherit{from} {};", names.join(" "))
        }
    }
}

fn render_lambda_arg(arg: &NixLambdaArg) -> String {
    match arg {
        NixLambdaArg::Plain(x) => x.to_string(),
        NixLambdaArg::Unwrapped {
            name,
            params,
            allow_extra,
        } => {
            let mut parts = params
                .iter()
                .map(|(name, default)| match default {
                    Some(value) => format!("{name} ? {}", value.render()),
                    None => name.to_string(),
                })
                .collect::<Vec<_>>();
            if *allow_extra {
                parts.push("...".to_string());
            }
            let pattern = if parts.is_empty() {
                "{}".to_string()
            } else {
                format!("{{ {} }}", parts.join(", "))
            };
            match name {
                Some(name) => format!("{name}@{pattern}"),
                None => pattern,
            }
        }
    }
}

fn render_apply(function: &NixVal, args: &[NixVal]) -> String {
    let mut out = Vec::with_capacity(args.len() + 1);
    out.push(format!("({})", function.render()));
    out.extend(args.iter().map(|x| x.render_as_arg()));
    out.join(" ")
}

fn render_select(base: &NixVal, path: &[NixDotPathPart]) -> String {
    let mut out = format!("({})", base.render());
    for k in path {
        out.push('.');
        out.push_str(&render_dot_path_part(k));
    }
    out
}

fn render_dot_path(path: &[NixDotPathPart]) -> String {
    path.iter()
        .map(render_dot_path_part)
        .collect::<Vec<_>>()
        .join(".")
}

fn render_dot_path_part(part: &NixDotPathPart) -> String {
    match part {
        NixDotPathPart::Identifier(x) => x.to_string(),
        NixDotPathPart::String(x) => render_nix_string(x),
    }
}

fn render_nix_string(parts: &[NixStringLikePart]) -> String {
    let mut out = String::from("\"");
    for part in parts {
        match part {
            NixStringLikePart::Text(x) => out.push_str(&escape_nix_string(x)),
            NixStringLikePart::Expr(x) => out.push_str(&format!("${{{}}}", x.render())),
        }
    }
    out.push('"');
    out
}

fn render_nix_path(parts: &[NixStringLikePart]) -> String {
    parts
        .iter()
        .map(|part| match part {
            NixStringLikePart::Text(x) => x.clone(),
            NixStringLikePart::Expr(x) => format!("${{{}}}", x.render()),
        })
        .collect::<Vec<_>>()
        .join("")
}

fn escape_nix_string(x: &str) -> String {
    let mut out = String::new();
    let mut chars = x.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '$' if chars.peek() == Some(&'{') => out.push_str("\\$"),
            _ => out.push(ch),
        }
    }

    out
}
