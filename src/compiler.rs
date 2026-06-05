use crate::{nix::NixExpr, parser::Thing, Parser};

pub fn compile_source(source: &str) -> Result<String, String> {
    let thing = Parser::new(source).parse()?;

    todo!()
}

fn lower_thing(thing: &Thing) -> Result<NixExpr, String> {
    match thing {
        Thing::Atom(x) => Ok(lower_atom(x)),
        Thing::String(x) => Ok(NixExpr::String(x.clone())),
        Thing::List(x) => lower_list(items),
    }
}

fn lower_atom(atom: &str) -> NixExpr {
    match atom {
        "true" => NixExpr::Bool(true),
        "false" => NixExpr::Bool(false),
        "nil" => NixExpr::Null,
        _ => NixExpr::Identifier(atom.to_string()),
    }
}

fn lower_list(lst: &[Thing]) -> Result<NixExpr, String> {
    let Some(head) = lst.first() else {
        return Err("empty lists aren't valid expressions yet".to_string());
    };
    let rest = &lst[1..];

    match head {
        Thing::Atom(x) => match x.as_str() {
            "attrs" => lower_attrs(rest),
            "list" => lst[1..]
                .iter()
                .map(lower_thing)
                .collect::<Result<Vec<_>, _>>()
                .map(NixExpr::List),
            "fn" => lower_fn(rest),
            "let" => lower_let(rest),
            "select" => lower_select(rest),
            "path" => lower_path(rest),
            "call" => lower_call(rest),
            _ => lower_application(lst),
        },
        _ => lower_application(lst),
    }
}

fn lower_let(lst: &[Thing]) -> Result<NixExpr, String> {
    if lst.len() != 2 {
        return Err("'let' expects a binding list and a body".to_string());
    }

    let Thing::List(binding_things) = &lst[0] else {
        return Err("'let' bindings must be a list".to_string());
    };
    let mut bindings_iter = binding_things.iter();

    let mut bindings = Vec::with_capacity(binding_things.len());

    while let Some(k) = bindings_iter.next() {
        let Thing::Atom(name) = k else {
            return Err("'let' binding names must be atoms".to_string());
        };

        let Some(v) = bindings_iter.next() else {
            return Err(format!("Missing value for binding '{}'", name));
        };
        bindings.push((name.clone(), lower_thing(v)?));
    }

    Ok(NixExpr::Let {
        entries: bindings,
        body: Box::new(lower_thing(&lst[1])?),
    })
}

fn lower_fn(lst: &[Thing]) -> Result<NixExpr, String> {
    if lst.len() != 2 {
        return Err("fn expects exactly one argument and a body".to_string());
    }

    let Thing::Atom(arg) = &lst[0] else {
        return Err("fn arg must be an atom".to_string());
    };

    Ok(NixExpr::Lambda {
        arg: arg.clone(),
        body: Box::new(lower_thing(&lst[1])?),
    })
}

fn lower_attrs(entries: &[Thing]) -> Result<NixExpr, String> {
    let mut lowered = Vec::with_capacity(entries.len());
    let mut entries_iter = entries.iter();

    while let Some(k) = entries_iter.next() {
        let Some(v) = entries_iter.next() else {
            return Err(format!("Missing value for attrset key '{}'", k));
        };

        lowered.push((make_attr_key(k)?, lower_thing(v)?));
    }

    Ok(NixExpr::AttrSet(lowered))
}

fn make_attr_key(thing: &Thing) -> Result<String, String> {
    match thing {
        Thing::Atom(x) | Thing::String(x) => Ok(x.clone()),
        _ => Err("Attrset keys must be atoms or strings".to_string()),
    }
}
