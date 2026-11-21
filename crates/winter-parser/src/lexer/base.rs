use std::iter::FromFn;
use std::num::NonZero;
use std::ops::Coroutine;
use std::str::Chars;

use bumpalo::Bump;
use itertools::{Itertools, PeekNth, peek_nth};
use piping::pipe;
use tracing::debug;

use super::LexerError;
use crate::types::{
  Lexeme, Spanned, SpannedError, WhitespaceKind, WinterAlloc,
};
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

#[derive(Debug)]
pub struct Lexer<'a, I: Iterator<Item = char> + 'a> {
  pub(super) alloc:         &'a WinterAlloc,
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
  alloc: &'input WinterAlloc,
) -> Lexer<'input, impl Iterator<Item = char> + 'input> {
  Lexer::new(make_chars(iterator), alloc)
}

pub type Lex<'a> = Spanned<Lexeme<'a>>;
pub type LexerRes<T> = Result<T, SpannedError<LexerError>>;
pub type LexerResOpt<T> = LexerRes<Option<T>>;
pub type LexemeRes<'a> = LexerRes<Lex<'a>>;
pub type LexemeResOpt<'a> = LexerResOpt<Lex<'a>>;

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

impl<'a, I: Iterator<Item = char> + 'a> Lexer<'a, I> {
  pub fn new(iter: I, alloc: &'a WinterAlloc) -> Self {
    Self {
      alloc,
      cur: Cursor::new(iter),
      // hardcoded for now!
      // TODO: use this
      byte_encoding: LexerByteEncoding::UTF_8,
    }
  }

  pub fn parse(mut self) -> FromFn<impl FnMut() -> Option<LexemeRes<'a>>> {
    std::iter::from_fn(move || {
      let tk_opt = self.parse_one();
      match tk_opt {
        | Ok(None) => None,
        | Ok(Some(tk)) => Some(Ok(tk)),
        | Err(err) => Some(Err(err)),
      }
    })
  }

  pub(super) fn parse_one(&mut self) -> LexemeResOpt<'a> {
    try_handle!(
      self.parse_ignored()?,
      self.parse_ellipsis()?,
      self.parse_single_char()?,
      self.parse_textual()?,
      self.parse_number()?,
      self.parse_atom()?
    );

    debug!("No more lexemes found, finalized!");
    ok_none!();
  }

  // parses hidden/ignored stuff like comments and whitespace
  fn parse_ignored(&mut self) -> LexemeResOpt<'a> {
    let Some(ch) = self.peek()? else { ok_none!() };
    let pos_start = self.pos();
    if ch == ';' {
      self.next()?;
      let mut out = Vec::<char>::new();
      loop {
        let Some(ch) = self.next()? else { break };
        match ch {
          | '\n' => break,
          | _ => out.push(ch),
        }
      }
      pipe! {
        out.iter()
        |> String::from_iter(__)
        |> self.alloc.arena.alloc(__)
        |> Lexeme::VALUE_COMMENT(__)
        |> ok_some!(span_this!(pos_start, self.pos(), __))
      }
    } else if ch.is_ascii_whitespace() {
      self.next()?;
      // NOTE: we only accept tabs or spaces, and whatever other insanities that
      // ascii supports as whitespace are handled as spaces. we can add
      // an option to hard-error instead of this silent behavior later
      let kind = WhitespaceKind::from_char(ch).unwrap_or(WhitespaceKind::SPACE);
      let mut count: NonZero<u8> = unsafe { NonZero::new_unchecked(1) };
      loop {
        let Some(ch_2) = self.peek()? else { break };
        if !ch_2.is_ascii_whitespace() {
          break;
        };
        let ch_2_kind =
          WhitespaceKind::from_char(ch_2).unwrap_or(WhitespaceKind::SPACE);
        if ch_2_kind == kind
          && let Some(new_count) = count.checked_add(1)
        {
          self.next()?;
          count = new_count;
        } else {
          debug!("Breaking whitespace lexeme bc. of u8 overflow");
          break;
        }
      }
      ok_some!(span_this!(
        pos_start,
        self.pos(),
        Lexeme::VALUE_WHITESPACE { kind, count }
      ))
    } else {
      ok_none!();
    };
  }

  fn parse_ellipsis(&mut self) -> LexemeResOpt<'a> {
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

  pub(super) fn parse_single_char(&mut self) -> LexemeResOpt<'a> {
    let Some(ch) = self.peek()? else { ok_none!() };
    if let Some(lx) = Lexeme::from_char(ch) {
      let pos = self.pos();
      self.next()?;
      ok_some!(span_this!(pos, lx));
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

pub(super) fn map_err<T, Err: std::error::Error + Send + Sync + 'static>(
  pos: usize,
  res: Result<T, Err>,
) -> LexerRes<T> {
  match res {
    | Ok(x) => Ok(x),
    | Err(err) => span_err!(pos, LexerError::other(err)),
  }
}
