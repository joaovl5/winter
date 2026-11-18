use std::path::PathBuf;

use clap::{Parser, Subcommand, ValueHint};

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
}

fn main() -> anyhow::Result<()> {
  human_panic::setup_panic!();
  logging::setup_logging();
  let args = CliArgs::parse();

  match args.command {
    | CliCommand::Compile { path } => println!("Compiling {path:?}!"),
  }

  anyhow::Ok(())
}
