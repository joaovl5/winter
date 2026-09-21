use crate::parser::{
    Ast, AstVal, BoundNode, CallableHeadNodeType, MapVal, Module, StringLikeASTPart,
};

#[derive(Debug)]
pub enum WalkArgs<'src, 'walk> {
    // i dont like this brand of separation between astval and ast, must refactor later
    Module {
        idx: usize,
        nodes: &'walk mut Vec<Ast<'src>>,
    },
    Ctx {
        idx: usize,
        nodes: &'walk mut Vec<AstVal<'src>>,
        // we're currently cloning this, we can optimize later
        parent: &'walk AstVal<'src>,
    },
    Val(&'walk mut AstVal<'src>),
}

type WalkDeepCb<'ast, 'cb> = dyn for<'call, 'walk> FnMut(&'call mut WalkArgs<'ast, 'walk>) + 'cb;

fn walk_deep_many<'ast>(
    many: &mut Vec<AstVal<'ast>>,
    callback: &mut WalkDeepCb<'ast, '_>,
    parent: &AstVal<'ast>,
) {
    let mut idx = 0;
    while idx < many.len() {
        let removed = walk_deep(
            &mut WalkArgs::Ctx {
                idx,
                nodes: many,
                parent,
            },
            callback,
        );
        if !removed {
            idx += 1;
        }
    }
}

fn walk_deep_bounds<'ast, 'walk>(
    bounds: &'walk mut [BoundNode<'ast>],
    callback: &mut WalkDeepCb<'ast, '_>,
) {
    let mut idx = 0;
    while idx < bounds.len() {
        walk_deep(&mut WalkArgs::Val(&mut bounds[idx].1), callback);
        idx += 1;
    }
}

fn walk_deep<'src, 'walk>(
    args: &'walk mut WalkArgs<'src, 'walk>,
    callback: &mut WalkDeepCb<'src, '_>,
) -> bool {
    let len = match args {
        WalkArgs::Module { nodes, .. } => nodes.len(),
        WalkArgs::Ctx { nodes, .. } => nodes.len(),
        WalkArgs::Val(_) => 0,
    };
    callback(args);
    // A callback can remove the current declaration and later declarations.
    // Visit the shifted node at this index on the next iteration.
    let removed = match args {
        WalkArgs::Module { nodes, .. } => nodes.len() < len,
        WalkArgs::Ctx { nodes, .. } => nodes.len() < len,
        WalkArgs::Val(_) => false,
    };
    if removed {
        return true;
    }
    let node = match args {
        WalkArgs::Module { idx, nodes } => &mut nodes[*idx].value,
        WalkArgs::Ctx { idx, nodes, .. } => &mut nodes[*idx],
        WalkArgs::Val(val) => val,
    };

    let node_clone = node.clone();

    use AstVal::*;
    match node {
        List(nodes) => walk_deep_many(nodes, callback, &node_clone),
        StringLike { parts, .. } => {
            let mut idx = 0;
            while idx < parts.len() {
                let part = &mut parts[idx];
                let StringLikeASTPart::Expr(part_expr) = part else {
                    idx += 1;
                    continue;
                };
                walk_deep(&mut WalkArgs::Val(part_expr), callback);
                idx += 1;
            }
        }
        Map(MapVal { binds, .. }) => {
            let mut idx = 0;
            while idx < binds.len() {
                let (path, path_binding) = &mut binds[idx];
                walk_deep_many(path, callback, &node_clone);
                walk_deep(&mut WalkArgs::Val(path_binding), callback);
                idx += 1;
            }
        }
        Callable(head, body) => {
            if let CallableHeadNodeType::Expr(expr) = head {
                walk_deep(&mut WalkArgs::Val(expr), callback);
            }
            walk_deep_many(body, callback, &node_clone);
        }
        Selection(nodes) => walk_deep_many(nodes, callback, &node_clone),
        Let { locals, body } => {
            walk_deep_bounds(locals.binds.as_mut_slice(), callback);
            walk_deep(&mut WalkArgs::Val(body), callback);
        }
        Locals(locals) => walk_deep_bounds(locals.binds.as_mut_slice(), callback),
        Function { params, body, .. } => {
            for param in params {
                if let crate::parser::FuncParam::Unwrapped { params, .. } = param {
                    for (_, default) in params {
                        if let Some(default) = default {
                            walk_deep(&mut WalkArgs::Val(default), callback);
                        }
                    }
                }
            }
            walk_deep_many(body, callback, &node_clone);
        }
        _ => (),
    }
    false
}

impl<'ast> AstVal<'ast> {
    pub fn walk_deep(&mut self, callback: &mut WalkDeepCb<'ast, '_>) {
        walk_deep(&mut WalkArgs::Val(self), callback);
    }
}

impl<'ast> Module<'ast> {
    pub fn walk_deep(&mut self, callback: &mut WalkDeepCb<'ast, '_>) {
        let mut idx = 0;
        while idx < self.nodes.len() {
            let removed = walk_deep(
                &mut WalkArgs::Module {
                    idx,
                    nodes: &mut self.nodes,
                },
                callback,
            );
            if !removed {
                idx += 1;
            }
        }
    }
}
