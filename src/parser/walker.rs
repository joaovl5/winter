use itertools::Itertools;

use crate::parser::{
    Ast, AstVal, CallableHeadNodeType, FuncParam, ManyNodes, Module, StringLikeASTPart,
};

/// Action that applies  during `AstVal.walk`
pub enum WalkAction<'x> {
    None,
    ReplaceSelf(AstVal<'x>),
    RemoveSelf,
    InsertBefore(AstVal<'x>),
    InsertAfter(AstVal<'x>),
    RewriteItems(ManyNodes<'x>),
}

pub struct WalkCtx<'x, 'y> {
    pub index: usize,
    pub items: &'y [AstVal<'x>],
}

pub struct WalkArgs<'x, 'y> {
    pub node: &'y AstVal<'x>,
    pub ctx: WalkCtx<'x, 'y>,
}

type WalkCallback<'src> = dyn for<'walk> FnMut(WalkArgs<'src, 'walk>) -> WalkAction<'src>;

impl<'src> Module<'src> {
    fn apply_walk_callback<'cb>(
        &mut self,
        callback: &mut WalkCallback<'src>,
        node: AstVal<'src>,
        index: usize,
        items: &'cb mut Vec<AstVal<'src>>,
    ) {
        let action = callback(WalkArgs {
            node: &node,
            ctx: WalkCtx { index, items },
        });

        match action {
            WalkAction::None => return,
            WalkAction::ReplaceSelf(new_node) => {
                items[index] = new_node;
            }
            WalkAction::RemoveSelf => {
                items.remove(index);
            }
            WalkAction::InsertBefore(new_node) => {
                items.insert(index, new_node);
            }
            WalkAction::InsertAfter(new_node) => {
                if index + 1 > items.len() {
                    items.push(new_node);
                } else {
                    items.insert(index + 1, new_node);
                }
            }
            WalkAction::RewriteItems(new_items) => {
                *items = new_items;
            }
        };
    }

    /// Walk through AST with a callback that is called at each node or sub-node
    ///
    /// * `callback`: Function taking a mutable AstVal, and an optional `context` of
    ///   sibling nodes.
    pub fn walk(
        &mut self,
        // callback for consuming walked nodes
        callback: &mut WalkCallback<'src>,
    ) {
        let mut self_iter = self.nodes.iter_mut().enumerate().collect_vec();
        for (index, node) in self_iter {
            use AstVal::*;
            self.apply_walk_callback(
                callback,
                node.value,
                index,
                &mut self.nodes.iter().map(|x| (*x).value).collect(),
            );
            // REFACTOR TO INCLUDE RECURSION LATER
            match &mut node.value {
                StringLike { parts, .. } => {
                    let mut expr_parts: Vec<AstVal<'src>> = parts
                        .iter()
                        .filter_map(|part| match part {
                            StringLikeASTPart::Expr(expr) => Some(*expr),
                            StringLikeASTPart::Text(_) => None,
                        })
                        .collect();
                    let mut inner_index = 0;
                    for expr_p in expr_parts {
                        self.apply_walk_callback(callback, expr_p, inner_index, &mut expr_parts);
                        inner_index += 1;
                    }
                }
                List(nodes) => {
                    let mut inner_index = 0;
                    for node in nodes {
                        self.apply_walk_callback(callback, *node, inner_index, &mut nodes);
                        inner_index += 1;
                    }
                }
                Map(map_val) => {
                    let mut inner_index = 0;
                    let mut mapped_items = map_val.binds.iter().map(|(_, x)| *x).collect();
                    for (key_parts, value) in map_val.binds {
                        self.apply_walk_callback(callback, value, inner_index, &mut mapped_items);
                        inner_index += 1;
                    }
                }
                Callable(head, nodes) => {
                    if let CallableHeadNodeType::Expr(head_expr) = head {
                        let expr = head_expr.as_mut();
                        // TODO RECURSE
                        self.apply_walk_callback(callback, *expr, 0, &mut Vec::new());
                    }
                    let mut inner_index = 0;
                    for node in nodes {
                        self.apply_walk_callback(callback, *node, inner_index, nodes);
                        inner_index += 1;
                    }
                }
                Selection(nodes) => {
                    let mut inner_index = 0;
                    for node in nodes {
                        self.apply_walk_callback(callback, *node, inner_index, nodes);
                        inner_index += 1;
                    }
                }
                Let { locals, body } => {
                    let mut inner_index = 0;
                    let mut mapped_binds = locals.binds.iter().map(|(_, x)| *x).collect();
                    for (_, node) in &mut locals.binds {
                        self.apply_walk_callback(callback, *node, inner_index, &mut mapped_binds);
                        inner_index += 1;
                    }
                    // TODO RECURSE ON BODY
                }
                Locals(locals) => {
                    let mut inner_index = 0;
                    let mut mapped_binds = locals.binds.iter().map(|(_, x)| *x).collect();
                    for (_, node) in &mut locals.binds {
                        self.apply_walk_callback(callback, *node, inner_index, &mut mapped_binds);
                        inner_index += 1;
                    }
                }
                Function { params, body, .. } => {
                    // let filtered_param_items = params
                    //     .iter_mut()
                    //     .filter_map(|x| match x {
                    //         FuncParam::Unwrapped { params, .. } => Some(params),
                    //         _ => None,
                    //     })
                    //     .flatten()
                    //     .collect_vec();
                    // for (_, def_value) in filtered_param_items {
                    //     if let Some(def_value_inner) = def_value {
                    //         callback(def_value_inner.as_mut(), Some(self));
                    //     }
                    // }
                    let mut inner_index = 0;
                    for node in body {
                        self.apply_walk_callback(callback, *node, inner_index, body);
                        inner_index += 1;
                    }
                }
                _ => (),
            };
        }
    }
}
