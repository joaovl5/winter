use super::*;
use crate::tokenizer::TokenVal;

impl<'tks, 'src> Parser<'tks, 'src> {
    pub(super) fn parse_let(&mut self) -> ParserResOpt<Ast<'src>> {
        // in (
        self.next(); // in `let`
        let pos_start = self.tokens[self.pos].pos_start;

        // expect list of bindings
        let binds_start_tk = match self.next() {
            Some(tk) => tk,
            None => return Err(self.error("Unterminated let bind")),
        };
        if !matches!(binds_start_tk.value, TokenVal::OpenBracket) {
            return Err(self.error("Expected bracket after `let` call"));
        }
        // get binds
        let (binds, inherits) = self.parse_binds_list(TokenVal::CloseBracket)?;
        let Some(body) = self.parse_one()? else {
            return Err(self.error("Expected a body in let bind"));
        };

        let let_end_tk = match self.next() {
            Some(tk) => tk,
            None => return Err(self.error("Unterminated let bind")),
        };
        if !matches!(let_end_tk.value, TokenVal::CloseParen) {
            return Err(self.error("Expected parenthesis after `let` body"));
        }

        let pos_end = self.tokens[self.pos - 1].pos_end;

        Ok(Some(Ast {
            pos_start,
            pos_end,
            value: AstVal::Let {
                locals: LocalsLike { binds, inherits },
                body: Box::new(body.value),
            },
        }))
    }

    pub(super) fn parse_locals(&mut self) -> ParserResOpt<Ast<'src>> {
        // in (
        self.next(); // in `locals`
        let pos_start = self.tokens[self.pos].pos_start;

        let (binds, inherits) = self.parse_binds_list(TokenVal::CloseParen)?;
        let pos_end = self.tokens[self.pos - 1].pos_end;
        Ok(Some(Ast {
            pos_start,
            pos_end,
            value: AstVal::Locals(LocalsLike { binds, inherits }),
        }))
    }

    pub(super) fn parse_lone_inherit(&mut self) -> ParserResOpt<Ast<'src>> {
        // in (
        self.next(); // in `locals`
        let pos_start = self.tokens[self.pos].pos_start;
        let inherits = self.parse_inherits()?;
        let pos_end = self.tokens[self.pos - 1].pos_end;
        Ok(Some(Ast {
            pos_start,
            pos_end,
            value: AstVal::Inherit(inherits),
        }))
    }

    fn parse_binds_list(
        &mut self,
        stop_tk_val: TokenVal,
    ) -> ParserRes<(BoundNodes<'src>, InheritNodes<'src>)> {
        let mut binds: BoundNodes<'src> = Vec::new();
        let mut inherits: InheritNodes<'src> = Vec::new();
        loop {
            let Some(next_tk) = self.peek() else {
                return Err(self.error("Unterminated let bind"));
            };
            match &next_tk.value {
                x if *x == stop_tk_val => break,
                TokenVal::OpenParen
                    if let TokenVal::Atom(at) = self
                        .peek_nth(1)
                        .ok_or_else(|| self.error("Unterminated block"))?
                        .value
                        && at == "inherit" =>
                {
                    self.next();
                    self.next();
                    inherits.extend(self.parse_inherits()?);
                }
                _ => {
                    let bind = self
                        .parse_one_bind()?
                        .ok_or_else(|| self.error("Expected bind inside `let`"))?;
                    binds.push(bind);
                }
            }
        }
        self.next();

        Ok((binds, inherits))
    }

    pub(super) fn parse_inherits(&mut self) -> ParserRes<InheritNodes<'src>> {
        let mut out: InheritNodes<'src> = Vec::new();

        loop {
            let Some(bind_tk) = self.next() else {
                return Err(self.error("Unterminated inherit block"));
            };
            let bind_at = match bind_tk.value {
                TokenVal::CloseParen => break,
                TokenVal::Atom(bind_at) => bind_at,
                _ => {
                    return Err(self.error("Inherit item can only be an identifier"));
                }
            };

            let Some(next_tk) = self.peek() else {
                return Err(self.error("Unterminated inherit block"));
            };
            match next_tk.value {
                TokenVal::OpenParen => {
                    // consume list (A)
                    self.next();
                    let items = self.parse_inherit_items(TokenVal::CloseParen)?;
                    out.push((bind_at, Some(items)));
                }
                TokenVal::Atom(_) | TokenVal::CloseParen => {
                    // consume item (from `bind_at`) (B)
                    out.push((bind_at, None));
                }
                _ => {
                    return Err(
                        self.error("Inherit binding cannot contain non-identifier expressions")
                    )
                }
            }
        }

        Ok(out)
    }

    fn parse_inherit_items(
        &mut self,
        stop_tk_val: TokenVal,
    ) -> ParserRes<Vec<TypedReference<'src>>> {
        let mut items = Vec::new();
        loop {
            let Some(item_tk) = self.peek() else {
                return Err(self.error("Unterminated inherit items"));
            };
            match &item_tk.value {
                x if *x == stop_tk_val => break,
                _ => items.push(self.parse_typed_reference()?),
            }
        }
        self.next();

        Ok(items)
    }

    fn parse_one_bind(&mut self) -> ParserResOpt<BoundNode<'src>> {
        let Some(bind_tk) = self.next() else {
            return Ok(None);
        };
        let TokenVal::Atom(bind_at) = bind_tk.value else {
            return Err(self.error("Expected identifier within bind declaration"));
        };
        let Some(bound_ast) = self.parse_one()? else {
            return Err(self.error("Expected value for bind declaration"));
        };
        // TODO: handle type annotations
        Ok(Some((
            TypedReference {
                name: bind_at,
                type_val: None,
            },
            bound_ast.value,
        )))
    }
}
