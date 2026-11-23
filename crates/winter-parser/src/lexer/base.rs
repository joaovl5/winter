use strum_macros::Display;

pub type Lexer<'a> = logos::Lexer<'a, NormalToken<'a>>;

#[derive(logos::Logos, Debug, PartialEq, Clone, Display)]
#[logos(skip "((\r\n)+|[ \t\n]+)")]
pub enum NormalToken<'a> {
  // ## Start/stop tokens
  #[token("(")]
  START_PAREN,
  #[token("[")]
  START_SQUARE_BRACKET,
  #[token("{")]
  START_CURLY_BRACE,
  #[token(")")]
  CLOSE_PAREN,
  #[token("]")]
  CLOSE_SQUARE_BRACKET,
  #[token("}")]
  CLOSE_CURLY_BRACE,

  // ## Symbols
  #[token("`")]
  SYM_BACKTICK,
  #[token(",")]
  SYM_COMMA,
  #[token(".")]
  SYM_DOT,
  #[token("...")]
  SYM_ELLIPSIS,
  #[token("@")]
  SYM_AT,
  #[token("!")]
  SYM_BANG,
  #[token("#")]
  SYM_HASH,
  #[token("?")]
  SYM_QUESTION,

  #[token("+")]
  SYM_PLUS,
  #[token("++")]
  SYM_DOUBLE_PLUS,
  #[token("-")]
  SYM_MINUS,
  #[token("*")]
  SYM_STAR,
  #[token("/")]
  SYM_SLASH,
  #[token("//")]
  SYM_DOUBLE_SLASH,

  #[token(">")]
  SYM_GT,
  #[token(">=")]
  SYM_GTE,
  #[token("<")]
  SYM_LT,
  #[token("<=")]
  SYM_LTE,

  #[token("=")]
  SYM_EQ,
  #[token("!=")]
  SYM_NEQ,
  #[token("&&")]
  SYM_BOOL_AND,
  #[token("||")]
  SYM_BOOL_OR,
  #[token("->")]
  SYM_IMPLIES,
  #[token("or")]
  SYM_REC_OR,
  #[token("has")]
  SYM_REC_HAS,

  #[token("|>")]
  SYM_PIPE_RIGHT,
  #[token("<|")]
  SYM_PIPE_LEFT,

  // ## Keywords
  // ### General Keywords
  #[token("inherit")]
  KW_INHERIT,
  #[token("locals")]
  KW_LOCALS,
  #[token("let")]
  KW_LET,
  #[token("fn")]
  KW_FN,
  #[token("type")]
  KW_TYPE,
  #[token("namespace")]
  KW_NAMESPACE,
  #[token("namespace-use")]
  KW_NAMESPACE_USE,
  #[token("as")]
  KW_AS,
  #[token("if")]
  KW_IF,
  #[token("match")]
  KW_MATCH,

  // ### Type Keywords
  #[token("nil")]
  KW_NIL,
  #[token("bool")]
  KW_BOOL,
  #[token("str")]
  KW_STR,
  #[token("path")]
  KW_PATH,
  #[token("list")]
  KW_LIST,
  #[token("record")]
  KW_RECORD,
  #[token("expr")]
  KW_EXPR,
  #[token("forall")]
  KW_FORALL,

  // ### Macro/compile-time (CT) Keywords
  #[token("!macro")]
  KW_CT_MACRO,
  #[token("!expr")]
  KW_CT_EXPR,
  #[token("!first")]
  KW_CT_FIRST,
  #[token("!second")]
  KW_CT_SECOND,
  #[token("!last")]
  KW_CT_LAST,
  #[token("!tail")]
  KW_CT_TAIL,
  #[token("!unpack")]
  KW_CT_UNPACK,
  #[token("!gensym")]
  KW_CT_GENSYM,

  // ## Values
  #[regex("(@)?_*[a-zA-Z][_a-zA-Z0-9-']*", handle_atom)]
  VALUE_ATOM(&'a str),
  #[regex(r#"[0-9](\.[0-9]+)?"#)]
  VALUE_NUMBER(&'a str),
  #[token("\"", handle_textual)]
  VALUE_STRING(Vec<TextualToken<'a>>),
  #[token("p\"", handle_textual)]
  VALUE_PATH(Vec<TextualToken<'a>>),

  // ## Comments
  #[regex(";\\.[^\n]*", allow_greedy = true)]
  COMMENT_DOC(&'a str),
  #[regex(";[^\n]*", allow_greedy = true)]
  COMMENT_NORMAL(&'a str),
  // the parser itself will have to consume/ignore the block, logos doesn't
  // have the regex support to allow the `;_(\((?>[^()]*|(?1))*\))` regex
  #[token(";_")]
  COMMENT_FORM(&'a str),
}

#[derive(logos::Logos, Debug, PartialEq, Clone)]
pub enum TextualToken<'a> {
  #[token("\"")]
  DOUBLE_QUOTE,

  #[regex("[^\"$\\\\]+")]
  #[token("$")]
  LITERAL(&'a str),

  #[token("${", handle_interpolation)]
  INTERPOLATION(Vec<NormalToken<'a>>),
  // TODO: ESCAPED CHARS
}

fn handle_atom<'a>(
  lex: &mut logos::Lexer<'a, NormalToken<'a>>,
) -> Option<&'a str> {
  let out = lex.slice();
  match out.strip_prefix('@') {
    | Some(x) => Some(x),
    | None => Some(out),
  }
}

fn handle_textual<'a>(
  lex: &mut logos::Lexer<'a, NormalToken<'a>>,
) -> Option<Vec<TextualToken<'a>>> {
  let mut textual_lexer = lex.clone().morph::<TextualToken<'a>>();
  let mut tokens = Vec::<TextualToken<'a>>::new();
  for tk in textual_lexer.by_ref() {
    match tk {
      | Ok(TextualToken::DOUBLE_QUOTE) => break,
      | Err(_) => return None,
      | Ok(tk_value) => tokens.push(tk_value),
    }
  }
  *lex = textual_lexer.morph();

  Some(tokens)
}

fn handle_interpolation<'a>(
  lex: &mut logos::Lexer<'a, TextualToken<'a>>,
) -> Option<Vec<NormalToken<'a>>> {
  let mut normal_lexer = lex.clone().morph::<NormalToken<'a>>();
  let mut tokens = Vec::<NormalToken<'a>>::new();
  let mut brace_depth = 1;
  for tk in normal_lexer.by_ref() {
    match tk {
      | Ok(NormalToken::CLOSE_CURLY_BRACE) if brace_depth == 1 => break,
      | Err(_) => return None,
      | Ok(tk_value) => {
        if tk_value == NormalToken::START_CURLY_BRACE {
          brace_depth += 1;
        } else if tk_value == NormalToken::CLOSE_CURLY_BRACE {
          brace_depth -= 1;
        }

        tokens.push(tk_value)
      },
    }
  }
  *lex = normal_lexer.morph();

  Some(tokens)
}
