use super::*;
use crate::tokenizer::TokenVal;

impl<'tks, 'src> Parser<'tks, 'src> {
    pub(super) fn parse_fn(&mut self) -> ParserResOpt<Ast<'src>> {
        // in (
        self.next(); // in `fn`
        let pos_start = self.tokens[self.pos].pos_start;

        let name = match self
            .peek()
            .ok_or_else(|| self.error("Unterminated function definition"))?
            .value
        {
            TokenVal::Atom(_) => {
                // In this case, `@` may signal a single and named unpacked param, like:
                // ```
                // (fn args@{pkgs, config, ...})
                // ```
                // Thus, to distinguish between a return type `name@T`, we check if
                // T starts with `{` here
                let nth_1 = &self
                    .peek_nth(1)
                    .ok_or_else(|| self.error("Unterminated function definition"))?
                    .value;
                let nth_2 = &self
                    .peek_nth(2)
                    .ok_or_else(|| self.error("Unterminated function definition"))?
                    .value;
                if matches!(nth_1, TokenVal::At) && matches!(nth_2, TokenVal::OpenBrace) {
                    None
                } else {
                    Some(self.parse_typed_reference()?)
                }
            }
            _ => None,
        };
        let params = self.parse_fn_params()?;
        if params.is_empty() {
            return Err(self.error("Function parameters must not be empty"));
        }
        let mut body: ManyNodes<'src> = Vec::new();

        loop {
            let Some(next_tk) = self.peek() else {
                return Err(self.error("Unterminated function definition"));
            };
            match &next_tk.value {
                TokenVal::CloseParen => break,
                _ => body.push(
                    self.parse_one()?
                        .ok_or_else(|| self.error("Expected node within function body"))?
                        .value,
                ),
            }
        }
        self.next();

        if body.is_empty() {
            return Err(self.error("Function body must not be empty"));
        }

        let pos_end = self.tokens[self.pos - 1].pos_end;

        Ok(Some(Ast {
            pos_start,
            pos_end,
            value: AstVal::Function { name, params, body },
        }))
    }

    fn parse_fn_params(&mut self) -> ParserRes<ManyParams<'src>> {
        let mut params: ManyParams<'src> = Vec::new();
        // all functions should have at least one parameter!

        let params_start_tk = match self.peek() {
            Some(tk) => tk,
            None => return Err(self.error("Unterminated function definition")),
        };
        match params_start_tk.value {
            // 1) treating single-param unpacked map
            // 1.1) case where param is named
            TokenVal::Atom(_)
                if let TokenVal::At = &self
                    .peek_nth(1)
                    .ok_or_else(|| self.error("Unterminated function definition"))?
                    .value =>
            {
                if !matches!(
                    &self
                        .peek_nth(2)
                        .ok_or_else(|| self.error("Unterminated function definition"))?
                        .value,
                    TokenVal::OpenBrace
                ) {
                    return Err(
                        self.error("Expected unpacked map after named single-argument shorthand")
                    );
                }
                let ref_name = self.parse_typed_reference()?;
                self.next();
                params.push(self.parse_unpacked_fn_param(Some(ref_name))?);
            }
            // 1.2) case where param is not named
            TokenVal::OpenBrace => params.push(self.parse_unpacked_fn_param(None)?),

            // 2) treating list of multiple params
            TokenVal::OpenBracket => {
                self.next();
                loop {
                    let Some(next_tk) = self.peek() else {
                        return Err(self.error("Unterminated function parameters"));
                    };
                    match &next_tk.value {
                        TokenVal::CloseBracket => break,
                        _ => params.push(self.parse_one_fn_param()?),
                    }
                }
                self.next();
            }
            _ => {
                return Err(
                    self.error("Unexpected token where a function argument definition should be")
                )
            }
        }

        Ok(params)
    }

    fn parse_one_fn_param(&mut self) -> ParserRes<FuncParam<'src>> {
        let res = match &self
            .peek()
            .ok_or_else(|| self.error("Expected function parameter"))?
            .value
        {
            TokenVal::Atom(_) => {
                let ref_name = self.parse_typed_reference()?;
                match &self
                    .peek()
                    .ok_or_else(|| self.error("Unterminated function parameters"))?
                    .value
                {
                    // using `At` here is fine given we expect `parse_typed_reference`
                    // to handle parsing type-annotation `At` signs
                    TokenVal::At => {
                        self.next();
                        self.parse_unpacked_fn_param(Some(ref_name))?
                    }
                    _ => FuncParam::Plain(ref_name),
                }
            }
            TokenVal::OpenBrace => self.parse_unpacked_fn_param(None)?,
            _ => return Err(self.error("Unexpected token within function parameters")),
        };

        Ok(res)
    }

    /// Parse an unpacked, and potentially named, function parameter
    ///
    /// * `name_ref`: Optional name for function parameter
    fn parse_unpacked_fn_param(
        &mut self,
        name_ref: Option<TypedReference<'src>>,
    ) -> ParserRes<FuncParam<'src>> {
        let name = match name_ref {
            Some(x) => {
                if x.type_val.is_some() {
                    return Err(
                        self.error("Unpacked function parameters cannot have type signatures")
                    );
                }
                Some(x.name)
            }
            _ => None,
        };
        let params_start_tk = match self.next() {
            Some(tk) => tk,
            None => return Err(self.error("Unterminated function parameters")),
        };
        if !matches!(params_start_tk.value, TokenVal::OpenBrace) {
            return Err(self.error("Expected brace after `@` for parameter unpacking"));
        }
        let mut params: Vec<(TypedReference<'src>, Option<OneNode<'src>>)> = Vec::new();
        let mut allow_extra = false;

        loop {
            match &self
                .peek()
                .ok_or_else(|| self.error("Unterminated unpacked parameter"))?
                .value
            {
                TokenVal::CloseBrace => break,
                TokenVal::Ellipsis => {
                    self.next();
                    if !allow_extra {
                        allow_extra = true;
                    } else {
                        return Err(self.error(
                            "Cannot have more than one ellipsis within unpacked parameter",
                        ));
                    }
                }
                TokenVal::Atom(_) => {
                    let typed_name = self.parse_typed_reference()?;
                    let default_value = match &self
                        .peek()
                        .ok_or_else(|| self.error("Unterminated unpacked parameter"))?
                        .value
                    {
                        TokenVal::QuestionMark => {
                            self.next();
                            let expr = self
                                .parse_one()?
                                .ok_or_else(|| self.error("Unterminated unpacked parameter"))?
                                .value;
                            Some(Box::new(expr))
                        }
                        _ => None,
                    };
                    params.push((typed_name, default_value))
                }
                _ => return Err(self.error("Unexpected token within unpacked parameter")),
            }
        }
        self.next();

        Ok(FuncParam::Unwrapped {
            name,
            params,
            allow_extra,
        })
    }
}
