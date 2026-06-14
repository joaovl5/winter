use super::*;
use crate::tokenizer::{StringLikeTokenPart, TokenVal};

impl<'tks, 'src> Parser<'tks, 'src> {
    pub(super) fn parse_dotted_path(
        &mut self,
        allow_eof: bool,
    ) -> ParserRes<(ManyNodes<'src>, usize)> {
        // assumes `peek()` is in pos to get first item of path
        let mut items: ManyNodes<'src> = Vec::new();
        let mut pos_end: usize = self.tokens[self.pos].pos_end;
        loop {
            let next_tk = match self.next() {
                Some(tk) => tk,
                None if !allow_eof => return Err(self.error("Unterminated dotted path")),
                _ => break,
            };
            pos_end = next_tk.pos_end;
            let ast_val = match &next_tk.value {
                TokenVal::Atom(name) => AstVal::Identifier(name),
                TokenVal::StringLike { parts, is_path } if !is_path => {
                    self.parse_str_like(parts, false)?
                }
                _ => {
                    return Err(
                        self.error("Dotted path only supports identifier or string attributes")
                    )
                }
            };
            items.push(ast_val);
            // expected to be a dot
            let dot_tk = match self.peek() {
                Some(tk) => &tk.value,
                None => {
                    return Err(self.error("Unterminated dotted path"));
                }
            };
            if !matches!(dot_tk, TokenVal::Dot) {
                break;
            }
            self.next();
        }
        Ok((items, pos_end))
    }

    pub(super) fn parse_str_like(
        &self,
        parts: &[StringLikeTokenPart<'src>],
        is_path: bool,
    ) -> ParserRes<AstVal<'src>> {
        let mut out_parts: Vec<StringLikeASTPart<'src>> = Vec::new();
        for tk_part in parts {
            let out_part = match tk_part {
                StringLikeTokenPart::Text(txt) => StringLikeASTPart::Text(txt.clone()),
                StringLikeTokenPart::Expr(part_tks) => {
                    // This won't give exact end positions, but that is fine for now!!
                    let err_empty = self.error("String interpolation must not be empty");
                    let err_many =
                        self.error("String interpolation must contain exactly one expression");
                    let mut part_ast = Parser::new(part_tks).parse()?;
                    match part_ast.len() {
                        1 => StringLikeASTPart::Expr(Box::new(part_ast.pop().unwrap().value)),
                        0 => return Err(err_empty),
                        _ => return Err(err_many),
                    }
                }
            };
            out_parts.push(out_part);
        }
        let out = AstVal::StringLike {
            parts: out_parts,
            is_path,
        };
        Ok(out)
    }
}
