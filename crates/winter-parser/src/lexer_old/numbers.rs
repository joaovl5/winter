use itertools::Itertools;
use piping::pipe;

use super::{LexemeResOpt, Lexer, LexerRes};
use crate::lexer::LexerError;
use crate::types::Lexeme;
use crate::{ok, ok_none, ok_some, span_err, span_this};

impl<'a, I: Iterator<Item = char> + 'a> Lexer<'a, I> {
  pub(super) fn parse_number(&mut self) -> LexemeResOpt<'a> {
    self.cur.pos_checkpoint();
    let pos_start = self.pos();
    let mut parts = Vec::<char>::new();

    let Some(ch) = self.peek()? else { ok_none!() };
    if ch == '-' {
      self.next()?;
      parts.push(ch);
    }
    parts.extend({
      let digit_parts = self.traverse_digits()?;
      if digit_parts.is_empty() {
        super::map_err(self.pos(), self.cur.pos_rollback())?;
        ok_none!();
      }
      digit_parts
    });
    // we already commit here before checking further because, if we actually
    // have an invalid state going forward from here, we consider that an error,
    // and not a "None" case, and as such we won't rollback.
    super::map_err(self.pos(), self.cur.pos_commit())?;
    let Some(ch_2) = self.peek()? else {
      pipe! {
        parts.iter().join("")
        |> self.alloc.arena.alloc(__)
        |> Lexeme::VALUE_NUMBER(__)
        |> ok_some!(span_this!(pos_start, self.pos(), __))
      }
    };

    if ch_2 == '.' {
      self.next()?;
      parts.push(ch_2);
      parts.extend({
        let digit_parts = self.traverse_digits()?;
        if digit_parts.is_empty() {
          let err = LexerError::UNT_NUM_DOT(parts.iter().join(""));
          span_err!(pos_start, self.pos(), err);
        };
        digit_parts
      });
    }

    pipe! {
      parts.iter().join("")
      |> self.alloc.arena.alloc(__)
      |> Lexeme::VALUE_NUMBER(__)
      |> ok_some!(span_this!(pos_start, self.pos(), __))
    }
  }

  fn traverse_digits(&mut self) -> LexerRes<Vec<char>> {
    let mut out = Vec::<char>::new();
    loop {
      let Some(ch) = self.peek()? else { break };
      if !ch.is_ascii_digit() {
        break;
      };
      out.push(ch);
      self.next()?;
    }
    ok!(out)
  }
}
