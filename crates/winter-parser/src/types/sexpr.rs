#[derive(Debug, PartialEq, Eq)]
pub struct Cons<'a, T> {
  pub value: &'a T,
  pub next:  Option<&'a Cons<'a, T>>,
}

pub type SXCons<'a> = Cons<'a, SXAtom<'a>>;

#[derive(Debug, PartialEq, Eq)]
pub enum SXAtom<'a> {
  NIL,
  BOOL(bool),
  SYMBOL(&'a str),
  TEXTUAL(SXTextualCons<'a>),
  NUMBER(&'a str),
}

pub type SXTextualCons<'a> = Cons<'a, SXTextualAtom<'a>>;

#[derive(Debug, PartialEq, Eq)]
pub enum SXTextualAtom<'a> {
  LITERAL(&'a str),
  EXPR(SXCons<'a>),
}
