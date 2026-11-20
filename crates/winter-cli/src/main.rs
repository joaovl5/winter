#![feature(string_into_chars)]

use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint};
use winter_parser::lexer::Lexer;

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
      let content = fs::read_to_string(path)?.into_chars();
      let mut lexer = Lexer::new(content);
      let lexer_iter = lexer.parse();
      for lex_r in lexer_iter {
        let Some(lex) = lex_r? else {
          break;
        };
        lex.1.pprint();
      }
    },
  }

  anyhow::Ok(())
}
