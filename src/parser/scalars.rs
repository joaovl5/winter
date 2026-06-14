use super::*;
use crate::tokenizer::TokenVal;

impl<'tks, 'src> Parser<'tks, 'src> {
    pub(super) fn parse_scalar(&mut self) -> ParserResOpt<Ast<'src>> {
        let Some(tk) = self.peek() else {
            return Ok(None);
        };
        let pos_start = tk.pos_start;
        let mut pos_end = tk.pos_end;
        let mut call_next = true;
        let value = match &tk.value {
            TokenVal::Atom(at) => {
                call_next = false;
                let (value, atom_pos_end) = self.parse_atom(at)?;
                pos_end = atom_pos_end;
                value
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
            pos_start,
            pos_end,
            value,
        };
        Ok(Some(ast))
    }

    fn parse_atom(&mut self, at: &'src str) -> ParserRes<(AstVal<'src>, usize)> {
        let pos_end = self.tokens[self.pos].pos_end;
        let mut call_next = true;
        let mut out_pos_end = pos_end;
        let out = match at {
            "nil" => AstVal::Nil,
            "true" => AstVal::Bool(true),
            "false" => AstVal::Bool(false),
            _ => {
                // allow for nested attribute selection
                let (mut sel_nodes, sel_pos_end) = self.parse_dotted_path(true)?;
                call_next = false; // parse_dotted_path already advances
                out_pos_end = sel_pos_end;
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
        Ok((out, out_pos_end))
    }
}
