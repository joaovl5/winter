use super::*;
use crate::tokenizer::TokenVal;

impl<'tks, 'src> Parser<'tks, 'src> {
    pub(super) fn parse_list(&mut self) -> ParserResOpt<Ast<'src>> {
        let open_tk = self
            .next()
            .ok_or_else(|| self.error("Expected opening parenthesis"))?;

        let mut items: Vec<AstVal<'src>> = Vec::new();

        loop {
            match self
                .peek()
                .ok_or_else(|| self.error("Unterminated list"))?
                .value
            {
                TokenVal::CloseBracket => break,
                _ => items.push(
                    self.parse_one()?
                        .ok_or_else(|| self.error("Unterminated list"))?
                        .value,
                ),
            }
        }
        let Some(close_tk) = self.next() else {
            return Err(self.error("Unterminated block"));
        };

        Ok(Some(Ast {
            pos_start: open_tk.pos_start,
            pos_end: close_tk.pos_end,
            value: AstVal::List(items),
        }))
    }
}
