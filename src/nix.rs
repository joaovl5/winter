type OneNixVal<'x> = Box<NixVal<'x>>;
type NixValList<'x> = Vec<NixVal<'x>>;
type NixValMap<'x> = Vec<(String, NixVal<'x>)>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NixVal<'src> {
    // TODO: numbers, operators
    Null,
    Bool(bool),
    Number(&'src str),
    Identifier(&'src str),
    String(String),
    Path(String),
    List(NixValList<'src>),
    AttrSet(NixValMap<'src>),
    Lambda {
        arg: String,
        body: OneNixVal<'src>,
    },
    Let {
        entries: NixValMap<'src>,
        body: OneNixVal<'src>,
    },
    Apply {
        function: OneNixVal<'src>,
        args: NixValList<'src>,
    },
    Select {
        base: OneNixVal<'src>,
        path: Vec<String>,
    },
}

impl<'src> NixVal<'src> {
    pub fn render(&self) -> String {
        match self {
            NixVal::Identifier(x) => x.to_string(),
            NixVal::Path(x) => x.clone(),
            NixVal::Bool(x) => x.to_string(),
            NixVal::Null => "null".to_string(),
            NixVal::String(x) => render_nix_string(x),
            NixVal::List(x) => render_list(x),
            NixVal::AttrSet(x) => render_attrset(x),
            NixVal::Lambda { arg, body } => format!("{arg}: {}", body.render()),
            NixVal::Let { entries, body } => render_let(entries, body),
            NixVal::Apply { function, args } => render_apply(function, args),
            NixVal::Select { base, path } => render_select(base, path),
        }
    }

    fn render_as_arg(&self) -> String {
        let rendered = self.render();
        match self {
            NixVal::Lambda { .. } | NixVal::Let { .. } | NixVal::Apply { .. } => {
                format!("({rendered})")
            }
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

fn render_attrset(entries: &NixValMap) -> String {
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

fn render_let(entries: &NixValMap, body: &NixVal) -> String {
    let mut out = String::from("let ");

    for (k, v) in entries {
        out.push_str(&format!("{} = {}; ", k, v.render()));
    }

    out.push_str("in ");
    out.push_str(&body.render());

    out
}

fn render_apply(function: &NixVal, args: &NixValList) -> String {
    let mut out = Vec::with_capacity(args.len() + 1);
    out.push(format!("({})", function.render()));
    out.extend(args.iter().map(|x| x.render_as_arg()));
    out.join(" ")
}

fn render_select(base: &NixVal, path: &[String]) -> String {
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
