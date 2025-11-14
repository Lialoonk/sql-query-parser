use pest::Parser;
use pest_derive::Parser;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

pub use pest::iterators::Pairs;

/// Main SQL parser struct using pest grammar
#[derive(Parser)]
#[grammar = "grammar/grammar.pest"]
pub struct SqlParser;

/// Metadata extracted from SQL query parsing containing tables, columns, functions, etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
pub struct QueryMetadata {
    /// Set of table names referenced in the query
    pub tables: HashSet<String>,
    /// Set of column names referenced in the query
    pub columns: HashSet<String>,
    /// Map of table/column aliases (alias -> original name)
    pub aliases: HashMap<String, String>,
    /// Set of function names used in the query
    pub functions: HashSet<String>,
    /// Set of aggregate function names (SUM, COUNT, AVG, etc.)
    pub aggregates: HashSet<String>,
    /// List of JOIN operations with their details
    pub joins: Vec<JoinInfo>,
}

/// Information about a JOIN operation in the query
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct JoinInfo {
    /// Type of JOIN (INNER, LEFT, RIGHT, FULL, etc.)
    pub join_type: Option<String>,
    /// Name of the joined table
    pub table: String,
    /// Optional alias for the joined table
    pub alias: Option<String>,
    /// ON condition for the JOIN
    pub condition: String,
}

/// Parse SQL query and return the parse tree
///
/// # Arguments
/// * `input` - SQL query string to parse
///
/// # Returns
/// Parse tree pairs on success, or parsing error
#[allow(clippy::result_large_err)]
pub fn parse_sql(
    input: &str,
) -> Result<pest::iterators::Pairs<'_, Rule>, pest::error::Error<Rule>> {
    SqlParser::parse(Rule::sql, input)
}

/// Analyze SQL query and extract metadata (tables, columns, functions, etc.)
///
/// # Arguments
/// * `input` - SQL query string to analyze
///
/// # Returns
/// QueryMetadata struct with extracted information, or parsing error
#[allow(clippy::result_large_err)]
pub fn analyze_sql(input: &str) -> Result<QueryMetadata, pest::error::Error<Rule>> {
    let pairs = SqlParser::parse(Rule::sql, input)?;
    let mut metadata = QueryMetadata::default();

    analyze_pairs(pairs, &mut metadata);

    Ok(metadata)
}

/// Analyze SQL query and return metadata as pretty-printed JSON
///
/// # Arguments
/// * `input` - SQL query string to analyze
///
/// # Returns
/// JSON string with query metadata, or parsing/serialization error
#[allow(clippy::result_large_err)]
pub fn analyze_sql_json(input: &str) -> Result<String, pest::error::Error<Rule>> {
    let metadata = analyze_sql(input)?;
    let json = serde_json::to_string_pretty(&metadata).map_err(|e| {
        pest::error::Error::new_from_span(
            pest::error::ErrorVariant::CustomError {
                message: format!("JSON serialization error: {}", e),
            },
            pest::Span::new(input, 0, input.len()).unwrap(),
        )
    })?;
    Ok(json)
}

/// Recursively analyze parse tree pairs and extract metadata
fn analyze_pairs(pairs: pest::iterators::Pairs<Rule>, metadata: &mut QueryMetadata) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::statement => analyze_pairs(pair.into_inner(), metadata),
            Rule::select_stmt => analyze_select_stmt(pair.into_inner(), metadata),
            Rule::insert_stmt => analyze_insert_stmt(pair.into_inner(), metadata),
            Rule::update_stmt => analyze_update_stmt(pair.into_inner(), metadata),
            Rule::delete_stmt => analyze_delete_stmt(pair.into_inner(), metadata),
            _ => analyze_pairs(pair.into_inner(), metadata),
        }
    }
}

/// Analyze SELECT statement components
fn analyze_select_stmt(pairs: pest::iterators::Pairs<Rule>, metadata: &mut QueryMetadata) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::from_item => analyze_from_item(pair.into_inner(), metadata),
            Rule::join_clause => analyze_join_clause(pair.into_inner(), metadata),
            Rule::projection => analyze_projection(pair.into_inner(), metadata),
            Rule::where_clause => analyze_where_clause(pair.into_inner(), metadata),
            _ => analyze_pairs(pair.into_inner(), metadata),
        }
    }
}

/// Analyze FROM clause items
fn analyze_from_item(pairs: pest::iterators::Pairs<Rule>, metadata: &mut QueryMetadata) {
    for pair in pairs {
        if let Rule::table_factor = pair.as_rule() {
            analyze_table_factor(pair.into_inner(), metadata);
        }
    }
}

/// Analyze table references and their aliases
fn analyze_table_factor(pairs: pest::iterators::Pairs<Rule>, metadata: &mut QueryMetadata) {
    let mut table_name = None;
    let mut alias = None;

    for pair in pairs {
        match pair.as_rule() {
            Rule::identifier if table_name.is_none() => {
                table_name = Some(pair.as_str().to_string());
            }
            Rule::identifier => {
                alias = Some(pair.as_str().to_string());
            }
            Rule::alias_identifier => {
                alias = Some(pair.as_str().to_string());
            }
            _ => analyze_pairs(pair.into_inner(), metadata),
        }
    }

    if let Some(table) = table_name {
        metadata.tables.insert(table.clone());
        if let Some(alias_name) = alias {
            metadata.aliases.insert(alias_name, table);
        }
    }
}

