use std::collections::HashMap;

use itertools::Itertools;

use crate::parser::{Ast, AstVal, CallableHeadNodeType, FuncParam, StringLikeASTPart};

#[derive(Debug, Clone)]
/// Handles checking scope-related semantics such as:
/// - lowering `locals` into `let`
/// - lowering lone `inherits` into `let`
/// - merging same-scope `let` nodes
pub struct ScopeChecker<'src> {
    pub nodes: Vec<Ast<'src>>,
}

#[derive(Debug, Clone)]
pub struct Scope<'src> {
    pub binds: HashMap<&'src str, AstVal<'src>>,
}

#[derive(Debug, Clone)]
pub struct ScopeBind<'src> {
    pub value: AstVal<'src>,
    pub references: usize,
}

pub(super) type SemanRes<T> = Result<T, SemanticsError>;

impl<'src> ScopeChecker<'src> {
    pub fn new(nodes: Vec<Ast<'src>>) -> Self {
        Self { nodes }
    }

    pub fn handle_locals(&mut self) -> SemanRes<()> {
        let iter = self.nodes.iter_mut();
        for ast in iter {
            ast.value.walk(|x| match x {
                AstVal::Locals(locals) => {
                    // Find locals body!
                    todo!()
                }
                _ => (),
            });
        }

        Ok(())
    }

    fn error(&self, msg: &str, pos: usize) -> SemanticsError {
        let msg = msg.to_string();
        let Some(ast) = &self.nodes.get(pos).or(self.nodes.last()) else {
            return SemanticsError {
                msg,
                pos_start: 0,
                pos_end: 0,
            };
        };
        return SemanticsError {
            msg,
            pos_start: ast.pos_start,
            pos_end: ast.pos_end,
        };
    }
}

#[derive(Debug, Clone)]
pub struct SemanticsError {
    pub msg: String,
    pub pos_start: usize,
    pub pos_end: usize,
}
