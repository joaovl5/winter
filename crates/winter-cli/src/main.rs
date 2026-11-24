use std::fs;
use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint};
use winter_parser::parser::WinterParser;
use winter_parser::types::WinterAlloc;

mod logging;

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
      let _alloc = WinterAlloc::new();

      let parse_result = WinterParser::parse(&content);
      pest_ascii_tree::print_ascii_tree(parse_result);
    },
  }

  anyhow::Ok(())
}