/// Analyze JOIN clauses and extract join information
fn analyze_join_clause(pairs: pest::iterators::Pairs<Rule>, metadata: &mut QueryMetadata) {
    let mut join_type = None;
    let mut table = None;
    let mut alias = None;
    let mut condition = String::new();

    for pair in pairs {
        match pair.as_rule() {
            Rule::JOIN_TYPE => join_type = Some(pair.as_str().to_string()),
            Rule::table_factor => {
                for inner_pair in pair.into_inner() {
                    match inner_pair.as_rule() {
                        Rule::identifier if table.is_none() => {
                            table = Some(inner_pair.as_str().to_string());
                        }
                        Rule::identifier => {
                            alias = Some(inner_pair.as_str().to_string());
                        }
                        Rule::alias_identifier => {
                            alias = Some(inner_pair.as_str().to_string());
                        }
                        _ => analyze_pairs(inner_pair.into_inner(), metadata),
                    }
                }
            }
            Rule::ON_KEY => {}
            _ => {
                condition = pair.as_str().to_string();
                analyze_expression_for_metadata(pair.into_inner(), metadata);
            }
        }
    }

    if let Some(table_name) = table {
        if let Some(alias_name) = alias.clone() {
            metadata.aliases.insert(alias_name, table_name.clone());
        }

        metadata.joins.push(JoinInfo {
            join_type,
            table: table_name,
            alias,
            condition,
        });
    }
}

/// Analyze SELECT projection (column list or *)
fn analyze_projection(pairs: pest::iterators::Pairs<Rule>, metadata: &mut QueryMetadata) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::projection_list => {
                for item in pair.into_inner() {
                    if let Rule::projection_item = item.as_rule() {
                        analyze_projection_item(item.into_inner(), metadata);
                    }
                }
            }
            _ => analyze_pairs(pair.into_inner(), metadata),
        }
    }
}

/// Analyze individual projection items (columns, expressions)
fn analyze_projection_item(pairs: pest::iterators::Pairs<Rule>, metadata: &mut QueryMetadata) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::expr => analyze_expression_for_metadata(pair.into_inner(), metadata),
            _ => analyze_pairs(pair.into_inner(), metadata),
        }
    }
}

/// Analyze WHERE clause expressions
fn analyze_where_clause(pairs: pest::iterators::Pairs<Rule>, metadata: &mut QueryMetadata) {
    for pair in pairs {
        if let Rule::expr = pair.as_rule() {
            analyze_expression_for_metadata(pair.into_inner(), metadata);
        }
    }
}

/// Extract metadata from expressions (columns, functions, tables)
fn analyze_expression_for_metadata(
    pairs: pest::iterators::Pairs<Rule>,
    metadata: &mut QueryMetadata,
) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::column => {
                metadata.columns.insert(pair.as_str().to_string());
            }
            Rule::function_call => {
                let func_name = pair.as_str().split('(').next().unwrap_or("").to_string();
                metadata.functions.insert(func_name.clone());

                let aggregates = ["SUM", "COUNT", "AVG", "MIN", "MAX"];
                if aggregates.contains(&func_name.to_uppercase().as_str()) {
                    metadata.aggregates.insert(func_name);
                }
            }
            Rule::identifier => {
                if !metadata.aliases.contains_key(pair.as_str()) {
                    metadata.tables.insert(pair.as_str().to_string());
                }
            }
            _ => analyze_expression_for_metadata(pair.into_inner(), metadata),
        }
    }
}

/// Analyze INSERT statements
fn analyze_insert_stmt(pairs: pest::iterators::Pairs<Rule>, metadata: &mut QueryMetadata) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::identifier => {
                metadata.tables.insert(pair.as_str().to_string());
            }
            Rule::expr => {
                analyze_expression_for_metadata(pair.into_inner(), metadata);
            }
            _ => analyze_pairs(pair.into_inner(), metadata),
        }
    }
}

/// Analyze UPDATE statements
fn analyze_update_stmt(pairs: pest::iterators::Pairs<Rule>, metadata: &mut QueryMetadata) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::identifier => {
                metadata.tables.insert(pair.as_str().to_string());
            }
            Rule::set_list => {
                analyze_set_list(pair.into_inner(), metadata);
            }
            Rule::where_clause => {
                analyze_where_clause(pair.into_inner(), metadata);
            }
            _ => analyze_pairs(pair.into_inner(), metadata),
        }
    }
}

/// Analyze DELETE statements
fn analyze_delete_stmt(pairs: pest::iterators::Pairs<Rule>, metadata: &mut QueryMetadata) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::identifier => {
                metadata.tables.insert(pair.as_str().to_string());
            }
            Rule::where_clause => {
                analyze_where_clause(pair.into_inner(), metadata);
            }
            _ => analyze_pairs(pair.into_inner(), metadata),
        }
    }
}

/// Analyze SET clause in UPDATE statements
fn analyze_set_list(pairs: pest::iterators::Pairs<Rule>, metadata: &mut QueryMetadata) {
    for pair in pairs {
        if let Rule::set_item = pair.as_rule() {
            analyze_set_item(pair.into_inner(), metadata);
        }
    }
}

/// Analyze individual SET items (column = value)
fn analyze_set_item(pairs: pest::iterators::Pairs<Rule>, metadata: &mut QueryMetadata) {
    for pair in pairs {
        match pair.as_rule() {
            Rule::identifier => {
                metadata.columns.insert(pair.as_str().to_string());
            }
            Rule::expr => {
                analyze_expression_for_metadata(pair.into_inner(), metadata);
            }
            _ => analyze_pairs(pair.into_inner(), metadata),
        }
    }
}
