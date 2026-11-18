use super::{LexemeResOpt, Lexer};

impl<'a, I: Iterator<Item = char> + 'a> Lexer<I> {
  pub(super) fn parse_textual(&mut self) -> LexemeResOpt<'a> {
    todo!()
  }
}
