use std::env;

fn main() {
    let query = env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("SELECT id, name FROM users WHERE id = 1"));

    match sql_query_parser::parse_sql(&query) {
        Ok(pairs) => {
            println!("Parsed: {:?}", pairs);
        }
        Err(error) => {
            eprintln!("Failed to parse query: {error}");
            std::process::exit(1);
        }
    }

    match sql_query_parser::analyze_sql(&query) {
        Ok(metadata) => {
            println!("{:?}", metadata);
        }
        Err(error) => {
            eprintln!("Failed to analyze query: {error}");
            std::process::exit(1);
        }
    }

    match sql_query_parser::analyze_sql_json(&query) {
        Ok(json) => {
            println!("{}", json);
        }
        Err(error) => {
            eprintln!("Failed to generate JSON: {error}");
            std::process::exit(1);
        }
    }
}
