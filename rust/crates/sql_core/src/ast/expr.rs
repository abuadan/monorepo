use super::{AstStr, query::Query};
use crate::span::{ByteSpan, Spanned};

#[derive(Debug, Clone, PartialEq, Eq)]
/// An abstract syntax tree node representing a SQL expression.
///
/// Expressions evaluate to a scalar value and can be composed of literals, identifiers,
/// function calls, binary operators, and subqueries.
pub enum Expr<'a> {
    Placeholder {
        name: AstStr<'a>,
        span: ByteSpan,
    },
    Subquery {
        query: Box<Query<'a>>,
        span: ByteSpan,
    },
    Identifier {
        parts: Vec<Spanned<AstStr<'a>>>,
        span: ByteSpan,
    },
    Literal {
        value: AstStr<'a>,
        span: ByteSpan,
    },
    Wildcard {
        span: ByteSpan,
    },
    Function {
        name: Spanned<AstStr<'a>>,
        args: Vec<Expr<'a>>,
        kind: Option<AggregateKind>,
        span: ByteSpan,
    },
    Named {
        expr: Box<Expr<'a>>,
        name: Spanned<AstStr<'a>>,
        span: ByteSpan,
    },
    Array {
        elements: Vec<Expr<'a>>,
        span: ByteSpan,
    },
    FieldAccess {
        expr: Box<Expr<'a>>,
        field: Spanned<AstStr<'a>>,
        span: ByteSpan,
    },
    Cast {
        expr: Box<Expr<'a>>,
        data_type: Spanned<AstStr<'a>>,
        span: ByteSpan,
    },
    OuterJoin {
        expr: Box<Expr<'a>>,
        span: ByteSpan,
    },
    GroupingSets {
        sets: Vec<Vec<Expr<'a>>>,
        span: ByteSpan,
    },
    Index {
        expr: Box<Expr<'a>>,
        index: Box<Expr<'a>>,
        span: ByteSpan,
    },
    QualifiedWildcard {
        expr: Box<Expr<'a>>,
        span: ByteSpan,
    },
    InList {
        expr: Box<Expr<'a>>,
        items: Vec<Expr<'a>>,
        negated: bool,
        span: ByteSpan,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr<'a>>,
        span: ByteSpan,
    },
    Binary {
        left: Box<Expr<'a>>,
        op: BinaryOp,
        right: Box<Expr<'a>>,
        span: ByteSpan,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Or,
    And,
    Concat,
    Eq,
    NotEq,
    NullSafeEq,
    Lt,
    Lte,
    Gt,
    Gte,
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Pos,
    Neg,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AggregateKind {
    Count,
    Sum,
    Avg,
    Min,
    Max,
}
