use std::num::NonZero;

use kinded::Kinded;

use super::Spanned;

#[derive(Kinded, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lexeme<'a> {
  START_PAREN,          // (
  START_SQUARE_BRACKET, // [
  START_CURLY_BRACE,    // {
  CLOSE_PAREN,          // )
  CLOSE_SQUARE_BRACKET, // ]
  CLOSE_CURLY_BRACE,    // }
  // keep-sorted start
  BACKTICK,         // `
  COMMA,            // ,
  DOT,              // .
  ELLIPSIS,         // ...
  SIGN_AT,          // @
  SIGN_EXCLAMATION, // !
  SIGN_HASHTAG,     // #
  SIGN_QUESTION,    // ?
  // keep-sorted end
  VALUE_ATOM(&'a str),
  VALUE_NUMBER(&'a str),
  VALUE_TEXTUAL { parts: &'a [LexemeTextualPart<'a>], is_path: bool },
  VALUE_COMMENT(&'a str),
  VALUE_WHITESPACE { kind: WhitespaceKind, count: NonZero<u8> },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WhitespaceKind {
  TAB,
  SPACE,
  BREAK,
}

impl WhitespaceKind {
  pub fn from_char(ch: char) -> Option<Self> {
    match ch {
      | '\t' => Some(Self::TAB),
      | ' ' => Some(Self::SPACE),
      | '\n' => Some(Self::BREAK),
      | _ => None,
    }
  }

  pub fn as_str(&self) -> &'static str {
    match *self {
      | Self::TAB => "\\t",
      | Self::BREAK => "\\n",
      | Self::SPACE => " ",
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexemeTextualPart<'a> {
  TEXT(&'a str),
  EXPR(&'a [Lexeme<'a>]),
}

pub type SpannedLexeme<'a> = Spanned<Lexeme<'a>>;

impl<'a> Lexeme<'a> {
  /// Get the character attributed to this lexeme, if applicable, otherwise just
  /// returns `None`.
  pub fn get_char(&self) -> Option<char> {
    Some(match self {
      | Self::START_PAREN => '(',
      | Self::START_SQUARE_BRACKET => '[',
      | Self::START_CURLY_BRACE => '{',
      | Self::CLOSE_PAREN => ')',
      | Self::CLOSE_SQUARE_BRACKET => ']',
      | Self::CLOSE_CURLY_BRACE => '}',
      // keep-sorted start
      | Self::BACKTICK => '`',
      | Self::COMMA => ',',
      | Self::DOT => '.',
      | Self::SIGN_AT => '@',
      | Self::SIGN_EXCLAMATION => '!',
      | Self::SIGN_HASHTAG => '#',
      // keep-sorted end
      | _ => return None,
    })
  }

  pub fn from_char(char: char) -> Option<Self> {
    Some(match char {
      | '(' => Self::START_PAREN,
      | '[' => Self::START_SQUARE_BRACKET,
      | '{' => Self::START_CURLY_BRACE,
      | ')' => Self::CLOSE_PAREN,
      | ']' => Self::CLOSE_SQUARE_BRACKET,
      | '}' => Self::CLOSE_CURLY_BRACE,
      // keep-sorted start
      | '!' => Self::SIGN_EXCLAMATION,
      | '#' => Self::SIGN_HASHTAG,
      | ',' => Self::COMMA,
      | '.' => Self::DOT,
      | '@' => Self::SIGN_AT,
      | '`' => Self::BACKTICK,
      // keep-sorted end
      | _ => return None,
    })
  }
}
