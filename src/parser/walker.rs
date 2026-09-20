use crate::parser::{AstVal, BoundNode, CallableHeadNodeType, MapVal, Module, StringLikeASTPart};

#[derive(Debug)]
pub enum WalkArgs<'src, 'walk> {
    Ctx {
        /// Index of the `value` node within `nodes`
        idx: usize,
        nodes: &'walk mut [AstVal<'src>],
    },
    Val(&'walk mut AstVal<'src>),
}

type WalkDeepCb<'ast> = dyn for<'call, 'walk> FnMut(&'call mut WalkArgs<'ast, 'walk>);

fn walk_deep_many<'src>(many: &mut [AstVal<'src>], callback: &mut WalkDeepCb<'src>) {
    let mut idx = 0;
    while idx < many.len() {
        walk_deep(&mut WalkArgs::Ctx { idx, nodes: many }, callback);
        idx += 1;
    }
}

fn walk_deep_bounds<'src, 'walk>(
    bounds: &'walk mut [BoundNode<'src>],
    callback: &mut WalkDeepCb<'src>,
) {
    let mut idx = 0;
    while idx < bounds.len() {
        walk_deep(&mut WalkArgs::Val(&mut bounds[idx].1), callback);
        idx += 1;
    }
}

fn walk_deep<'src, 'walk>(args: &'walk mut WalkArgs<'src, 'walk>, callback: &mut WalkDeepCb<'src>) {
    callback(args);
    let node = match args {
        WalkArgs::Ctx { idx, nodes } => &mut nodes[*idx],
        WalkArgs::Val(val) => val,
    };

    use AstVal::*;
    match node {
        List(nodes) => walk_deep_many(nodes.as_mut_slice(), callback),
        StringLike { parts, .. } => {
            let mut idx = 0;
            while idx < parts.len() {
                let part = &mut parts[idx];
                let StringLikeASTPart::Expr(part_expr) = part else {
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
                walk_deep_many(path, callback);
                walk_deep(&mut WalkArgs::Val(path_binding), callback);
                idx += 1;
            }
            todo!()
        }
        Callable(head, body) => {
            if let CallableHeadNodeType::Expr(expr) = head {
                walk_deep(&mut WalkArgs::Val(expr), callback)
            }
            walk_deep_many(body, callback);
        }
        Selection(nodes) => walk_deep_many(nodes, callback),
        Let { locals, body } => {
            walk_deep_bounds(locals.binds.as_mut_slice(), callback);
            walk_deep(&mut WalkArgs::Val(body), callback);
        }
        Locals(locals) => walk_deep_bounds(locals.binds.as_mut_slice(), callback),
        _ => todo!(),
    }
    todo!()
}

impl<'ast> AstVal<'ast> {
    pub fn walk_deep(&mut self, callback: &mut WalkDeepCb<'ast>) {
        walk_deep(&mut WalkArgs::Val(self), callback);
    }
}

impl<'ast> Module<'ast> {
    pub fn walk_deep(&mut self, callback: &mut WalkDeepCb<'ast>) {
        let nodes_slice: &mut [AstVal] = self.nodes.iter_mut().map(|x| x.value).collect();

        walk_deep(
            &mut WalkArgs::Ctx {
                idx: 0,
                nodes: nodes_slice,
            },
            callback,
        );
    }
}
