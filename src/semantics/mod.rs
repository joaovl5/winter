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

impl<'src> ScopeChecker<'src> {
    /// Handle (locals) nodes and have them be transformed into (let) nodes
    pub fn handle_locals(&mut self) -> SemanRes<()> {
        let iter = self.module.nodes.iter_mut();
        for ast in iter {
            ast.value.walk_deep(&mut |args: &mut WalkArgs| {
                let node = match args {
                    WalkArgs::Ctx { idx, nodes } => &mut nodes[*idx],
                    WalkArgs::Val(val) => val,
                };
                // todo later
                // match node {
                //     AstVal::Locals(locals)
                // }
            });
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct SemanticsError {
    pub msg: String,
    pub pos_start: usize,
    pub pos_end: usize,
}
