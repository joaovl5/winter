use console::{StyledObject, style};
use itertools::Itertools;
use winter_parser::types::{Lexeme, LexemeTextualPart};

use super::PrettyPrintable;

impl PrettyPrintable for Lexeme {
  type Printable = String;

  fn pprint_str(&self) -> StyledObject<String> {
    let sym_pre = style("«").cyan();
    let sym_post = style("»").cyan();
    let sym = match self {
      | Lexeme::VALUE_ATOM(_) => "atom",
      | Lexeme::VALUE_NUMBER(_) => "num",
      | Lexeme::VALUE_TEXTUAL { is_path, .. } => match *is_path {
        | true => "path",
        | false => "str",
      },
      | Lexeme::VALUE_COMMENT(_) => "comment",
      | Lexeme::VALUE_WHITESPACE { .. } => "ws",
      | x if let Some(ch) = x.get_char() => &ch.to_string(),
      | _ => unreachable!(),
    };
    let inner: Option<StyledObject<String>> = match self {
      | Lexeme::VALUE_ATOM(x)
      | Lexeme::VALUE_NUMBER(x)
      | Lexeme::VALUE_COMMENT(x) => Some(style(x.to_string())),
      | Lexeme::VALUE_TEXTUAL { parts, .. } => Some(style(
        parts
          .iter()
          .map(|part| match part {
            | LexemeTextualPart::TEXT(txt) => style(txt.clone()),
            | LexemeTextualPart::EXPR(expr) => style(
              expr
                .iter()
                .map(Lexeme::pprint_str)
                .join(" ")
                .clone(),
            ),
          })
          .join(" "),
      )),
      | _ => None,
    };
    let inner_repr = match inner {
      | None => style("".to_string()),
      | Some(v) => style(format!("({})", v)),
    };
    let out = format!("{sym_pre}{sym}{inner_repr}{sym_post}");
    style(out)
  }

  fn pprint(&self) {
    print!("{} ", self.pprint_str());
  }
}
