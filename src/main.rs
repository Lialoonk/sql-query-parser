use std::env;

fn main() {
    let query = env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("SELECT id, name FROM users WHERE id = 1"));

    match sql_query_parser::parse_sql(&query) {
        Ok(mut pairs) => {
            println!("Parsed successfully: {}", query);
            if let Some(pair) = pairs.next() {
                println!("Root rule: {:?}", pair.as_rule());
            }
        }
        Err(error) => {
            eprintln!("Failed to parse query: {error}");
            std::process::exit(1);
        }
    }
}
