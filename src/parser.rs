use std::{iter::Peekable, slice::Iter};

use crate::tokenizer::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AstVal<'ast> {
    Null,
    Bool(bool),
    Number(&'ast str),
    StringLike {
        parts: Vec<StringLikePart<'ast>>,
        is_path: bool,
    },
    Identifier(&'ast str),
    List(ManyNodes<'ast>),
    Map(Vec<TwoNode<'ast>>),
    Callable(CallableNodeVal<'ast>),
}

type OneNode<'ast> = Box<AstVal<'ast>>;
type TwoNode<'ast> = Box<(Box<AstVal<'ast>>, Box<AstVal<'ast>>)>;
type ManyNodes<'ast> = Vec<AstVal<'ast>>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringLikePart<'ast> {
    Text(String),
    Expr(OneNode<'ast>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OperatorNodeVal<'ast> {
    // number ops
    Add(ManyNodes<'ast>), // +
    Sub(TwoNode<'ast>),   // -
    Neg(OneNode<'ast>),   // - with one arg
    Mul(ManyNodes<'ast>), // *
    Div(TwoNode<'ast>),   // /
    Gt(TwoNode<'ast>),    // >
    Lt(TwoNode<'ast>),    // <
    Gte(TwoNode<'ast>),   // >=
    Lte(TwoNode<'ast>),   // <=
    // boolean ops
    Eq(TwoNode<'ast>),     // =
    Neq(TwoNode<'ast>),    // not=
    Not(OneNode<'ast>),    // not
    All(ManyNodes<'ast>),  // all
    Any(ManyNodes<'ast>),  // any
    Thus(ManyNodes<'ast>), // thus
    // attrs selectors
    Sel(TwoNode<'ast>), // .
    Or(TwoNode<'ast>),  // or
    Has(TwoNode<'ast>), // ?
    // list ops
    Conc(ManyNodes<'ast>), // ++
    // map ops
    Updt(ManyNodes<'ast>), // //
    // fn ops
    PipeL(ManyNodes<'ast>), // <|
    PipeR(ManyNodes<'ast>), // |>
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallableNodeVal<'ast> {
    /// translates to nix operator
    Op(OperatorNodeVal<'ast>),
    /// anything else that may be callable
    Expr(OneNode<'ast>, ManyNodes<'ast>),
}

pub struct Parser<'ast> {
    tokens: &'ast Vec<Token<'ast>>,
    pos: usize,
    iter: Peekable<Iter<'ast, Token<'ast>>>,
}

impl<'ast> Parser<'ast> {
    pub fn new(tokens: &'ast Vec<Token<'ast>>) -> Self {
        Self {
            pos: 0,
            iter: tokens.iter().peekable(),
            tokens,
        }
    }

    pub fn parse(mut self) -> Result<Vec<AstVal>, String> {
        todo!()
    }
}

#[derive(Debug, Clone)]
pub struct ParserError {
    msg: String,
    // token: Token,
    pos: usize,
}
