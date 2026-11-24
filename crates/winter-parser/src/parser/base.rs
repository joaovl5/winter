use pest::Parser;
pub use pest::error::Error;
pub use pest::iterators::Pairs;

#[derive(pest_derive::Parser)]
#[grammar = "parser/grammar.pest"]
struct _Parser;

pub struct WinterParser;

impl WinterParser {
  pub fn parse(val: &str) -> Result<Pairs<'_, Rule>, Error<Rule>> {
    _Parser::parse(Rule::file, val)
  }
}
