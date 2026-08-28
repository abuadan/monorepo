use super::*;

#[test]
fn empty_input() {
    assert_parse_err("");
}

#[test]
fn only_whitespace() {
    assert_parse_err("   ");
}

#[test]
fn select_without_projection() {
    assert_parse_err("SELECT FROM users");
}

#[test]
fn select_without_from_still_valid() {
    parse_query("SELECT 1 + 1").expect("SELECT without FROM is valid");
}

#[test]
fn unmatched_opening_parenthesis() {
    assert_parse_err("SELECT (1 + 2 FROM t");
}

#[test]
fn unmatched_closing_parenthesis() {
    assert_parse_err("SELECT 1 + 2) FROM t");
}

#[test]
fn unclosed_subquery_in_from() {
    assert_parse_err("SELECT * FROM (SELECT id FROM users");
}

#[test]
fn where_before_from() {
    assert_parse_err("SELECT * WHERE id = 1 FROM t");
}

#[test]
fn having_without_group_by_is_accepted() {
    parse_query("SELECT dept FROM t HAVING COUNT(*) > 5")
        .expect("HAVING without GROUP BY is accepted by this grammar");
}

#[test]
fn order_by_before_where() {
    assert_parse_err("SELECT * FROM t ORDER BY id WHERE id = 1");
}

#[test]
fn limit_before_where() {
    assert_parse_err("SELECT * FROM t LIMIT 10 WHERE id = 1");
}

#[test]
fn select_keyword_only() {
    assert_parse_err("SELECT");
}

#[test]
fn trailing_comma_in_projection() {
    assert_parse_err("SELECT id, FROM users");
}

#[test]
fn trailing_comma_in_from() {
    assert_parse_err("SELECT * FROM a,");
}

#[test]
fn trailing_comma_in_group_by() {
    assert_parse_err("SELECT a FROM t GROUP BY a,");
}

#[test]
fn trailing_comma_in_order_by() {
    assert_parse_err("SELECT * FROM t ORDER BY a,");
}

#[test]
fn join_without_on_or_using_is_allowed() {
    let s = select("SELECT * FROM a INNER JOIN b");
    assert_eq!(s.joins[0].operator, JoinOperator::Inner);
    assert!(matches!(s.joins[0].constraint, JoinConstraint::None));
}

#[test]
fn on_clause_without_condition() {
    assert_parse_err("SELECT * FROM a JOIN b ON");
}

#[test]
fn update_without_set() {
    assert!(
        parse_statement("UPDATE users WHERE id = 1").is_err(),
        "UPDATE without SET must fail"
    );
}

#[test]
fn update_set_without_value() {
    assert!(
        parse_statement("UPDATE t SET name =").is_err(),
        "incomplete assignment must fail"
    );
}

#[test]
fn select_is_not_a_valid_table_name_in_from() {
    assert_parse_err("SELECT * FROM SELECT");
}

#[test]
fn where_is_not_a_valid_alias() {
    assert_parse_err("SELECT * FROM t AS WHERE");
}

#[test]
fn dangling_and_operator() {
    assert_parse_err("SELECT * FROM t WHERE a = 1 AND");
}

#[test]
fn dangling_or_operator() {
    assert_parse_err("SELECT * FROM t WHERE OR b = 2");
}

#[test]
fn double_operator_without_operand() {
    assert_parse_err("SELECT * FROM t WHERE a == b");
}

#[test]
fn bare_dot_is_not_a_number() {
    assert_parse_err("SELECT . FROM t");
}

#[test]
fn with_without_body() {
    assert_parse_err("WITH cte AS (SELECT 1)");
}

#[test]
fn cte_missing_as_keyword() {
    assert_parse_err("WITH cte (SELECT 1) SELECT * FROM cte");
}

#[test]
fn cte_missing_parentheses() {
    assert_parse_err("WITH cte AS SELECT 1 SELECT * FROM cte");
}

#[test]
fn create_table_without_definition_is_rejected() {
    assert!(
        parse_statement("CREATE TABLE users").is_err(),
        "CREATE TABLE without columns or AS query must fail"
    );
}

#[test]
fn create_view_without_as_query_is_rejected() {
    assert!(
        parse_statement("CREATE VIEW users_view").is_err(),
        "CREATE VIEW without AS query must fail"
    );
}

#[test]
fn alter_table_without_action_is_rejected() {
    assert!(
        parse_statement("ALTER TABLE users").is_err(),
        "ALTER TABLE without action must fail"
    );
}

#[test]
fn insert_missing_into_keyword() {
    assert_parse_err("INSERT TABLE target SELECT * FROM src");
}

#[test]
fn create_table_missing_name() {
    assert_parse_err("CREATE TABLE (id INT)");
}

#[test]
fn alter_table_missing_name() {
    assert_parse_err("ALTER TABLE ADD COLUMN id INT");
}

