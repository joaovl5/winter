use super::*;
use crate::tokenizer::TokenVal;

impl<'tks, 'src> Parser<'tks, 'src> {
    pub(super) fn parse_scalar(&mut self) -> ParserResOpt<Ast<'src>> {
        let Some(tk) = self.peek() else {
            return Ok(None);
        };
        let mut call_next = true;
        let value = match &tk.value {
            TokenVal::Atom(at) => {
                call_next = false;
                self.parse_atom(at)?
            }
            TokenVal::Number(num) => AstVal::Number(num),
            TokenVal::StringLike { parts, is_path } => {
                self.parse_str_like(parts.as_slice(), *is_path)?
            }
            _ => return Err(self.error("Unexpected token")),
        };
        if call_next {
            self.next();
        }

        let ast = Ast {
            pos_start: tk.pos_start,
            pos_end: tk.pos_end,
            value,
        };
        Ok(Some(ast))
    }

    fn parse_atom(&mut self, at: &'src str) -> ParserRes<AstVal<'src>> {
        let mut call_next = true;
        let out = match at {
            "nil" => AstVal::Nil,
            "true" => AstVal::Bool(true),
            "false" => AstVal::Bool(false),
            _ => {
                // allow for nested attribute selection
                let (mut sel_nodes, _) = self.parse_dotted_path(false)?;
                call_next = false; // necessary due to `parse_dotted_path` calling next()
                                   // which we already do here, after the `match` clause
                match sel_nodes.len() {
                    0 => return Err(self.error("Atom was empty")),
                    1 => sel_nodes.pop().unwrap(),
                    _ => AstVal::Selection(sel_nodes),
                }
            }
        };
        if call_next {
            self.next();
        }
        Ok(out)
    }
}
