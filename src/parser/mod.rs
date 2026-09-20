use itertools::{peek_nth, PeekNth};
use std::slice::Iter;

use crate::tokenizer::{Token, TokenVal};
use crate::utils::ResOpt;

pub mod ast;
pub mod binds;
pub mod function;
pub mod list;
pub mod map;
pub mod paren;
pub mod refs;
pub mod scalars;
pub mod shared;
pub mod walker;

pub use ast::*;

#[derive(Debug, Clone)]
pub struct Parser<'tks, 'src> {
    /// we have two generic lifetimes to differentiate between
    /// a) source string borrows (`'src`)
    /// b) token iter borrows (`'tks`)
    /// which is useful for some subproblems of the parser
    tokens: &'tks [Token<'src>],
    pos: usize,
    iter: Box<PeekNth<Iter<'tks, Token<'src>>>>,
}

pub(super) type ParserResOpt<T> = ResOpt<T, ParserError>;
pub(super) type ParserRes<T> = Result<T, ParserError>;

impl<'tks, 'src> Parser<'tks, 'src> {
    pub fn new(tokens: &'tks [Token<'src>]) -> Self {
        Self {
            pos: 0,
            iter: Box::new(peek_nth(tokens.iter())),
            tokens,
        }
    }

    pub fn parse(&mut self) -> Result<Module<'src>, ParserError> {
        let mut nodes: Vec<Ast<'src>> = Vec::new();
        loop {
            let tk = self.parse_one()?;
            match tk {
                Some(tk) => nodes.push(tk),
                None => break,
            }
        }

        Ok(Module { nodes })
    }

    fn parse_one(&mut self) -> ParserResOpt<Ast<'src>> {
        /*
         * Precedence - outside-in
         *  1 - callables ()
         *      - < selectable (atom + . + atom + ...) > + <thing[]>
         *      - selectable precedence:
         *         1 - operators
         *         1 - builtin stuff - path - fn/lambda
         *         2 - other stuff
         *  1 - lists []
         *      - <thing>[]
         *  1 - maps {}
         *      - (< attrpath (atom/str + . + atom/str + ...) > + <thing>)[]
         *      - OR
         *      - (! <compileable-to-str-expr> <thing>)[]
         *  2 - bool - nil - fn - lambda
         *
         *  Later sem. analysis - outside-in
         *  - unpack `fn`s into `lambda`s
         *  - unpack `locals` into `let`s
         *  - unpack `inherit`s
         *      - in `let`s
         *      - in lists (to with)
         *      - in attrsets
         *  - type analysis
         *  - resolve declared types/enums
         *      - types instances -> attrsets
         *      - enum values -> values
         *  - compile ! keys in attrs
         */
        let Some(tk) = self.peek() else {
            return Ok(None);
        };

        // We do NOT call `self.next()` here
        // We expect the clauses to perform this on their own
        // (parse_paren, parse_list, etc)
        match tk.value {
            TokenVal::OpenParen => self.parse_paren(),
            TokenVal::OpenBracket => self.parse_list(),
            TokenVal::OpenBrace => self.parse_map(),
            _ => self.parse_scalar(),
        }
    }

    fn error(&self, msg: &str) -> ParserError {
        // try to get current token, otherwise get the last
        let msg = msg.to_string();
        let Some(tk) = &self.tokens.get(self.pos).or(self.tokens.last()) else {
            return ParserError {
                msg,
                pos_start: 0,
                pos_end: 0,
            };
        };
        ParserError {
            msg,
            pos_start: tk.pos_start,
            pos_end: tk.pos_end,
        }
    }

    fn peek(&mut self) -> Option<&'tks Token<'src>> {
        self.iter.peek().cloned()
    }

    fn peek_nth(&mut self, nth: usize) -> Option<&'tks Token<'src>> {
        self.iter.peek_nth(nth).cloned()
    }

    fn next(&mut self) -> Option<&'tks Token<'src>> {
        let ch = self.iter.next()?;
        self.pos += 1;
        Some(ch)
    }

    fn attempt<T>(
        &mut self,
        // function acting upon a forked 'Parser'
        f: impl FnOnce(&mut Self) -> ParserResOpt<T>,
    ) -> ParserResOpt<T> {
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
pub struct ParserError {
    pub msg: String,
    pub pos_start: usize,
    pub pos_end: usize,
}
