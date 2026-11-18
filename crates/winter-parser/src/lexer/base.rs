use std::num::NonZero;
use std::ops::Coroutine;
use std::str::Chars;

use itertools::{Itertools, PeekNth, peek_nth};

use super::LexerError;
use crate::types::{Lexeme, Spanned, SpannedError};
use crate::utils::Cursor;
use crate::{ok_none, ok_some, span_err, span_this};

/// Determines which byte encoding to use for getting character lengths.
#[derive(Debug, Clone)]
pub enum LexerByteEncoding {
  UTF_8,
  UTF_16,
}

impl LexerByteEncoding {
  fn get_char_len(&self, char: &char) -> usize {
    match self {
      | Self::UTF_8 => char.len_utf8(),
      | Self::UTF_16 => char.len_utf16(),
    }
  }
}

#[derive(Debug, Clone)]
pub struct Lexer<I: Iterator<Item = char>> {
  pub(super) cur:           Cursor<char, I>,
  pub(super) byte_encoding: LexerByteEncoding,
}

fn make_chars<'input>(
  iter: impl Iterator<Item = &'input str> + 'input,
) -> PeekNth<impl Iterator<Item = char> + 'input> {
  peek_nth(iter.flat_map(str::chars))
}

pub fn mk_lexer_str_iter<'input>(
  iterator: impl Iterator<Item = &'input str> + 'input,
) -> Lexer<impl Iterator<Item = char> + 'input> {
  Lexer::new(make_chars(iterator))
}

pub type Lex = Spanned<Lexeme>;
pub type LexerRes<T> = Result<T, SpannedError<LexerError>>;
pub type LexerResOpt<T> = LexerRes<Option<T>>;
pub type LexemeRes = LexerRes<Lex>;
pub type LexemeResOpt = LexerResOpt<Lex>;

/// Checks if a returned `Option<T>` is `Some(T)` and, if so, returns early with
/// an `ok_some!()` call. This can be done repeatedly, for many functions, by
/// using more arguments to the macro.
macro_rules! try_handle {
  ($($x:expr),+) => {
    $(if let Some(tk) = $x {
      ok_some!(tk);
    })+
  };
}

impl<I: Iterator<Item = char>> Lexer<I> {
  pub fn new(iter: I) -> Self {
    Self {
      cur:           Cursor::new(iter),
      // hardcoded for now!
      // TODO: use this
      byte_encoding: LexerByteEncoding::UTF_8,
    }
  }

  pub fn parse(&mut self) -> impl Iterator<Item = LexemeResOpt> {
    std::iter::from_coroutine(
      #[coroutine]
      || {
        loop {
          let tk = self.parse_one();
          let should_stop = tk.as_ref().is_ok_and(|tk_opt| tk_opt.is_none())
            || tk.as_ref().is_err();
          yield tk;
          if should_stop {
            break;
          }
        }
      },
    )
  }

  pub(super) fn parse_one(&mut self) -> LexemeResOpt {
    try_handle!(
      self.parse_comment()?,
      self.parse_ellipsis()?,
      self.parse_single_char()?,
      self.parse_textual()?,
      self.parse_number()?,
      self.parse_atom()?
    );

    ok_none!();
  }

  fn parse_comment(&mut self) -> LexemeResOpt {
    if !self.peek()?.is_some_and(|x| x == ';') {
      ok_none!();
    };
    let _ = self.next()?;
    let pos_start = self.pos();
    let mut out = Vec::<char>::new();
    loop {
      let Some(ch) = self.next()? else { break };
      match ch {
        | '\n' => break,
        | _ => out.push(ch),
      }
    }

    ok_some!(span_this!(
      pos_start,
      self.pos(),
      Lexeme::VALUE_COMMENT(String::from_iter(out.iter()))
    ))
  }

  fn parse_ellipsis(&mut self) -> LexemeResOpt {
    if !self.starts_with("...") {
      ok_none!();
    }
    const THREE_IS_NOT_ZERO_TRUST_ME_BRO: NonZero<usize> =
      NonZero::new(3usize).unwrap();
    let x = self
      .cur
      .advance_by(THREE_IS_NOT_ZERO_TRUST_ME_BRO);
    ok_some!(span_this!(self.pos() - 3, self.pos(), Lexeme::ELLIPSIS));
  }

  fn parse_single_char(&mut self) -> LexemeResOpt {
    let ch = self.peek()?.expect("shouldn't be none");
    if let Some(lx) = Lexeme::from_char(ch) {
      ok_some!(span_this!(self.pos(), lx));
    }
    ok_none!();
  }

  pub(super) fn peek(&mut self) -> LexerResOpt<char> {
    map_err(self.pos(), self.cur.peek())
  }

  pub(super) fn next(&mut self) -> LexerResOpt<char> {
    map_err(self.pos(), self.cur.next())
  }

  pub(super) fn pos(&self) -> usize {
    self.cur.pos
  }

  pub(super) fn starts_with(&mut self, s: &str) -> bool {
    s.chars()
      .enumerate()
      .all(|(i, c)| self.cur.peek_nth(i).is_ok_and(|x| x == Some(c)))
  }
}

pub(super) fn map_err<T, Err: std::error::Error + 'static>(
  pos: usize,
  res: Result<T, Err>,
) -> LexerRes<T> {
  match res {
    | Ok(x) => Ok(x),
    | Err(err) => span_err!(pos, LexerError::other(err)),
  }
}
