use crate::types::{Lexeme, LexemeKind, WantedCandidates};

#[derive(thiserror::Error, Debug)]
pub enum ParserError {
  #[error("Unexpected end of file")]
  UNEX_EOF,

  #[error("Unexpected token! Got '{got}', wanted {wanted}")]
  UNEX_TOKEN { got: LexemeKind, wanted: WantedCandidates<LexemeKind> },
}
