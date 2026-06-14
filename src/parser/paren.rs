use super::*;
use crate::tokenizer::TokenVal;

impl<'tks, 'src> Parser<'tks, 'src> {
    pub(super) fn parse_paren(&mut self) -> ParserResOpt<Ast<'src>> {
        /*
         * - parse callable head
         *   - if callable is special, pass over to specialized functions
         *      - functions
         *      - macros
         *      - ...
         *   - otherwise, parse tail of callable and return a generic `Callable`
         */
        let open_tk = &self
            .next()
            .ok_or_else(|| self.error("Expected opening parenthesis"))?;
        let head_t_val = &self
            .peek()
            .ok_or_else(|| self.error("Unterminated block"))?
            .value;
        let head: CallableHeadNodeType<'src>;
        if let Some(special_form) = self.parse_paren_special(head_t_val)? {
            return Ok(Some(special_form));
        }
        if let Some(head_op) = self.parse_paren_head_op(head_t_val)? {
            head = CallableHeadNodeType::Op(head_op);
        } else {
            let expr = self
                .parse_paren_head_expr()?
                .ok_or_else(|| self.error("Expected a paren head"))?;
            head = CallableHeadNodeType::Expr(Box::new(expr.value))
        }

        let mut tail: Vec<AstVal<'src>> = Vec::new();

        loop {
            match self
                .peek()
                .ok_or_else(|| self.error("Unterminated block"))?
                .value
            {
                TokenVal::CloseParen => break,
                _ => tail.push(
                    self.parse_one()?
                        .ok_or_else(|| self.error("Unterminated block"))?
                        .value,
                ),
            }
        }
        let close_tk = match self.next() {
            Some(tk) => tk,
            None => return Err(self.error("Unterminated block")),
        };

        Ok(Some(Ast {
            pos_start: open_tk.pos_start,
            pos_end: close_tk.pos_end,
            value: AstVal::Callable(head, tail),
        }))
    }

    fn parse_paren_special(&mut self, val: &TokenVal) -> ParserResOpt<Ast<'src>> {
        let TokenVal::Atom(x) = val else {
            return Ok(None);
        };
        match *x {
            "let" => self.parse_let(),
            "locals" => self.parse_locals(),
            "inherit" => self.parse_lone_inherit(),
            "fn" => self.parse_fn(),
            "type" => todo!(),
            "enum" => todo!(),
            _ => Ok(None),
        }
    }

    fn parse_paren_head_op(&mut self, val: &TokenVal) -> ParserResOpt<OperatorType> {
        use OperatorType::*;

        let res: OperatorType = match val {
            TokenVal::Atom(x) => match *x {
                "+" => Add,
                "-" => Sub,
                "*" => Mul,
                "/" => Div,
                ">" => Gt,
                "<" => Lt,
                ">=" => Gte,
                "<=" => Lte,
                "=" => Eq,
                "not=" => Neq,
                "not" => Not,
                "all" => All,
                "any" => Any,
                "thus" => Thus,
                "or" => Or,
                "has" => Has,
                "++" => Conc,
                "//" => Updt,
                "<|" => PipeL,
                "|>" => PipeR,
                _ => return Ok(None),
            },
            TokenVal::Dot => Sel,
            _ => return Ok(None),
        };
        self.next();
        Ok(Some(res))
    }

    fn parse_paren_head_expr(&mut self) -> ParserResOpt<Ast<'src>> {
        // first check if next thing is an expr
        // otherwise use assume a dotted path
        let Some(tk) = self.peek() else {
            return Err(self.error("Unexpectedly missing token"));
        };
        let TokenVal::Atom(_) = tk.value else {
            return self.parse_one();
        };
        let pos_start = tk.pos_start;
        let (mut head_nodes, pos_end) = self.parse_dotted_path(false)?;
        let ast_val = match head_nodes.len() {
            // 0 is unreachable
            1 => head_nodes.pop().unwrap(),
            _ => AstVal::Selection(head_nodes),
        };
        let ast = Ast {
            value: ast_val,
            pos_start,
            pos_end,
        };

        Ok(Some(ast))
    }
}
