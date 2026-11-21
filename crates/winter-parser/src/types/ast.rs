use bitbybit::bitfield;
use derive_more::{From, Into};
use kinded::Kinded;
use strum_macros::{Display, EnumString};

// -- Main AST Type --

#[derive(Kinded, Debug, PartialEq, Eq)]
pub enum Node<'a> {
  NIL,
  BOOL(bool),
  NUMBER(&'a str),
  IDENTIFIER(&'a str),
  TEXTUAL {
    parts:   &'a [NodeTextualPart<'a>],
    is_path: bool,
  },
  LIST(&'a [Node<'a>]),
  SELECTION(&'a [Node<'a>]),
  MAP {
    inherits: &'a [InheritBind<'a>],
    binds:    &'a [MapBind<'a>],
  },
  INHERITS {
    inherits: &'a [InheritBind<'a>],
  },
  LOCALS {
    inherits: &'a [InheritBind<'a>],
    binds:    &'a [RegularBind<'a>],
  },
  LET {
    inherits: &'a [InheritBind<'a>],
    binds:    &'a [RegularBind<'a>],
    body:     &'a Node<'a>,
  },
  FUNCTION {
    name:      Option<TypedReference<'a>>,
    params:    &'a [FunctionParam<'a>],
    body:      &'a [Node<'a>],
    type_args: Option<&'a [TypeAnnotation<'a>]>,
  },
  CALLABLE {
    head: CallableHead<'a>,
    tail: Vec<Node<'a>>,
  },
  TYPE_DECLARATION(DeclaredType<'a>),
}

// -- Backing types for AST --

#[derive(Debug, PartialEq, Eq)]
pub enum NodeTextualPart<'a> {
  TEXT(&'a str),
  EXPR(&'a Node<'a>),
}

#[derive(Debug, PartialEq, Eq)]
pub enum FunctionParam<'a> {
  PLAIN(TypedReference<'a>),
  UNWRAPPED {
    name:        Option<TypedReference<'a>>,
    params:      &'a [UnwrappedParam<'a>],
    allow_extra: bool,
  },
}

#[derive(Debug, PartialEq, Eq)]
pub enum CallableHead<'a> {
  OPERATOR(OperatorKind),
  EXPR(&'a Node<'a>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct TypeArgument<'a> {
  name:    &'a str,
  default: Option<TypeAnnotation<'a>>,
  bound:   Option<TypeAnnotation<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum DeclaredTypeKind<'a> {
  /// Creates a NewType from a type construction.
  NEW_TYPE(TypeAnnotation<'a>),
  ATTRIBUTES(&'a [TypedReference<'a>]),
}

#[derive(Debug, PartialEq, Eq)]
pub struct DeclaredType<'a> {
  kind:      DeclaredTypeKind<'a>,
  type_args: Option<Vec<TypeAnnotation<'a>>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TypeAnnotation<'a> {
  ANY,
  NIL,
  STRING,
  PATH,
  NUMBER,
  /// Refer to a type argument.
  TYPE_ARG_NAME(&'a str),
  /// Refer to a type.
  TYPE_NAME {
    name:            &'a str,
    type_arg_values: Option<&'a [TypeAnnotation<'a>]>,
  },
  LIST(&'a [TypeAnnotation<'a>]),
  UNION(&'a [TypeAnnotation<'a>]),
  /// Combines two types `A` and `B`. This requires both types to be compound
  /// types. If types do not have an intersection, this should fail.
  ///
  /// - list intersections are equivalent to "narrowing down" the type, e.g.:
  ///   `list<Any> & list<str>` is `list<str>`
  /// - map-based intersections join fields from both types, failing if there is
  ///   no reconciliation available
  INTERSECTION(&'a [TypeAnnotation<'a>]),
  MAP {
    key_type:   &'a TypeAnnotation<'a>,
    value_type: &'a TypeAnnotation<'a>,
  },
  FUNCTION {
    args_type: &'a [TypeAnnotation<'a>],
    out_type:  &'a TypeAnnotation<'a>,
  },
}

#[derive(Debug, PartialEq, Eq, EnumString, Display)]
pub enum OperatorKind {
  #[strum(serialize = "+")]
  ADD,
  #[strum(serialize = "-")]
  SUB,
  #[strum(serialize = "*")]
  MUL,
  #[strum(serialize = "/")]
  DIV,
  #[strum(serialize = ">")]
  GT,
  #[strum(serialize = "<")]
  LT,
  #[strum(serialize = ">=")]
  GTE,
  #[strum(serialize = "<=")]
  LTE,
  #[strum(serialize = "=")]
  EQ,
  #[strum(serialize = "not=")]
  NEQ,
  #[strum(serialize = "not")]
  NOT,
  #[strum(serialize = "all")]
  ALL,
  #[strum(serialize = "any")]
  ANY,
  #[strum(serialize = "thus")]
  THUS,
  #[strum(serialize = ".")]
  SEL,
  #[strum(serialize = "or")]
  OR,
  #[strum(serialize = "has")]
  HAS,
  #[strum(serialize = "++")]
  CONC,
  #[strum(serialize = "//")]
  UPDT,
  #[strum(serialize = "<|")]
  LPIPE,
  #[strum(serialize = "|>")]
  RPIPE,
}

// -- Newtypes --

#[derive(Debug, PartialEq, Eq, From, Into)]
pub struct UnwrappedParam<'a>(pub TypedReference<'a>, pub Option<Node<'a>>);

#[derive(Debug, PartialEq, Eq, From, Into)]
pub struct RegularBind<'a>(pub TypedReference<'a>, pub Node<'a>);

#[derive(Debug, PartialEq, Eq, From, Into)]
pub struct MapBind<'a>(pub Vec<Node<'a>>, pub Node<'a>);

#[derive(Debug, PartialEq, Eq, From, Into)]
pub struct InheritBind<'a>(pub &'a str, pub Option<Vec<TypedReference<'a>>>);

#[derive(Debug, PartialEq, Eq, From, Into)]
pub struct TypedReference<'a>(pub &'a str, pub Option<TypeAnnotation<'a>>);
