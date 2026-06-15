mod compiler;
mod nix;
mod parser;
mod semantics;
mod tokenizer;
mod utils;

use crate::compiler::compile_source;

fn test_nix_compile() {
    compile_source(todo!());
}

fn main() {
    test_nix_compile();
}
