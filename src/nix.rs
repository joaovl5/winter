type SomeNixExpr = Box<NixExpr>;
type NixExprList = Vec<NixExpr>;
type NixExprMap = Vec<(String, NixExpr)>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NixExpr {
    // TODO: numbers, operators
    Null,
    Identifier(String),
    String(String),
    Bool(bool),
    Path(String),
    List(NixExprList),
    AttrSet(NixExprMap),
    Lambda {
        arg: String,
        body: SomeNixExpr,
    },
    Let {
        entries: NixExprMap,
        body: SomeNixExpr,
    },
    Apply {
        function: SomeNixExpr,
        args: NixExprList,
    },
    Select {
        base: SomeNixExpr,
        path: Vec<String>,
    },
}

impl NixExpr {
    pub fn render(&self) -> String {
        match self {
            NixExpr::Identifier(x) => x.clone(),
            NixExpr::Path(x) => x.clone(),
            NixExpr::Bool(x) => x.to_string(),
            NixExpr::Null => "null".to_string(),
            NixExpr::String(x) => render_nix_string(x),
            NixExpr::List(x) => render_list(x),
            NixExpr::AttrSet(x) => render_attrset(x),
            NixExpr::Lambda { arg, body } => format!("{arg}: {}", body.render()),
            NixExpr::Let { entries, body } => render_let(entries, body),
            NixExpr::Apply { function, args } => render_apply(function, args),
            NixExpr::Select { base, path } => render_select(base, path),
        }
    }

    fn render_as_arg(&self) -> String {
        let rendered = self.render();
        match self {
            NixExpr::Lambda { .. } | NixExpr::Let { .. } | NixExpr::Apply { .. } => {
                format!("({rendered})")
            }
            _ => rendered,
        }
    }
}

fn render_list(lst: &[NixExpr]) -> String {
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

fn render_attrset(entries: &NixExprMap) -> String {
    if entries.is_empty() {
        return "{}".to_string();
    }

    let mut out = String::new();
    out.push_str("{ ");
    for (k, v) in entries {
        out.push_str(&format!("{} = {}; ", render_nix_string(k), v.render()));
    }
    out.push_str(" }");

    out
}

fn render_let(entries: &NixExprMap, body: &NixExpr) -> String {
    let mut out = String::from("let ");

    for (k, v) in entries {
        out.push_str(&format!("{} = {}; ", k, v.render()));
    }

    out.push_str("in ");
    out.push_str(&body.render());

    out
}

fn render_apply(function: &NixExpr, args: &NixExprList) -> String {
    let mut out = Vec::with_capacity(args.len() + 1);
    out.push(format!("({})", function.render()));
    out.extend(args.iter().map(|x| x.render_as_arg()));
    out.join(" ")
}

fn render_select(base: &NixExpr, path: &[String]) -> String {
    let mut out = format!("({})", base.render());
    for k in path {
        out.push('.');
        out.push_str(&render_nix_string(k));
    }
    out
}

fn render_nix_string(x: &str) -> String {
    let mut out = String::from("\"");
    let mut chars = x.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\t"),
            '$' if chars.peek() == Some(&'{') => out.push_str("\\$"),
            _ => out.push(ch),
        }
    }

    out.push('"');
    out
}
