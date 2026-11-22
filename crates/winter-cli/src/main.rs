#![allow(unused)]
use std::fs;
use std::path::PathBuf;

use anyhow::bail;
use clap::{Parser, Subcommand, ValueHint};
use winter_parser::lexer::Lexer;
use winter_parser::types::WinterAlloc;

use self::pprint::PrettyPrintable;

mod logging;
mod pprint;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct CliArgs {
  #[command(subcommand)]
  command: CliCommand,
}

#[derive(Subcommand, Debug)]
enum CliCommand {
  Compile {
    #[arg(short, long, value_hint = ValueHint::AnyPath)]
    path: PathBuf,
  },
  Lex {
    #[arg(short, long, value_hint = ValueHint::AnyPath)]
    path: PathBuf,
  },
}

fn main() -> anyhow::Result<()> {
  human_panic::setup_panic!();
  logging::setup_logging();
  let args = CliArgs::parse();

  match args.command {
    | CliCommand::Compile { path } => println!("Compiling {path:?}!"),
    | CliCommand::Lex { path } => {
      let content = fs::read_to_string(path.clone())?;
      println!("\n-- Lexing {path:?} --\n");
      // let alloc = WinterAlloc::new();
      let lexer = Lexer::new(&content);
      for (lex_r, _lex_span) in lexer.spanned() {
        let Ok(lex) = lex_r else {
          println!("\nLexer failed!");
          break;
        };
        print!("{} ", lex);
      }
      println!();
    },
  }

  anyhow::Ok(())
}
