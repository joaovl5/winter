use super::*;

impl<'tks, 'src> Parser<'tks, 'src> {
    pub(super) fn parse_typed_reference(&mut self) -> ParserRes<TypedReference<'src>> {
        let name = match self
            .next()
            .ok_or_else(|| self.error("Expected a reference"))?
            .value
        {
            TokenVal::Atom(at) => at,
            _ => {
                return Err(self.error(
                    "Unexpected token found where an identifier for a reference was expected",
                ))
            }
        };
        // TODO: handle `@` type annotations later, being careful for not mistakenly
        // taking an unpacked argument syntax - which also uses the `@` sign as a type
        // annotation
        let type_val = None;
        Ok(TypedReference { name, type_val })
    }
}
