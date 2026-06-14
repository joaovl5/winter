use crate::{parser::Parser, tokenizer::Tokenizer};

pub fn compile_source(source: &str) -> Result<String, String> {
    let mut tokenizer = Tokenizer::new(source);
    let tokens = tokenizer
        .parse()
        .map_err(|err| format!("Tokenizer error at {}: {}", err.pos, err.msg))?;

    let ast = Parser::new(&tokens).parse().map_err(|err| {
        format!(
            "Parser error at {}..{}: {}",
            err.pos_start, err.pos_end, err.msg
        )
    })?;

    todo!("compiler lowering is not implemented yet")
}
