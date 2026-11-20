use itertools::Itertools;

use super::{LexemeResOpt, Lexer};
use crate::lexer::LexerError;
use crate::types::{Lexeme, LexemeTextualPart};
use crate::{err, ok_none, ok_some, span_err, span_this};

impl<I: Iterator<Item = char>> Lexer<I> {
  pub(super) fn parse_textual(&mut self) -> LexemeResOpt {
    self.cur.pos_checkpoint();
    let pos_start = self.pos();
    let mut has_opened = false;
    let mut is_interp = false;
    let mut is_path = false;
    let mut parts = Vec::<LexemeTextualPart>::new();
    let mut current_val = Vec::<String>::new();

    loop {
      let Some(ch) = self.next()? else {
        if has_opened {
          span_err!(pos_start, self.pos(), LexerError::UNT_STR);
        } else {
          break;
        };
      };
      if !has_opened {
        match ch {
          | '"' => has_opened = true,
          | '$' if self.peek()?.is_some_and(|x| x == '"') => is_interp = true,
          | 'p' if !is_path => is_path = true,
          | _ => {
            break;
          },
        };
        continue;
      }
      match ch {
        | '"' => {
          parts.push(LexemeTextualPart::TEXT(current_val.iter().join(" ")));
          current_val.clear();
          break;
        },
        // NOTE: maybe this should be owned by the nix lowering step instead?
        // since this is effectively nix-specific logic we're encoding at the
        // lexer level
        | '$' => current_val.push("\\$".to_string()),
        | '\\' => match self.peek()? {
          | None => span_err!(self.pos(), LexerError::UNT_STR_BS),
          | Some(ch) if let Some(bs_val) = get_bs_val(ch) =>
            current_val.push(bs_val.to_string()),
          | Some(ch) => span_err!(self.pos(), LexerError::UNT_STR_BS_SEQ(ch)),
        },
        | '{' if is_interp => match self.peek()? {
          | None => span_err!(self.pos(), LexerError::UNT_STR_INTERP),
          | Some('{') => current_val.push("{".to_string()),
          | Some(_) => {
            parts.push(LexemeTextualPart::TEXT(current_val.iter().join(" ")));
            current_val.clear();
            let mut interp_parts = Vec::<Lexeme>::new();
            let mut brace_depth = 1;
            loop {
              let Some(tk) = self.parse_one()? else {
                span_err!(self.pos(), LexerError::UNT_STR_INTERP)
              };
              match tk.1 {
                | Lexeme::START_CURLY_BRACE => brace_depth += 1,
                | Lexeme::CLOSE_CURLY_BRACE => brace_depth -= 1,
                | _ => (),
              };
              if brace_depth == 0 {
                break;
              }
              // NOTE: we may remodel things later so parts can be spanned too
              interp_parts.push(tk.1);
            }
          },
        },
        | '}' if is_interp => match self.peek()? {
          | None => span_err!(self.pos(), LexerError::UNT_STR),
          | Some('}') => current_val.push("}".to_string()),
          | Some(_) => span_err!(self.pos(), LexerError::UNEX_STR_INTERP_CLOSE),
        },
        | _ => current_val.push(ch.to_string()),
      };
    }
    let pos_end = self.pos();
    if parts.is_empty() {
      super::map_err(self.pos(), self.cur.pos_rollback())?;
      ok_none!();
    } else {
      super::map_err(self.pos(), self.cur.pos_commit())?;
      let lex = Lexeme::VALUE_TEXTUAL { parts, is_path };
      ok_some!(span_this!(pos_start, pos_end, lex))
    };
  }
}

fn get_bs_val(ch: char) -> Option<&'static str> {
  match ch {
    | 'n' => Some("\n"),
    | 'r' => Some("\r"),
    | 't' => Some("\t"),
    | '"' => Some("\""),
    | '\\' => Some("\\"),
    | _ => None,
  }
}
