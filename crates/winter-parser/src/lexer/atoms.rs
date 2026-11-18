use super::{LexemeResOpt, Lexer};

impl<'a, I: Iterator<Item = char> + 'a> Lexer<I> {
  pub(super) fn parse_atom(&mut self) -> LexemeResOpt<'a> {
    todo!()
  }
}
