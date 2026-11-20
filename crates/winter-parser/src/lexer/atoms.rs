use itertools::Itertools;

use super::{LexemeResOpt, Lexer};
use crate::types::Lexeme;
use crate::{ok_none, ok_some, span_this};

impl<I: Iterator<Item = char>> Lexer<I> {
  pub(super) fn parse_atom(&mut self) -> LexemeResOpt {
    let pos_start = self.pos();
    let mut parts = Vec::<char>::new();
    while let Some(ch) = self.peek()? {
      if !is_atom_char(ch) {
        break;
      };
      parts.push(ch);
      self.next()?;
    }
    if parts.is_empty() {
      ok_none!();
    } else {
      let lex = Lexeme::VALUE_ATOM(parts.iter().join(""));
      ok_some!(span_this!(pos_start, self.pos(), lex));
    }
  }
}

fn is_atom_char(ch: char) -> bool {
  ch.is_ascii()
    && !ch.is_whitespace()
    && ch != ';'
    && ch != '"'
    && Lexeme::from_char(ch).is_none()
}
