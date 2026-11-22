use itertools::Itertools;
use piping::pipe;

use super::{LexemeResOpt, Lexer};
use crate::types::Lexeme;
use crate::{ok_none, ok_some, span_this};

impl<'a, I: Iterator<Item = char> + 'a> Lexer<'a, I> {
  pub(super) fn parse_atom(&mut self) -> LexemeResOpt<'a> {
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
      pipe! {
        parts.iter().join("")
        |> self.alloc.arena.alloc(__)
        |> Lexeme::VALUE_ATOM(__)
        |> ok_some!(span_this!(pos_start, self.pos(), __))
      }
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
