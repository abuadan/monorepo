use super::{
    AstStr,
    dml::{InsertStatement, UpdateStatement},
    expr::Expr,
};
use crate::span::{ByteSpan, Spanned};
use std::ops::Deref;

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents a complete SQL Query, potentially including Common Table Expressions (CTEs).
///
/// This is the top-level AST node for any query operation.
pub struct Query<'a> {
    pub with: Vec<Cte<'a>>,
    pub body: QueryBody<'a>,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Cte<'a> {
    pub name: Spanned<AstStr<'a>>,
    pub query: Box<CteBody<'a>>,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryBody<'a> {
    Select(SelectStatement<'a>),
    Insert(InsertStatement<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CteBody<'a> {
    Query(Box<Query<'a>>),
    Update(UpdateStatement<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
/// Represents a standard `SELECT` statement in SQL.
///
/// Contains clauses for projection, filtering (`WHERE`), grouping, sorting, and limiting.
pub struct SelectStatement<'a> {
    pub distinct: bool,
    pub projection: Vec<SelectItem<'a>>,
    pub from: Vec<TableReference<'a>>,
    pub joins: Vec<Join<'a>>,
    pub selection: Option<Expr<'a>>,
    pub group_by: Vec<Expr<'a>>,
    pub having: Option<Expr<'a>>,
    pub order_by: Vec<OrderByExpr<'a>>,
    pub limit: Option<Expr<'a>>,
    pub offset: Option<Expr<'a>>,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectItem<'a> {
    pub expr: Expr<'a>,
    pub alias: Option<Spanned<AstStr<'a>>>,
    pub wildcard_options: WildcardOptions<'a>,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WildcardOptions<'a> {
    pub ilike: Option<AstStr<'a>>,
    pub exclude: Vec<Vec<Spanned<AstStr<'a>>>>,
    pub replace: Vec<ReplaceItem<'a>>,
    pub rename: Vec<RenameItem<'a>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplaceItem<'a> {
    pub expr: Expr<'a>,
    pub column: Spanned<AstStr<'a>>,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenameItem<'a> {
    pub from: Spanned<AstStr<'a>>,
    pub to: Spanned<AstStr<'a>>,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableReference<'a> {
    pub factor: TableFactor<'a>,
    pub alias: Option<Spanned<AstStr<'a>>>,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Join<'a> {
    pub operator: JoinOperator,
    pub relation: TableReference<'a>,
    pub constraint: JoinConstraint<'a>,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JoinOperator {
    Inner,
    Left,
    Right,
    Full,
    Cross,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinConstraint<'a> {
    On(Expr<'a>),
    Using(Vec<Spanned<AstStr<'a>>>),
    None,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OrderByExpr<'a> {
    pub expr: Expr<'a>,
    pub direction: Option<SortDirection>,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortDirection {
    Asc,
    Desc,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TableFactor<'a> {
    Named {
        name: Vec<Spanned<AstStr<'a>>>,
    },
    Stage {
        location: Spanned<AstStr<'a>>,
    },
    Function {
        name: Spanned<AstStr<'a>>,
        args: Vec<Expr<'a>>,
        with_ordinality: bool,
    },
    Unnest {
        args: Vec<Expr<'a>>,
        with_ordinality: bool,
    },
    Derived {
        subquery: Box<Query<'a>>,
    },
}

impl<'a> QueryBody<'a> {
    pub fn as_select(&self) -> Option<&SelectStatement<'a>> {
        match self {
            Self::Select(select) => Some(select),
            Self::Insert(_) => None,
        }
    }
}

impl<'a> CteBody<'a> {
    pub fn as_query(&self) -> Option<&Query<'a>> {
        match self {
            Self::Query(query) => Some(query),
            Self::Update(_) => None,
        }
    }

    pub fn as_select(&self) -> Option<&SelectStatement<'a>> {
        self.as_query()?.body.as_select()
    }
}

impl<'a> Deref for QueryBody<'a> {
    type Target = SelectStatement<'a>;

    fn deref(&self) -> &Self::Target {
        self.as_select()
            .expect("query body is not a SELECT statement")
    }
}
