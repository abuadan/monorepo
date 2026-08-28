use super::{
    AstStr,
    expr::Expr,
    query::{Query, SelectStatement},
};
use crate::span::{ByteSpan, Spanned};

#[derive(Debug, Clone, PartialEq, Eq)]
/// The root AST node for any SQL statement.
///
/// Wraps all supported operations including Queries and DML (Data Manipulation Language) statements.
pub enum Statement<'a> {
    Query(Box<Query<'a>>),
    Insert(Box<Query<'a>>),
    Update(UpdateStatement<'a>),
    Create(CreateStatement<'a>),
    Alter(AlterStatement<'a>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InsertStatement<'a> {
    pub from: Vec<Vec<Spanned<AstStr<'a>>>>,
    pub target: Vec<Spanned<AstStr<'a>>>,
    pub partitioned_by: Vec<Spanned<AstStr<'a>>>,
    pub source: SelectStatement<'a>,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateStatement<'a> {
    pub table: Vec<Spanned<AstStr<'a>>>,
    pub assignments: Vec<Assignment<'a>>,
    pub selection: Option<Expr<'a>>,
    pub returning: Vec<Expr<'a>>,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Assignment<'a> {
    pub column: Spanned<AstStr<'a>>,
    pub value: Expr<'a>,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateStatement<'a> {
    pub or_replace: bool,
    pub object_type: Spanned<AstStr<'a>>,
    pub if_not_exists: bool,
    pub name: Vec<Spanned<AstStr<'a>>>,
    pub definition: CreateDefinition<'a>,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlterStatement<'a> {
    pub object_type: Spanned<AstStr<'a>>,
    pub name: Vec<Spanned<AstStr<'a>>>,
    pub action: AlterAction<'a>,
    pub span: ByteSpan,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateDefinition<'a> {
    Columns(Vec<ColumnDef<'a>>),
    AsQuery(Box<Query<'a>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlterAction<'a> {
    AddColumn(ColumnDef<'a>),
    RenameTo(Vec<Spanned<AstStr<'a>>>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ColumnDef<'a> {
    pub name: Spanned<AstStr<'a>>,
    pub data_type: Spanned<AstStr<'a>>,
    pub span: ByteSpan,
}
