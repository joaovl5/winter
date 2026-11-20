use itertools::Itertools;

use super::{LexemeResOpt, Lexer, LexerRes};
use crate::lexer::LexerError;
use crate::types::Lexeme;
use crate::{ok, ok_none, ok_some, span_err, span_this};

impl<I: Iterator<Item = char>> Lexer<I> {
  pub(super) fn parse_number(&mut self) -> LexemeResOpt {
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
      let lex = Lexeme::VALUE_NUMBER(parts.iter().join(""));
      ok_some!(span_this!(pos_start, self.pos(), lex));
    };
    if ch_2 != '.' {
      let err = LexerError::UNEX_NUM_CHAR(ch_2, parts.iter().join(""));
      span_err!(pos_start, self.pos(), err);
    }
    parts.extend({
      let digit_parts = self.traverse_digits()?;
      if digit_parts.is_empty() {
        let err = LexerError::UNT_NUM_DOT(parts.iter().join(""));
        span_err!(pos_start, self.pos(), err);
      };
      digit_parts
    });

    let lex = Lexeme::VALUE_NUMBER(parts.iter().join(""));
    ok_some!(span_this!(pos_start, self.pos(), lex));
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
