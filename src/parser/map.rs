use super::*;
use crate::tokenizer::TokenVal;

impl<'tks, 'src> Parser<'tks, 'src> {
    pub(super) fn parse_map(&mut self) -> ParserResOpt<Ast<'src>> {
        let open_tk = self
            .next()
            .ok_or_else(|| self.error("Expected opening brace"))?;

        let mut items: Vec<(ManyNodes<'src>, AstVal<'src>)> = Vec::new();
        let mut inherits: InheritNodes<'src> = Vec::new();

        loop {
            match self
                .peek()
                .ok_or_else(|| self.error("Unterminated map"))?
                .value
            {
                TokenVal::Exclamation => {
                    return Err(self.error("the `!` syntax for map keys is not implemented yet"))
                }
                TokenVal::CloseBrace => break,
                TokenVal::OpenParen
                    if let TokenVal::Atom(at) = self
                        .peek_nth(1)
                        .ok_or_else(|| self.error("Unterminated map"))?
                        .value
                        && at == "inherit" =>
                {
                    self.next();
                    self.next();
                    inherits.extend(self.parse_inherits()?);
                }
                _ => {
                    // need to get a tuple
                    // get dotted path first as `ManyNode`
                    let (key, _) = self.parse_dotted_path(false)?;
                    let val = self
                        .parse_one()?
                        .ok_or_else(|| self.error("Expected a value next to a key inside map"))?;
                    items.push((key, val.value));
                }
            }
        }

        let Some(close_tk) = self.next() else {
            return Err(self.error("Unterminated map"));
        };

        Ok(Some(Ast {
            pos_start: open_tk.pos_start,
            pos_end: close_tk.pos_end,
            value: AstVal::Map(MapVal {
                binds: items,
                inherits,
            }),
        }))
    }
}
