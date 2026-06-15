use itertools::Itertools;

pub type _Node<'x> = AstVal<'x>;
pub type OneNode<'x> = Box<_Node<'x>>;
pub type ManyNodes<'x> = Vec<_Node<'x>>;
pub type BoundNode<'x> = (TypedReference<'x>, _Node<'x>);
pub type BoundNodes<'x> = Vec<BoundNode<'x>>;
pub type InheritNode<'x> = (&'x str, Option<Vec<TypedReference<'x>>>);
pub type InheritNodes<'x> = Vec<InheritNode<'x>>;

pub type _Type<'x> = TypeAnnotation<'x>;
pub type OneType<'x> = Box<_Type<'x>>;
pub type TwoType<'x> = Box<(_Type<'x>, _Type<'x>)>;
pub type ManyTypes<'x> = Vec<_Type<'x>>;

pub type _Param<'x> = FuncParam<'x>;
pub type OneParam<'x> = Box<_Param<'x>>;
pub type ManyParams<'x> = Vec<_Param<'x>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ast<'ast> {
    pub value: AstVal<'ast>,
    pub pos_start: usize,
    pub pos_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstVal<'src> {
    Nil,
    Bool(bool),
    Number(&'src str),
    StringLike {
        parts: Vec<StringLikeASTPart<'src>>,
        is_path: bool,
    },
    Identifier(&'src str),
    List(ManyNodes<'src>),
    Map(MapVal<'src>), // nested `ManyNode` for dot paths
    Callable(CallableHeadNodeType<'src>, ManyNodes<'src>),
    Selection(ManyNodes<'src>), // `<a>.<b>.<c>.(...)`
    // binds
    Let {
        locals: LocalsLike<'src>,
        body: OneNode<'src>,
    },
    Locals(LocalsLike<'src>),
    Inherit(InheritNodes<'src>),
    // funcs
    /// direct nix counterpart to lambdas
    // Lambda(OneParam<'src>, OneNode<'src>), TODO: do later
    /// abstraction over lambdas
    Function {
        name: Option<TypedReference<'src>>,
        params: ManyParams<'src>,
        body: ManyNodes<'src>,
    },

    // -- stuff below this line is not done --
    DeclaredType(DeclaredType<'src>),
    DeclaredEnum(DeclaredEnum<'src>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FuncParam<'src> {
    Plain(TypedReference<'src>),
    /// unwrapped map/attrset
    Unwrapped {
        /// name for arg, if it is named
        name: Option<&'src str>,
        /// list of unpacked parameters that
        /// - have names and optional types (TypedReference)
        /// - have optional default values (OneNode)
        params: Vec<(TypedReference<'src>, Option<OneNode<'src>>)>,
        /// When ellipsis is detected
        allow_extra: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapVal<'src> {
    pub binds: Vec<(ManyNodes<'src>, AstVal<'src>)>,
    pub inherits: InheritNodes<'src>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalsLike<'src> {
    pub binds: BoundNodes<'src>,
    pub inherits: InheritNodes<'src>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypedReference<'src> {
    pub name: &'src str,
    pub type_val: Option<TypeAnnotation<'src>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeAnnotation<'src> {
    Any,
    Nil,
    String,
    Path,
    Number,
    TypeName(&'src str), // custom type
    List(OneType<'src>),
    Map(TwoType<'src>),
    Fn(ManyTypes<'src>, OneType<'src>),
    Union(ManyTypes<'src>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredEnum<'src> {
    pub name: &'src str,
    pub type_val: OneType<'src>,
    pub items: Vec<(&'src str, OneNode<'src>)>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredType<'src> {
    pub name: &'src str,
    pub value: DeclaredTypeVal<'src>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DeclaredTypeVal<'src> {
    Intersection(ManyTypes<'src>),
    Attributes(Vec<DeclaredTypeAttr<'src>>),
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DeclaredTypeAttr<'src> {
    pub name: &'src str,
    pub type_val: OneType<'src>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringLikeASTPart<'src> {
    Text(String),
    Expr(OneNode<'src>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallableHeadNodeType<'src> {
    Op(OperatorType),
    Expr(OneNode<'src>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatorType {
    // number ops
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Gt,  // >
    Lt,  // <
    Gte, // >=
    Lte, // <=
    // boolean ops
    Eq,   // =
    Neq,  // not=
    Not,  // not
    All,  // all
    Any,  // any
    Thus, // thus
    // attrs selectors
    Sel, // .
    Or,  // or
    Has, // has
    // list ops
    Conc, // ++
    // map ops
    Updt, // //
    // fn ops
    PipeL, // <|
    PipeR, // |>
}

impl<'src> AstVal<'src> {
    /// Walk through AST with a callback that is called at each node or sub-node
    ///
    /// * `callback`: Function taking a mutable AstVal, and an optional `context` of
    ///   sibling nodes.
    pub fn walk(
        &mut self,
        // callback for consuming walked nodes
        mut callback: impl FnMut(&mut AstVal<'src>),
    ) {
        use AstVal::*;
        callback(self);
        match self {
            StringLike { parts, .. } => {
                let expr_parts: Vec<&mut AstVal<'src>> = parts
                    .iter_mut()
                    .filter_map(|part| match part {
                        StringLikeASTPart::Expr(expr) => Some(expr.as_mut()),
                        StringLikeASTPart::Text(_) => None,
                    })
                    .collect();
                for expr_p in expr_parts {
                    callback(expr_p);
                }
            }
            List(nodes) => {
                for node in nodes {
                    callback(node);
                }
            }
            Map(map_val) => {
                for (key_parts, value) in &mut map_val.binds {
                    for key_part in key_parts {
                        callback(key_part);
                    }
                    callback(value);
                }
            }
            Callable(head, nodes) => {
                if let CallableHeadNodeType::Expr(head_expr) = head {
                    callback(head_expr);
                }
                for node in nodes {
                    callback(node);
                }
            }
            Selection(nodes) => {
                for node in nodes {
                    callback(node);
                }
            }
            Let { locals, body } => {
                for bind in &mut locals.binds {
                    let (_, bind_val) = bind;
                    callback(bind_val);
                }
                callback(body);
            }
            Locals(locals) => {
                for bind in &mut locals.binds {
                    let (_, bind_val) = bind;
                    callback(bind_val);
                }
            }
            Function { params, body, .. } => {
                let filtered_param_items = params
                    .iter_mut()
                    .filter_map(|x| match x {
                        FuncParam::Unwrapped { params, .. } => Some(params),
                        _ => None,
                    })
                    .flatten()
                    .collect_vec();
                for (_, def_value) in filtered_param_items {
                    if let Some(def_value_inner) = def_value {
                        callback(def_value_inner.as_mut());
                    }
                }
                for node in body {
                    callback(node);
                }
            }
            _ => (),
        };
    }
}
