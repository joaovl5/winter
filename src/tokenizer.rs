use std::{iter::Peekable, str::Chars};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TokenVal<'tk> {
    OpenParen,    // (
    OpenBracket,  // [
    OpenBrace,    // {
    CloseParen,   // )
    CloseBracket, // ]
    CloseBrace,   // }
    Hash,         // #
    At,           // @
    Exclamation,  // !
    QuestionMark, // ?
    Backtick,     // `
    Comma,        // ,
    Dot,          // .
    Ellipsis,     // ...
    /// <some-identifiable-form> or
    /// some non-string, non-number type/type-value
    /// like nil/bools or operators/functions
    Atom(&'tk str),
    /// number (decimal/int)
    Number(&'tk str),
    /// string/path construct
    StringLike {
        parts: Vec<StringLikeTokenPart<'tk>>,
        is_path: bool,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Token<'tk> {
    pub value: TokenVal<'tk>,
    pub pos_start: usize,
    pub pos_end: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StringLikeTokenPart<'tk> {
    Text(String),
    Expr(Vec<Token<'tk>>), // tokens between braces
}

#[derive(Debug, Clone)]
pub struct Tokenizer<'tk> {
    pos: usize,
    source: &'tk str,
    iter: Peekable<Chars<'tk>>,
}

impl<'tk> Tokenizer<'tk> {
    pub fn new(source: &'tk str) -> Self {
        Self {
            pos: 0,
            iter: source.chars().peekable(),
            source,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Token<'tk>>, TokenizerError> {
        let mut out: Vec<Token<'tk>> = Vec::new();
        loop {
            let tk = self.parse_one()?;
            match tk {
                Some(tk) => out.push(tk),
                None => break,
            }
        }

        Ok(out)
    }

    fn parse_one(&mut self) -> Result<Option<Token<'tk>>, TokenizerError> {
        self.skip_ignored();

        let Some(ch) = self.peek() else {
            return Ok(None);
        };

        if self.starts_with("...") {
            let pos_start = self.pos;
            self.next();
            self.next();
            self.next();
            return Ok(Some(Token {
                value: TokenVal::Ellipsis,
                pos_start,
                pos_end: self.pos,
            }));
        }

        if let Some(tk_val) = Self::single_char_token(ch) {
            let pos_start = self.pos;
            self.next();
            return Ok(Some(Token {
                value: tk_val,
                pos_start,
                pos_end: self.pos,
            }));
        }

        let pos_start = self.pos;
        let tk_val = match self.attempt(|fork| fork.parse_string_like())? {
            Some(tk_str_like) => tk_str_like,
            None => match self.parse_number()? {
                Some(tk_num) => tk_num,
                None => self.parse_atom()?,
            },
        };

        return Ok(Some(Token {
            value: tk_val,
            pos_start,
            pos_end: self.pos,
        }));
    }

    fn single_char_token(ch: char) -> Option<TokenVal<'tk>> {
        use TokenVal::*;

        Some(match ch {
            '(' => OpenParen,
            '[' => OpenBracket,
            '{' => OpenBrace,
            ')' => CloseParen,
            ']' => CloseBracket,
            '}' => CloseBrace,
            '#' => Hash,
            '@' => At,
            '!' => Exclamation,
            '?' => QuestionMark,
            '`' => Backtick,
            ',' => Comma,
            '.' => Dot,
            _ => return None,
        })
    }

    fn parse_string_like(&mut self) -> Result<Option<TokenVal<'tk>>, TokenizerError> {
        let mut has_opened: bool = false;
        let mut is_interp: bool = false;
        let mut is_path: bool = false;
        let mut parts: Vec<StringLikeTokenPart<'tk>> = Vec::new();
        let mut cur_value: String = String::new();

        loop {
            let Some(ch) = self.next() else {
                if !has_opened {
                    return Ok(None);
                }
                return Err(self.error("Unterminated string"));
            };

            if !has_opened {
                match ch {
                    '"' => has_opened = true,
                    '$' if self.peek() == Some('"') => is_interp = true,
                    'p' if !is_path => is_path = true,
                    _ => return Ok(None),
                }
                continue;
            }

            match ch {
                '"' => {
                    parts.push(StringLikeTokenPart::Text(cur_value));
                    return Ok(Some(TokenVal::StringLike { parts, is_path }));
                }
                '$' => cur_value.push_str("\\$"), // avoid nix treating it as interp.
                '\\' => match self
                    .next()
                    .ok_or_else(|| self.error("Unterminated backslash"))?
                {
                    'n' => cur_value.push('\n'),
                    'r' => cur_value.push('\r'),
                    't' => cur_value.push('\t'),
                    '"' => cur_value.push('"'),
                    '\\' => cur_value.push('\\'),
                    other => {
                        return Err(
                            self.error(&format!("unsupported escape sequence of '\\{other}'"))
                        )
                    }
                },
                '{' if is_interp => match self
                    .peek()
                    .ok_or_else(|| self.error("Unterminated string"))?
                {
                    '{' => {
                        cur_value.push('{');
                        self.next();
                    }
                    _ => {
                        parts.push(StringLikeTokenPart::Text(cur_value));
                        parts.push(StringLikeTokenPart::Expr(self.parse_string_like_interp()?));
                        cur_value = String::new();
                    }
                },
                '}' if is_interp => match self
                    .peek()
                    .ok_or_else(|| self.error("Unterminated string"))?
                {
                    '}' => {
                        cur_value.push('}');
                        self.next();
                    }
                    _ => return Err(self.error("Unexpected `}` in interpolated string.")),
                },
                ch => cur_value.push(ch),
            }
        }
    }

    fn parse_string_like_interp(&mut self) -> Result<Vec<Token<'tk>>, TokenizerError> {
        // starts after `{`
        let mut out: Vec<Token<'tk>> = Vec::new();
        let mut brace_depth = 1;
        loop {
            let Some(tk) = self.parse_one()? else {
                return Err(self.error("Unterminated string interpolation"));
            };
            if matches!(&tk.value, TokenVal::OpenBrace) {
                brace_depth += 1;
                out.push(tk);
            } else if matches!(&tk.value, TokenVal::CloseBrace) {
                brace_depth -= 1;
                if brace_depth == 0 {
                    return Ok(out);
                }
                out.push(tk);
            } else {
                out.push(tk);
            }
        }
    }

    fn parse_number(&mut self) -> Result<Option<TokenVal<'tk>>, TokenizerError> {
        let start = self.pos;

        let Some(()) = self.attempt(|fork| {
            if fork.peek() == Some('-') {
                fork.next();
            }

            let digits_start = fork.pos;
            fork.traverse_digits();
            if digits_start == fork.pos {
                return Ok(None);
            }

            Ok(Some(()))
        })?
        else {
            return Ok(None);
        };

        self.attempt(|fork| {
            if fork.peek() != Some('.') {
                return Ok(None);
            }

            fork.next();

            let digits_start = fork.pos;
            fork.traverse_digits();
            if digits_start == fork.pos {
                return Ok(None);
            }

            Ok(Some(()))
        })?;

        if matches!(self.peek(), Some(ch) if Self::is_atom_char(ch)) {
            return Err(self.error("Invalid number literal"));
        }

        Ok(Some(TokenVal::Number(&self.source[start..self.pos])))
    }

    fn traverse_digits(&mut self) {
        while matches!(self.peek(), Some(ch) if ch.is_ascii_digit()) {
            self.next();
        }
    }

    fn parse_atom(&mut self) -> Result<TokenVal<'tk>, TokenizerError> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if !Self::is_atom_char(ch) {
                break;
            }
            self.next();
        }
        if start == self.pos {
            return Err(self.error("expected atom"));
        }
        Ok(TokenVal::Atom(&self.source[start..self.pos]))
    }

    fn is_atom_char(ch: char) -> bool {
        ch.is_ascii_graphic() // is visible char
            && ch != ';' // isn't comment
            && ch != '"' // isn't stringy
            && Self::single_char_token(ch).is_none() // isn't some other reserved char
    }

    fn error(&self, msg: &str) -> TokenizerError {
        return TokenizerError {
            msg: format!("{msg}"),
            pos: self.pos,
        };
    }

    fn skip_ignored(&mut self) {
        loop {
            match self.peek() {
                Some(ch) if ch.is_whitespace() => {
                    self.next();
                }
                Some(';') => while !matches!(self.next(), Some('\n') | None) {},
                _ => break,
            }
        }
    }

    fn peek(&mut self) -> Option<char> {
        return self.iter.peek().copied();
    }

    fn next(&mut self) -> Option<char> {
        let ch = self.iter.next()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn starts_with(&self, s: &str) -> bool {
        let mut iter = self.iter.clone();

        s.chars().all(|expected| iter.next() == Some(expected))
    }

    fn attempt<T>(
        &mut self,
        // function acting upon a forked `Tokenizer`
        f: impl FnOnce(&mut Self) -> Result<Option<T>, TokenizerError>,
    ) -> Result<Option<T>, TokenizerError> {
        let mut fork = self.clone();

        if let Some(value) = f(&mut fork)? {
            *self = fork;
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }
}

#[derive(Debug, Clone)]
pub struct TokenizerError {
    pub msg: String,
    pub pos: usize,
}
