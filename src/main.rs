mod compiler;
mod nix;
mod parser;
mod tokenizer;

use nix::NixExpr;
use parser::Parser;
use tokenizer::Tokenizer;

fn test_nix_compile() {
    let expr_1 = NixExpr::List(vec![
        NixExpr::String("Salve galera".to_string()),
        NixExpr::Lambda {
            arg: "pkgs".to_string(),
            body: Box::new(NixExpr::AttrSet(vec![(
                "salved".to_string(),
                NixExpr::Bool(true),
            )])),
        },
        NixExpr::String("Salve galera denovo".to_string()),
    ]);
    println!("expr_1:\n{}\n---", expr_1.render());
}

fn main() {
    test_nix_compile();
}
