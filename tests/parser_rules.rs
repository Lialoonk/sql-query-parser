use anyhow::{Context, Result};
use pest::Parser;
use sql_query_parser::{Rule, SqlParser};

fn assert_rule(rule: Rule, input: &str) -> Result<()> {
    SqlParser::parse(rule, input)
        .with_context(|| format!("Failed to parse '{input}' as {:?}", rule))?;
    Ok(())
}

fn assert_rule_fails(rule: Rule, input: &str) {
    if SqlParser::parse(rule, input).is_ok() {
        panic!("Expected rule {:?} to reject input: {input}", rule);
    }
}

#[test]
fn all_grammar_rules_test() -> Result<()> {
    let cases = [
        (Rule::WHITESPACE, " "),
        (Rule::NEWLINE, "\n"),
        (Rule::COMMENT, "-- demo\n"),
        (Rule::sql, "SELECT id FROM users"),
        (Rule::statement, "SELECT id FROM users"),
        (
            Rule::compound_select,
            "SELECT id FROM users UNION SELECT id FROM posts",
        ),
        (Rule::union_clause, "UNION SELECT id FROM users"),
        (Rule::select_stmt, "SELECT id FROM users WHERE id = 1"),
        (Rule::insert_stmt, "INSERT INTO users VALUES (1)"),
        (
            Rule::update_stmt,
            "UPDATE users SET name = 'John' WHERE id = 1",
        ),
        (Rule::delete_stmt, "DELETE FROM users WHERE id = 1"),
        (Rule::column_list, "(id, name)"),
        (Rule::value_rows, "(1),(2)"),
        (Rule::value_row, "(1, 2)"),
        (Rule::set_list, "name = 1, age = 2"),
        (Rule::set_item, "name = 1"),
        (Rule::distinct, "DISTINCT"),
        (Rule::projection, "*"),
        (Rule::projection_list, "id, name"),
        (Rule::projection_item, "COUNT(id) AS total"),
        (Rule::from_item, "users u"),
        (Rule::table_factor, "users AS u"),
        (
            Rule::join_clause,
            "JOIN posts p ON u.id = p.user_id AND p.user_id = u.id",
        ),
        (Rule::where_clause, "WHERE id = 1"),
        (Rule::group_by_clause, "GROUP BY id, name"),
        (Rule::having_clause, "HAVING COUNT(id) > 1"),
        (Rule::order_by_clause, "ORDER BY id DESC, name"),
        (Rule::limit_clause, "LIMIT 10"),
        (Rule::order_list, "id DESC, name"),
        (Rule::order_item, "id DESC"),
        (Rule::identifier_list, "id, name, age"),
        (Rule::expr_list, "id, 1, func(2)"),
        (Rule::expr, "id + 1"),
        (Rule::or_expr, "id = 1 OR name = 'a'"),
        (Rule::and_expr, "id = 1 AND name = 'a'"),
        (Rule::not_expr, "NOT id = 1"),
        (Rule::comparison, "id = 1"),
        (Rule::comparison_suffix, "= 1"),
        (Rule::in_rhs, "1, 2"),
        (Rule::comp_op, "="),
        (Rule::addition, "1 + 2 - 3"),
        (Rule::multiplication, "1 * 2 / 3"),
        (Rule::unary, "-id"),
        (Rule::primary, "(1)"),
        (Rule::function_call, "func(1, 2)"),
        (Rule::column, "users.id"),
        (Rule::literal, "'abc'"),
        (Rule::boolean, "TRUE"),
        (Rule::number, "-42"),
        (Rule::string, "'abc'"),
        (Rule::alias, "alias_name"),
        (Rule::identifier, "table_name"),
        (Rule::SELECT_KEY, "SELECT"),
        (Rule::FROM_KEY, "FROM"),
        (Rule::WHERE_KEY, "WHERE"),
        (Rule::GROUP_KEY, "GROUP"),
        (Rule::BY_KEY, "BY"),
        (Rule::HAVING_KEY, "HAVING"),
        (Rule::ORDER_KEY, "ORDER"),
        (Rule::LIMIT_KEY, "LIMIT"),
        (Rule::AS_KEY, "AS"),
        (Rule::JOIN_KEY, "JOIN"),
        (Rule::INNER_KEY, "INNER"),
        (Rule::LEFT_KEY, "LEFT"),
        (Rule::RIGHT_KEY, "RIGHT"),
        (Rule::FULL_KEY, "FULL"),
        (Rule::USING_KEY, "USING"),
        (Rule::ON_KEY, "ON"),
        (Rule::DISTINCT_KEY, "DISTINCT"),
        (Rule::ASC_KEY, "ASC"),
        (Rule::DESC_KEY, "DESC"),
        (Rule::AND_KEY, "AND"),
        (Rule::OR_KEY, "OR"),
        (Rule::NOT_KEY, "NOT"),
        (Rule::LIKE_KEY, "LIKE"),
        (Rule::TRUE_KEY, "TRUE"),
        (Rule::FALSE_KEY, "FALSE"),
        (Rule::NULL_KEY, "NULL"),
        (Rule::INSERT_KEY, "INSERT"),
        (Rule::INTO_KEY, "INTO"),
        (Rule::VALUES_KEY, "VALUES"),
        (Rule::UPDATE_KEY, "UPDATE"),
        (Rule::SET_KEY, "SET"),
        (Rule::DELETE_KEY, "DELETE"),
        (Rule::UNION_KEY, "UNION"),
        (Rule::ALL_KEY, "ALL"),
        (Rule::BETWEEN_KEY, "BETWEEN"),
        (Rule::IN_KEY, "IN"),
        (Rule::IS_KEY, "IS"),
        (Rule::JOIN_TYPE, "LEFT OUTER"),
        (Rule::OUTER_KEY, "OUTER"),
        (Rule::SPACE, " "),
    ];

    for (rule, input) in cases {
        assert_rule(rule, input)?;
    }

    Ok(())
}

#[test]
fn select_stmt_with_join_and_filters() -> Result<()> {
    assert_rule(
        Rule::select_stmt,
        "SELECT DISTINCT u.id, name \
         FROM users u JOIN posts p ON u.id = p.user_id \
         WHERE p.kind = 'blog' \
         GROUP BY u.id, name \
         HAVING COUNT(p.id) > 1 \
         ORDER BY u.id DESC \
         LIMIT 10",
    )
}

#[test]
fn insert_update_delete_statements() -> Result<()> {
    assert_rule(Rule::insert_stmt, "INSERT INTO metrics VALUES (sum(value))")?;
    assert_rule(
        Rule::update_stmt,
        "UPDATE users SET name = 'Alice', age = 42 WHERE id = 10",
    )?;
    assert_rule(
        Rule::delete_stmt,
        "DELETE FROM audit_logs WHERE created_at < '2024-01-01'",
    )?;
    Ok(())
}

#[test]
fn invalid_insert_syntax_is_rejected() {
    assert_rule_fails(Rule::insert_stmt, "INSERT INTO users VALUES");
}

#[test]
fn incomplete_where_expression_is_rejected() {
    assert_rule_fails(Rule::where_clause, "WHERE )");
}