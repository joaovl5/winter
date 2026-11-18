use std::fmt::Display;

use itertools::Itertools;
use thiserror::Error;

use crate::types::WantedCandidates;

#[derive(Error, Debug)]
pub enum LexerError {
  #[error("Unexpected end of file!")]
  UNEXPECTED_EOF,

  #[error("Unexpected char! Got '{got}', wanted {wanted}.")]
  UNEXPECTED_CHAR { got: char, wanted: WantedCandidates<char> },

  #[error(transparent)]
  OTHER(#[from] Box<dyn std::error::Error>),
}

impl LexerError {
  pub fn other<Err: std::error::Error + 'static>(err: Err) -> Self {
    Self::OTHER(Box::new(err))
  }
}
