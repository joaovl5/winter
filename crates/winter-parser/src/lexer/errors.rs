use std::fmt::Display;

use itertools::Itertools;
use thiserror::Error;

use crate::types::WantedCandidates;

#[derive(Error, Debug)]
pub enum LexerError {
  #[error("Unexpected end of file")]
  UNEX_EOF,

  #[error("Unexpected char! Got '{got}', wanted {wanted}")]
  UNEX_CHAR { got: char, wanted: WantedCandidates<char> },

  #[error("Unterminated string/path")]
  UNT_STR,

  #[error("Unterminated backslash")]
  UNT_STR_BS,

  #[error("Unexpected escape sequence: \\{0}")]
  UNT_STR_BS_SEQ(char),

  #[error("Unterminated interpolation")]
  UNT_STR_INTERP,

  #[error("Unterminated '}}' in interpolated string/path")]
  UNEX_STR_INTERP_CLOSE,

  #[error(transparent)]
  OTHER(#[from] Box<dyn std::error::Error>),
}

impl LexerError {
  pub fn other<Err: std::error::Error + 'static>(err: Err) -> Self {
    Self::OTHER(Box::new(err))
  }
}
