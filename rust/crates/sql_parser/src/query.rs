//! High-level SQL query parsing API.
//!
//! This module is the primary entry-point for external consumers of the SQL
//! parser. It exposes simple `parse_*` functions that accept a raw SQL string
//! and return a fully resolved AST or a descriptive [`ParseError`].
//!
//! # Quick Start
//!
//! ```rust
//! use sql_parser::query::parse_query;
//!
//! let sql = "SELECT id, name FROM employees WHERE department = 'eng'";
//! let query = parse_query(sql).expect("valid SQL");
//!
//! let select = query.body.as_select().unwrap();
//! assert_eq!(select.from.len(), 1);
//! ```
//!
//! # Custom Dialects
//!
//! The lower-level [`query_parser`] function accepts a [`Dialect`] reference
//! so you can plug in your own keyword-reservation rules:
//!
//! ```ignore
//! use sql_core::dialect::Dialect;
//! use sql_parser::query::query_parser;
//! use chumsky::prelude::*;
//!
//! #[derive(Debug)]
//! struct MyDialect;
//! impl Dialect for MyDialect { /* … */ }
//!
//! let dialect = MyDialect;
//! let parser  = query_parser::<_, _>(&dialect);
//! ```
//!
//! [`Dialect`]: sql_core::dialect::Dialect

use chumsky::{
    input::{Stream, ValueInput},
    prelude::*,
};

use sql_core::ast::{Cte, CteBody, Query, QueryBody, Statement};
use sql_core::error::ParseError;
use sql_core::lexer::{LexedToken, Token, tokenize_with_options};
use sql_core::options::ParserOptions;
use sql_core::span::ByteSpan;

use super::dml::{alter_parser, create_parser, insert_parser, update_parser};
use sql_core::helpers::{ParserExtra, first_error, identifier_word_parser};
use sql_expr::select_parser;

/// Parses a raw SQL string into a [`Query`] AST using the generic ANSI dialect.
///
/// This is the simplest entry-point. For dialect-specific parsing or to tune
/// lexer options, use [`parse_query_with_options`] instead.
///
/// The parser performs **zero-copy tokenisation** — all string data in the
/// returned AST borrows directly from `source`. The caller must ensure that
/// `source` outlives the returned [`Query`].
///
/// # Errors
///
/// Returns [`ParseError::UnexpectedToken`] if the SQL is syntactically invalid,
/// or [`ParseError::Eof`] if the input ends prematurely.
///
/// # Examples
///
/// ## Basic `SELECT`
///
/// ```rust
/// use sql_parser::query::parse_query;
///
/// let query = parse_query("SELECT 1").expect("valid SQL");
/// assert!(query.with.is_empty());
/// ```
///
/// ## Accessing the projection list
///
/// ```rust
/// use sql_parser::query::parse_query;
/// use sql_core::ast::Expr;
///
/// let query  = parse_query("SELECT id, name FROM users").unwrap();
/// let select = query.body.as_select().unwrap();
/// assert_eq!(select.projection.len(), 2);
/// ```
///
/// ## Handling parse errors
///
/// ```rust
/// use sql_parser::query::parse_query;
///
/// let result = parse_query("SELECT FROM");
/// assert!(result.is_err(), "invalid SQL should return an error");
/// ```
pub fn parse_query<'src>(source: &'src str) -> Result<Query<'src>, ParseError> {
    parse_query_with_options(source, &ParserOptions::default())
}

/// Parses a raw SQL string into a [`Query`] AST, with full control over parser
/// options (dialect feature-flags, alternate quoting modes, etc.).
///
/// See [`ParserOptions`] for the available configuration flags.
///
/// # Examples
///
/// ```rust
/// use sql_parser::query::parse_query_with_options;
/// use sql_core::options::ParserOptions;
///
/// let options = ParserOptions { allow_stage_refs: true, ..Default::default() };
/// let query   = parse_query_with_options("SELECT $1:stage FROM @my_stage", &options);
/// // Stage-ref syntax is accepted because allow_stage_refs is set.
/// assert!(query.is_ok());
/// ```
pub fn parse_query_with_options<'src>(
    source: &'src str,
    options: &ParserOptions,
) -> Result<Query<'src>, ParseError> {
    let tokens = lex_tokens(source, options)?;
    let parser_tokens = non_trivia_tokens(&tokens);
    let eoi = parser_tokens
        .last()
        .map(|(_, span)| span.end..span.end)
        .unwrap_or(0..0);
    let input = Stream::from_iter(parser_tokens).map(eoi, |x: (Token<'src>, ByteSpan)| x);

    query_parser(&sql_core::dialect::GENERIC_DIALECT)
        .parse(input)
        .into_result()
        .map_err(|errors| first_error(errors, &tokens))
}

/// Parses a raw SQL string into a [`Statement`] AST node.
///
/// `parse_statement` is more general than [`parse_query`]: it first tries to
/// parse the input as a `SELECT`/`INSERT`/`WITH` query, and on failure
/// retries as a DML statement (`UPDATE`, etc.).
///
/// Use this function when you do not know in advance whether the incoming SQL
/// is a query or a mutation.
///
/// # Errors
///
/// Returns a [`ParseError`] only when **both** the query and the DML parse
/// attempts fail. The error reports the *query* failure, as that is typically
/// the more descriptive of the two.
///
/// # Examples
///
/// ```rust
/// use sql_parser::query::parse_statement;
/// use sql_core::ast::Statement;
///
/// // A SELECT is returned as Statement::Query
/// let stmt = parse_statement("SELECT 1").unwrap();
/// assert!(matches!(stmt, Statement::Query(_)));
///
/// // An UPDATE is returned as Statement::Update
/// let stmt = parse_statement("UPDATE t SET col = 1 WHERE id = 42").unwrap();
/// assert!(matches!(stmt, Statement::Update(_)));
/// ```
pub fn parse_statement<'src>(source: &'src str) -> Result<Statement<'src>, ParseError> {
    parse_statement_with_options(source, &ParserOptions::default())
}

