use pest::Parser;
use pest_derive::Parser;

#[derive(Parser)]
#[grammar = "grammar/grammar.pest"]
pub struct SqlParser;

pub fn parse_sql(
    input: &str,
) -> Result<pest::iterators::Pairs<'_, Rule>, pest::error::Error<Rule>> {
    SqlParser::parse(Rule::sql, input)
}
