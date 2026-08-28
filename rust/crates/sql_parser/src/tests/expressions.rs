use super::*;

#[test]
fn deeply_nested_parentheses() {
    let s = select("SELECT ((((1 + 2))))");
    assert!(matches!(s.projection[0].expr, Expr::Binary { .. }));
}

#[test]
fn chained_comparisons_via_and() {
    let s = select("SELECT * FROM t WHERE a > 0 AND a < 100");
    assert!(matches!(
        s.selection.as_ref().expect("where"),
        Expr::Binary { op: BinaryOp::And, .. }
    ));
}

#[test]
fn complex_boolean_expression() {
    let s = select("SELECT * FROM t WHERE (a = 1 OR b = 2) AND c = 0");
    assert!(matches!(
        s.selection.as_ref().expect("where"),
        Expr::Binary { op: BinaryOp::And, .. }
    ));
}

#[test]
fn is_not_null_not_yet_supported() {
    assert_parse_err("SELECT * FROM t WHERE c IS NOT NULL");
}

#[test]
fn multiple_aggregate_functions_in_projection() {
    let s = select("SELECT COUNT(*), SUM(amount), AVG(score), MIN(age), MAX(age) FROM t");
    assert_eq!(s.projection.len(), 5);
}

#[test]
fn cast_with_precision() {
    let s = select("SELECT CAST(price AS NUMERIC(10)) FROM items");
    assert!(matches!(s.projection[0].expr, Expr::Cast { .. }));
}

#[test]
fn nested_subquery_in_where() {
    let s = select("SELECT * FROM t WHERE score > (SELECT AVG(score) FROM t)");
    assert!(s.selection.is_some());
}

#[test]
fn string_literal_in_expression() {
    let s = select("SELECT 'hello world' AS greeting");
    assert!(matches!(s.projection[0].expr, Expr::Literal { .. }));
}

#[test]
fn true_false_literals() {
    let s = select("SELECT * FROM t WHERE active = TRUE AND deleted = FALSE");
    assert!(s.selection.is_some());
}

#[test]
fn null_literal() {
    let s = select("SELECT NULL AS nothing");
    assert!(matches!(s.projection[0].expr, Expr::Literal { .. }));
}

#[test]
fn in_list_with_strings() {
    let s = select("SELECT * FROM t WHERE status IN ('active', 'pending', 'review')");
    if let Expr::InList { items, .. } = s.selection.as_ref().expect("where") {
        assert_eq!(items.len(), 3);
    } else {
        panic!("expected InList");
    }
}

#[test]
fn empty_in_list_is_err() {
    assert_parse_err("SELECT * FROM t WHERE id IN ()");
}
