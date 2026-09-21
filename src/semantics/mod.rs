use std::collections::HashMap;

// use itertools::Itertools;

use crate::parser::{walker::WalkArgs, AstVal, Module};

#[derive(Debug, Clone)]
/// Handles checking scope-related semantics such as:
/// - lowering `locals` into `let`
/// - lowering lone `inherits` into `let`
/// - merging same-scope `let` nodes
pub struct ScopeChecker<'x> {
    pub module: Box<Module<'x>>,
}

#[derive(Debug, Clone)]
pub struct Scope<'x> {
    pub binds: HashMap<&'x str, AstVal<'x>>,
}

#[derive(Debug, Clone)]
pub struct ScopeBind<'x> {
    pub value: AstVal<'x>,
    pub references: usize,
}

pub(super) type SemanRes<T> = Result<T, SemanticsError>;
pub(super) type SemanResMany<'x, T> = Result<T, Vec<SemanticsError>>;

impl<'ast> ScopeChecker<'ast> {
    /// Handle (locals) nodes and have them be transformed into (let) nodes
    pub fn handle_locals(&mut self) -> SemanResMany<'_, ()> {
        let mut errors: Vec<SemanticsError> = Vec::new();
        self.module.walk_deep(&mut |args: &mut WalkArgs| {
            let node = match args {
                WalkArgs::Module { idx, nodes } => &nodes[*idx].value,
                WalkArgs::Ctx { idx, nodes, .. } => &nodes[*idx],
                WalkArgs::Val(node) => node,
            };
            if !matches!(node, AstVal::Locals(_)) {
                return;
            }
            let error_msg = match args {
                WalkArgs::Module { idx, nodes } => (nodes[*idx + 1..]
                    .iter()
                    .all(|ast| matches!(ast.value, AstVal::Locals(_))))
                .then_some("must be followed by an expression!"),
                WalkArgs::Ctx { idx, nodes, parent } => match parent {
                    AstVal::Selection(_) => Some("cannot be used in a path selection!"),
                    AstVal::Map(_) => Some("cannot be used for map keys/values!"),
                    _ => (nodes[*idx + 1..]
                        .iter()
                        .all(|node| matches!(node, AstVal::Locals(_))))
                    .then_some("must be followed by an expression!"),
                },
                WalkArgs::Val(_) => Some("cannot be used on its own!"),
            };
            if let Some(error_msg) = error_msg {
                errors.push(SemanticsError {
                    msg: format!("`locals` {}\n context: {:#?}", error_msg, args),
                });
            }
        });
        if !errors.is_empty() {
            return Err(errors);
        }

        fn lower<'src, T>(nodes: &mut Vec<T>, value: impl Fn(&mut T) -> &mut AstVal<'src>) {
            // Lower later declarations first, so earlier bindings enclose them.
            for idx in (0..nodes.len()).rev() {
                if !matches!(value(&mut nodes[idx]), AstVal::Locals(_)) {
                    continue;
                }
                let mut declaration = nodes.remove(idx);
                let AstVal::Locals(locals) =
                    std::mem::replace(value(&mut declaration), AstVal::Nil)
                else {
                    unreachable!();
                };
                // Consumer-based placement belongs to a separate pass.
                for sibling in &mut nodes[idx..] {
                    let node = value(sibling);
                    let body = Box::new(std::mem::replace(node, AstVal::Nil));
                    *node = AstVal::Let {
                        locals: locals.clone(),
                        body,
                    };
                }
            }
        }

        self.module.walk_deep(&mut |args| match args {
            WalkArgs::Module { idx: 0, nodes } => lower(nodes, |ast| &mut ast.value),
            WalkArgs::Ctx {
                idx: 0,
                nodes,
                parent,
            } if !matches!(parent, AstVal::Selection(_) | AstVal::Map(_)) => {
                lower(nodes, |node| node);
            }
            _ => (),
        });
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SemanticsError {
    pub msg: String,
    // pub pos_start: usize,
    // pub pos_end: usize,
}
