use pest_derive::Parser;
use pest::Parser;

#[derive(Parser)]
#[grammar = "grammar/grammar.pest"]
pub struct SQLParser;

pub fn parse_sql(input: &str) -> Result<(), pest::error::Error<Rule>> {
    let _pairs = SQLParser::parse(Rule::program, input)?;
    Ok(())
}
