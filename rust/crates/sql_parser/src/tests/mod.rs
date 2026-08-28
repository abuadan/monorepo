use crate::query::{parse_query, parse_statement};
use sql_core::ast::{
    BinaryOp, Expr, JoinConstraint, JoinOperator, SelectStatement, SortDirection, Statement,
    TableFactor, UnaryOp,
};

mod expressions;
mod malformed;
mod valid;

fn select(sql: &'static str) -> SelectStatement<'static> {
    let query = parse_query(sql).expect("expected valid SQL");
    query.body.as_select().expect("expected SELECT body").clone()
}

fn assert_parse_err(sql: &'static str) {
    assert!(
        parse_query(sql).is_err(),
        "expected parse error for: {:?}",
        sql
    );
}
