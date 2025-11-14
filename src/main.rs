use clap::{Parser, Subcommand};
use std::fs;
use std::io::{self, Read};

/// SQL Query Parser - A tool for parsing and analyzing SQL queries
#[derive(Parser)]
#[command(name = "sql-query-parser")]
#[command(about = "Parse and analyze SQL queries with metadata extraction")]
#[command(version = "0.1.0")]
#[command(disable_help_subcommand = true)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Parse {
        #[arg(short, long)]
        query: Option<String>,

        #[arg(short, long)]
        file: Option<String>,

        #[arg(long, default_value = "parse")]
        format: String,
    },
    Help,
    Credits,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { query, file, format } => {
            let sql_query = match (query, file) {
                (Some(q), None) => q,
                (None, Some(filename)) => {
                    match fs::read_to_string(&filename) {
                        Ok(content) => content,
                        Err(e) => {
                            eprintln!("Error reading file '{}': {}", filename, e);
                            std::process::exit(1);
                        }
                    }
                }
                (None, None) => {
                    let mut buffer = String::new();
                    match io::stdin().read_to_string(&mut buffer) {
                        Ok(_) => buffer.trim().to_string(),
                        Err(e) => {
                            eprintln!("Error reading from stdin: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                (Some(_), Some(_)) => {
                    eprintln!("Error: Cannot specify both --query and --file");
                    std::process::exit(1);
                }
            };

            if sql_query.trim().is_empty() {
                eprintln!("Error: No SQL query provided. Use --query, --file, or pipe input.");
                std::process::exit(1);
            }

            match format.as_str() {
                "parse" => {
                    match sql_query_parser::parse_sql(&sql_query) {
                        Ok(pairs) => {
                            println!("Parse tree:\n {:#?}", pairs);
                        }
                        Err(error) => {
                            eprintln!("Failed to parse SQL query: {}", error);
                            std::process::exit(1);
                        }
                    }
                }
                "analyze" => {
                    match sql_query_parser::analyze_sql(&sql_query) {
                        Ok(metadata) => {
                            println!("SQL Query Analysis:");
                            println!("Tables: {:?}", metadata.tables);
                            println!("Columns: {:?}", metadata.columns);
                            println!("Aliases: {:?}", metadata.aliases);
                            println!("Functions: {:?}", metadata.functions);
                            println!("Aggregates: {:?}", metadata.aggregates);
                            println!("Joins: {:?}", metadata.joins);
                        }
                        Err(error) => {
                            eprintln!("Failed to analyze SQL query: {}", error);
                            std::process::exit(1);
                        }
                    }
                }
                "json" => {
                    match sql_query_parser::analyze_sql_json(&sql_query) {
                        Ok(json) => {
                            println!("{}", json);
                        }
                        Err(error) => {
                            eprintln!("Failed to generate JSON: {}", error);
                            std::process::exit(1);
                        }
                    }
                }
                _ => {
                    eprintln!("Error: Invalid format '{}'. Use 'parse', 'analyze', or 'json'", format);
                    std::process::exit(1);
                }
            }
        }
        Commands::Help => {
            print_help();
        }
        Commands::Credits => {
            print_credits();
        }
    }
}

/// Display help information about available commands and usage
fn print_help() {
    println!("SQL Query Parser v0.1.0");
    println!("A comprehensive tool for parsing and analyzing SQL queries with metadata extraction.");
    println!();
    println!("USAGE:");
    println!("    sql-query-parser <COMMAND>");
    println!();
    println!("COMMANDS:");
    println!("    parse    Parse a SQL query and display results");
    println!("    help     Display this help information");
    println!("    credits  Display credits and project information");
    println!();
    println!("PARSE OPTIONS:");
    println!("    -q, --query <QUERY>    SQL query to parse");
    println!("    -f, --file <FILE>      Read SQL query from file");
    println!("        --format <FORMAT>  Output format: parse, analyze, or json [default: parse]");
    println!();
    println!("EXAMPLES:");
    println!("    sql-query-parser parse --query \"SELECT * FROM users\"");
    println!("    sql-query-parser parse --file query.sql --format analyze");
    println!("    echo \"SELECT * FROM users\" | sql-query-parser parse --format json");
    println!("    sql-query-parser help");
    println!("    sql-query-parser credits");
    println!();
    println!("For more information, visit: https://github.com/Lialoonk/sql-query-parser");
}

/// Display project credits and information
fn print_credits() {
    println!("SQL Query Parser v0.1.0");
    println!();
    println!("A comprehensive SQL parsing and analysis tool built with Rust.");
    println!();
    println!("DEVELOPED BY:");
    println!("    Liza Klimenko");
    println!();
    println!("FEATURES:");
    println!("    Full SQL syntax parsing (SELECT, INSERT, UPDATE, DELETE)");
    println!("    JOIN operations support");
    println!("    Metadata extraction (tables, columns, functions, aliases)");
    println!("    JSON serialization of analysis results");
    println!("    File and stdin input support");
    println!();
    println!("REPOSITORY:");
    println!("    https://github.com/Lialoonk/sql-query-parser");
}