pub fn parse_statement_with_options<'src>(
    source: &'src str,
    options: &ParserOptions,
) -> Result<Statement<'src>, ParseError> {
    let tokens = lex_tokens(source, options)?;
    let parser_tokens = non_trivia_tokens(&tokens);
    let eoi = parser_tokens
        .last()
        .map(|(_, span)| span.end..span.end)
        .unwrap_or(0..0);
    let input = Stream::from_iter(parser_tokens).map(eoi, |x: (Token<'src>, ByteSpan)| x);

    statement_parser(&sql_core::dialect::GENERIC_DIALECT)
        .parse(input)
        .into_result()
        .map_err(|errors| first_error(errors, &tokens))
}

fn non_trivia_tokens<'a>(tokens: &[LexedToken<'a>]) -> Vec<LexedToken<'a>> {
    tokens
        .iter()
        .filter(|(token, _)| !token.is_trivia())
        .cloned()
        .collect()
}

fn lex_tokens<'src>(source: &'src str, options: &ParserOptions) -> Result<Vec<LexedToken<'src>>, ParseError> {
    tokenize_with_options(source, options).map_err(|err| match err {
        sql_core::error::LexError::InvalidToken { span } => ParseError::UnexpectedToken {
            found: "invalid token".to_owned(),
            span,
            labels: Vec::new(),
        },
    })
}

/// Lower-level query parser that accepts an explicit [`Dialect`][sql_core::dialect::Dialect].
///
/// Prefer [`parse_query`] for simple use-cases. Use this function when you need
/// to compose the parser with other Chumsky combinators or pass a custom dialect.
///
/// # Example
///
/// ```ignore
/// use sql_core::dialect::GENERIC_DIALECT;
/// use sql_parser::query::query_parser;
/// use chumsky::prelude::*;
///
/// let parser = query_parser::<_, _>(&GENERIC_DIALECT).then_ignore(end());
/// ```
pub fn query_parser<'src, I>(
    dialect: &'src dyn sql_core::dialect::Dialect,
) -> impl Parser<'src, I, Query<'src>, ParserExtra<'src>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = ByteSpan>,
{
    recursive(|query| {
        let word = identifier_word_parser(dialect);
        let comma = sql_macro::punct!(Comma).ignored();
        let l_paren = sql_macro::punct!(LParen).ignored();
        let r_paren = sql_macro::punct!(RParen).ignored();
        let select_stmt = select_parser(query.clone().boxed(), dialect);
        let update_stmt = update_parser(dialect);
        let insert_stmt = insert_parser(select_stmt.clone(), dialect);

        let cte = word
            .clone()
            .then_ignore(sql_macro::keyword!(AS))
            .then(
                query
                    .clone()
                    .map(|q| CteBody::Query(Box::new(q)))
                    .or(update_stmt.clone().map(CteBody::Update))
                    .delimited_by(l_paren, r_paren),
            )
            .map_with(|(name, query), extra| Cte {
                name,
                query: Box::new(query),
                span: extra.span(),
            });

        sql_macro::keyword!(WITH)
            .ignore_then(cte.separated_by(comma).at_least(1).collect::<Vec<_>>())
            .or_not()
            .then(
                insert_stmt
                    .map(QueryBody::Insert)
                    .or(select_stmt.map(QueryBody::Select)),
            )
            .map_with(|(with, body), extra| Query {
                with: with.unwrap_or_default(),
                body,
                span: extra.span(),
            })
            .boxed()
    })
}

pub fn statement_parser<'src, I>(
    dialect: &'src dyn sql_core::dialect::Dialect,
) -> impl Parser<'src, I, Statement<'src>, ParserExtra<'src>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = ByteSpan>,
{
    recursive(|_statement| {
        let query = query_parser(dialect).boxed();

        choice((
            query.clone().map(|query| match query.body {
                QueryBody::Insert(_) => Statement::Insert(Box::new(query)),
                QueryBody::Select(_) => Statement::Query(Box::new(query)),
            }),
            update_parser(dialect).map(Statement::Update),
            create_parser(query, dialect).map(Statement::Create),
            alter_parser(dialect).map(Statement::Alter),
        ))
    })
}
