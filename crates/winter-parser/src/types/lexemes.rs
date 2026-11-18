use super::Spanned;

pub enum Lexeme<'input> {
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
  VALUE_ATOM(&'input str),
  VALUE_NUMBER(&'input str),
  VALUE_TEXTUAL { parts: Vec<LexemeTextualPart<'input>>, is_path: bool },
  VALUE_COMMENT(&'input str),
}

pub enum LexemeTextualPart<'input> {
  TEXT(&'input str),
  EXPR(Vec<Lexeme<'input>>),
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
