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
    pub fn handle_locals(&mut self) -> SemanResMany<()> {
        let iter = self.module.nodes.iter_mut();
        let mut errors: Vec<SemanticsError> = Vec::new();
        for ast in iter {
            ast.value.walk_deep(&mut |args: &mut WalkArgs| {
                let node = match args {
                    // modules are always ok
                    WalkArgs::Module { idx, nodes } => &mut nodes[*idx].value,
                    WalkArgs::Ctx { idx, nodes, .. } => &mut nodes[*idx],
                    // we reject 'val' because locals are never valid in there
                    WalkArgs::Val(x) => x,
                };
                let AstVal::Locals(locals) = node else { return };
                // check for invalid use-cases of locals:
                let error_msg = match args {
                    // modules are always ok
                    WalkArgs::Module { .. } => None,
                    WalkArgs::Ctx { parent, nodes, .. } => {
                        if nodes.len() == 1 {
                            Some("cannot be used on its own!")
                        } else {
                            match parent {
                                AstVal::Selection(_) => Some("cannot be used in a path selection!"),
                                AstVal::Map(_) => Some("cannot be used for map keys/values!"),
                                _ => None,
                            }
                        }
                    }
                    WalkArgs::Val(_) => Some("cannot be used on its own!"),
                };
                if let Some(error_msg_val) = error_msg {
                    errors.push(SemanticsError {
                        msg: format!("`locals` {}\n context: {:#?}", error_msg_val, args),
                    });
                    return;
                }
                // we'll now deal with the "effective nodes" of the locals bindings
                // we care solely on the nodes that come *after* the locals
                match args {
                    WalkArgs::Module { idx, nodes } => {}
                    WalkArgs::Ctx { idx, nodes, parent } => {
                        let nodes_len = nodes.len();
                        let effective_nodes = nodes[idx..&mut nodes_len];
                    }
                    WalkArgs::Val(_) => (),
                };

                // todo later
                // match node {
                //     AstVal::Locals(locals)
                // }
            });
        }

        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SemanticsError {
    pub msg: String,
    // pub pos_start: usize,
    // pub pos_end: usize,
}
