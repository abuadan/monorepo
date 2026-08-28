//! SQL expression parsing via a composable, dialect-aware builder.
//!
//! This module exposes [`ExprParserBuilder`], the primary entry-point for
//! constructing a Chumsky parser that understands full SQL expression syntax:
//! literals, identifiers, function calls, binary operators, window functions,
//! subqueries, casts, array indexing, and more.
//!
//! # Extension Model
//!
//! Rather than hard-coding every operator variant, the builder accepts custom
//! "atom" parsers that are tried **before** the built-in atoms. This makes it
//! trivial to add dialect-specific syntax (e.g., PostgreSQL `->>`/`@>` JSON
//! operators) without forking the crate.
//!
//! # Example: Adding a BigQuery `SAFE_DIVIDE` Atom
//!
//! ```ignore
//! use sql_expr::ExprParserBuilder;
//! use sql_core::dialect::GENERIC_DIALECT;
//!
//! let safe_divide_atom = todo!("your chumsky parser here");
//!
//! let parser = ExprParserBuilder::new(&GENERIC_DIALECT)
//!     .with_custom_atom(safe_divide_atom)
//!     .build(subquery_parser.boxed());
//! ```

use chumsky::{input::ValueInput, prelude::*};

use sql_core::ast::{BinaryOp, Expr, Query, UnaryOp};
use sql_core::lexer::Token;
use sql_core::span::ByteSpan;

use sql_core::helpers::{
    ParserExtra, aggregate_kind, any_word_parser, expr_span, identifier_word_parser, punct,
};

/// A builder for constructing dialect-aware SQL expression parsers.
///
/// `ExprParserBuilder` is the primary mechanism for producing a fully-featured
/// SQL expression parser that respects dialect-specific reserved keywords and
/// can be extended with custom syntax atoms.
///
/// # Architecture
///
/// Internally this builds a [Pratt parser] using the Chumsky combinator library.
/// The expression hierarchy (from lowest to highest precedence) is:
///
/// ```text
/// expr
///   └─ binary  (OR, AND, =, !=, <, <=, >, >=, +, -, *, /)
///        └─ unary  (NOT, -, +)
///             └─ postfix  (field access, index, cast, outer join)
///                  └─ atom  (literal, identifier, function call, subquery,
///                            array literal, grouping sets, CAST(…), custom)
/// ```
///
/// Custom atoms registered with [`with_custom_atom`][Self::with_custom_atom]
/// are attempted **first**, allowing dialect-specific syntax to shadow the
/// built-in rules where necessary.
///
/// # Example: Extending with a Proprietary Function
///
/// ```ignore
/// use sql_expr::ExprParserBuilder;
/// use sql_core::{ast::Expr, dialect::GENERIC_DIALECT, span::ByteSpan};
/// use chumsky::prelude::*;
///
/// // A parser for Snowflake ZEROIFNULL(expr)
/// let zeroifnull = sql_macro::keyword!(ZEROIFNULL)
///     .ignore_then(
///         my_expr_parser.delimited_by(
///             sql_macro::punct!(LParen),
///             sql_macro::punct!(RParen),
///         )
///     )
///     .map_with(|inner, e| Expr::Function {
///         name: ("ZEROIFNULL".into(), e.span()),
///         args: vec![inner],
///         kind: None,
///         span: e.span(),
///     })
///     .boxed();
///
/// let expr_parser = ExprParserBuilder::new(&GENERIC_DIALECT)
///     .with_custom_atom(zeroifnull)
///     .build(subquery_parser.boxed());
/// ```
///
/// # Lifetimes
///
/// | Parameter | Meaning |
/// |-----------|----------|
/// | `'src`    | Tied to the lifetime of the source SQL string slice. All [`Expr`] nodes borrow directly from the input without copying. |
/// | `I`       | The Chumsky [`ValueInput`] token stream, constrained to emit [`Token<'src>`][sql_core::lexer::Token] tokens with [`ByteSpan`] spans. |
///
/// [Pratt parser]: https://en.wikipedia.org/wiki/Pratt_parser
pub struct ExprParserBuilder<'src, I>
where
    I: ValueInput<'src, Token = sql_core::lexer::Token<'src>, Span = ByteSpan>,
{
    dialect: &'src dyn sql_core::dialect::Dialect,
    custom_atoms: Vec<Boxed<'src, 'src, I, Expr<'src>, ParserExtra<'src>>>,
}

impl<'src, I> ExprParserBuilder<'src, I>
where
    I: ValueInput<'src, Token = sql_core::lexer::Token<'src>, Span = ByteSpan>,
{
    /// Creates a new builder configured for the given SQL dialect.
    ///
    /// Pass [`&GENERIC_DIALECT`][sql_core::dialect::GENERIC_DIALECT] for
    /// standard ANSI SQL behaviour, or supply your own [`Dialect`][sql_core::dialect::Dialect]
    /// implementation for custom keyword-reservation rules.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use sql_expr::ExprParserBuilder;
    /// use sql_core::dialect::GENERIC_DIALECT;
    ///
    /// let builder = ExprParserBuilder::new(&GENERIC_DIALECT);
    /// ```
    pub fn new(dialect: &'src dyn sql_core::dialect::Dialect) -> Self {
        Self {
            dialect,
            custom_atoms: Vec::new(),
        }
    }

    /// Registers a custom expression atom parser.
    ///
    /// Custom atoms are **tried first** in the choice chain. If the custom
    /// parser succeeds, its result is used; otherwise parsing falls through to
    /// the built-in atom rules (literals, identifiers, function calls, etc.).
    ///
    /// Call this method multiple times to register multiple atoms — they are
    /// tried in registration order.
    ///
    /// # Example
    ///
    /// ```ignore
    /// // Add support for PostgreSQL dollar-quoted constant syntax
    /// let dollar_constant = select! { Token::DollarConst(v) => v }
    ///     .map_with(|value, e| Expr::Literal { value, span: e.span() })
    ///     .boxed();
    ///
    /// let builder = ExprParserBuilder::new(&GENERIC_DIALECT)
    ///     .with_custom_atom(dollar_constant);
    /// ```
    pub fn with_custom_atom(
        mut self,
        parser: Boxed<'src, 'src, I, Expr<'src>, ParserExtra<'src>>,
    ) -> Self {
        self.custom_atoms.push(parser);
        self
    }

    /// Consumes the builder and produces a boxed expression parser.
    ///
    /// The `select` argument provides the subquery parser used inside
    /// scalar subquery expressions (`(SELECT …)`). Pass
    /// `chumsky::prelude::empty().map(|_| unreachable!()).boxed()` if
    /// subqueries are not needed in your context.
    ///
    /// # Example
    ///
    /// ```ignore
    /// use sql_expr::{ExprParserBuilder, select_parser};
    /// use sql_core::dialect::GENERIC_DIALECT;
    ///
    /// let select = select_parser(&GENERIC_DIALECT);
    /// let expr   = ExprParserBuilder::new(&GENERIC_DIALECT)
    ///     .build(select.boxed());
    /// ```
    pub fn build(
        self,
        query: Boxed<'src, 'src, I, Query<'src>, ParserExtra<'src>>,
    ) -> Boxed<'src, 'src, I, Expr<'src>, ParserExtra<'src>> {
        let dialect = self.dialect;
        let custom_atoms = self.custom_atoms;

        let word = identifier_word_parser(dialect);
        let any_word = any_word_parser();
        let comma = punct::<I>(Token::Comma).ignored();
        let dot = punct::<I>(Token::Dot).ignored();
        let colon = punct::<I>(Token::Colon).ignored();
        let l_paren = punct::<I>(Token::LParen).ignored();
        let r_paren = punct::<I>(Token::RParen).ignored();
        let l_bracket = punct::<I>(Token::LBracket).ignored();
        let r_bracket = punct::<I>(Token::RBracket).ignored();

        let qualified_name = word
            .clone()
            .separated_by(dot.clone())
            .at_least(1)
            .collect::<Vec<_>>()
            .boxed();

        recursive(|expr| {
        #[derive(Clone)]
        enum Postfix<'a> {
            Field(sql_core::span::Spanned<sql_core::ast::AstStr<'a>>),
            Index(Expr<'a>),
            Cast(sql_core::span::Spanned<sql_core::ast::AstStr<'a>>),
            OuterJoin,
            Wildcard,
        }

        let named_expr = expr
            .clone()
            .then(sql_macro::keyword!(AS).ignore_then(word.clone()))
            .map_with(|(expr, name), extra| Expr::Named {
                expr: Box::new(expr),
                name,
                span: extra.span(),
            })
            .boxed();

        let cast_data_type = any_word
            .clone()
            .then(
                select! { sql_core::lexer::Token::Number(value) => value }
                    .delimited_by(l_paren.clone(), r_paren.clone())
                    .or_not(),
            )
            .map_with(|(name, precision), extra| {
                let data_type = match precision {
                    Some(precision) => std::borrow::Cow::Owned(format!("{}({precision})", name.0)),
                    None => name.0,
                };
                (data_type, extra.span())
            })
            .boxed();

        let cast_function = sql_macro::keyword!(CAST)
            .ignore_then(
                expr.clone()
                    .then_ignore(sql_macro::keyword!(AS))
                    .then(cast_data_type)
                    .delimited_by(l_paren.clone(), r_paren.clone()),
            )
            .map_with(|(expr, data_type), extra| Expr::Cast {
                expr: Box::new(expr),
                data_type,
                span: extra.span(),
            })
            .boxed();

        let subquery_expr = query
            .clone()
            .map_with(|query, extra| Expr::Subquery {
                query: Box::new(query),
                span: extra.span(),
            })
            .boxed();

        let array = expr
            .clone()
            .separated_by(comma.clone())
            .collect::<Vec<_>>()
            .delimited_by(l_bracket.clone(), r_bracket.clone())
            .map_with(|elements, extra| Expr::Array {
                elements,
                span: extra.span(),
            })
            .boxed();

        let function = any_word
            .clone()
            .then(
                named_expr
                    .or(subquery_expr.clone())
                    .or(expr.clone())
                    .then(
                        choice((
                            sql_macro::keyword!(IGNORE).then(sql_macro::keyword!(NULLS)).ignored(),
                            sql_macro::keyword!(RESPECT).then(sql_macro::keyword!(NULLS)).ignored(),
                        ))
                        .or_not(),
                    )
                    .map(|(expr, _)| expr)
                    .separated_by(comma.clone())
                    .collect::<Vec<_>>()
                    .delimited_by(l_paren.clone(), r_paren.clone()),
            )
            .then(
                sql_macro::keyword!(OVER)
                    .ignore_then(
                        sql_macro::keyword!(PARTITION)
                            .ignore_then(sql_macro::keyword!(BY))
                            .ignore_then(
                                expr.clone()
                                    .separated_by(comma.clone())
                                    .at_least(1)
                                    .collect::<Vec<_>>(),
                            )
                            .or_not()
                            .then(
                                sql_macro::keyword!(ORDER)
                                    .ignore_then(sql_macro::keyword!(BY))
                                    .ignore_then(
                                        expr.clone()
                                            .separated_by(comma.clone())
                                            .at_least(1)
                                            .collect::<Vec<_>>(),
                                    )
                                    .or_not(),
                            )
                            .delimited_by(l_paren.clone(), r_paren.clone()),
                    )
                    .or_not(),
            )
            .map_with(|((name, args), _over), extra| Expr::Function {
                kind: aggregate_kind(&name.0),
                name,
                args,
                span: extra.span(),
            })
            .boxed();

        let identifier = qualified_name
            .clone()
            .map_with(|parts, extra| Expr::Identifier {
                parts,
                span: extra.span(),
            })
            .boxed();

        let grouping_set = expr
            .clone()
            .separated_by(comma.clone())
            .collect::<Vec<_>>()
            .delimited_by(l_paren.clone(), r_paren.clone())
            .boxed();

        let grouping_sets = sql_macro::keyword!(GROUPING)
            .ignore_then(sql_macro::keyword!(SETS))
            .ignore_then(
                grouping_set
                    .clone()
                    .separated_by(comma.clone())
                    .at_least(1)
                    .collect::<Vec<_>>()
                    .delimited_by(l_paren.clone(), r_paren.clone()),
            )
            .map_with(|sets, extra| Expr::GroupingSets {
                sets,
                span: extra.span(),
            })
            .boxed();

        let literal = choice((
            select! { sql_core::lexer::Token::Number(value) => value },
            select! { sql_core::lexer::Token::StringLiteral(value) => value },
            select! { sql_core::lexer::Token::Word(word) if word.text.eq_ignore_ascii_case("TRUE") => word.text },
            select! { sql_core::lexer::Token::Word(word) if word.text.eq_ignore_ascii_case("FALSE") => word.text },
            select! { sql_core::lexer::Token::Word(word) if word.text.eq_ignore_ascii_case("NULL") => word.text },
        ))
        .map_with(|value, extra| Expr::Literal {
            value,
            span: extra.span(),
        })
        .boxed();

        let placeholder = select! { sql_core::lexer::Token::Placeholder(value) => value }
            .map_with(|name, extra| Expr::Placeholder {
                name,
                span: extra.span(),
            })
            .boxed();

        let wildcard = sql_macro::punct!(Star)
            .map_with(|_, extra| Expr::Wildcard { span: extra.span() })
            .boxed();

        let base_atom = choice((
            grouping_sets,
            cast_function,
            function,
            subquery_expr,
            identifier,
            literal,
            placeholder,
            wildcard,
            array,
            expr.clone().delimited_by(l_paren.clone(), r_paren.clone()),
        ));

        let mut atom = base_atom.boxed();
        for custom in custom_atoms.clone() {
            atom = custom.or(atom).boxed();
        }


        let postfix = choice((
            dot.clone().ignore_then(word.clone()).map(Postfix::Field),
            dot.clone()
                .ignore_then(
                    select! { sql_core::lexer::Token::Placeholder(value) => value }
                        .map_with(|value, extra| (std::borrow::Cow::Owned(format!("${value}")), extra.span())),
                )
                .map(Postfix::Field),
            dot.clone()
                .ignore_then(sql_macro::punct!(Star))
                .to(Postfix::Wildcard),
            colon.clone().ignore_then(word.clone()).map(Postfix::Field),
            sql_macro::punct!(DoubleColon)
                .ignore_then(any_word.clone())
                .map(Postfix::Cast),
            sql_macro::punct!(LParen)
                .ignore_then(sql_macro::punct!(Plus))
                .then_ignore(sql_macro::punct!(RParen))
                .to(Postfix::OuterJoin),
            colon
                .clone()
                .ignore_then(
                    expr.clone()
                        .delimited_by(l_bracket.clone(), r_bracket.clone()),
                )
                .map(Postfix::Index),
            expr.clone()
                .delimited_by(l_bracket.clone(), r_bracket.clone())
                .map(Postfix::Index),
        ))
        .boxed();

        let primary = atom
            .clone()
            .foldl(postfix.repeated(), |left, op| match op {
                Postfix::Field(field) => {
                    let span = expr_span(&left).start..field.1.end;
                    Expr::FieldAccess {
                        expr: Box::new(left),
                        field,
                        span,
                    }
                }
                Postfix::Index(index) => {
                    let span = expr_span(&left).start..(expr_span(&index).end + 1);
                    Expr::Index {
                        expr: Box::new(left),
                        index: Box::new(index),
                        span,
                    }
                }
                Postfix::Cast(data_type) => {
                    let span = expr_span(&left).start..data_type.1.end;
                    Expr::Cast {
                        expr: Box::new(left),
                        data_type,
                        span,
                    }
                }
                Postfix::OuterJoin => {
                    let left_span = expr_span(&left);
                    Expr::OuterJoin {
                        expr: Box::new(left),
                        span: left_span.start..(left_span.end + 3),
                    }
                }
                Postfix::Wildcard => {
                    let left_span = expr_span(&left);
                    Expr::QualifiedWildcard {
                        expr: Box::new(left),
                        span: left_span.start..(left_span.end + 2),
                    }
                }
            })
            .boxed();

        let not_op = sql_macro::keyword!(NOT)
            .map_with(|_, e| e.span())
            .map(|s| (UnaryOp::Not, s));
        let pos_op = sql_macro::punct!(Plus)
            .map_with(|_, e| e.span())
            .map(|s| (UnaryOp::Pos, s));
        let neg_op = sql_macro::punct!(Minus)
            .map_with(|_, e| e.span())
            .map(|s| (UnaryOp::Neg, s));

        let in_list_op = sql_macro::keyword!(NOT)
            .or_not()
            .then_ignore(sql_macro::keyword!(IN))
            .then(
                expr.clone()
                    .separated_by(comma.clone())
                    .at_least(1)
                    .collect::<Vec<_>>()
                    .delimited_by(l_paren.clone(), r_paren.clone()),
            )
            .map_with(|(negated, items), e| (negated.is_some(), items, e.span()));

        use chumsky::pratt::{infix, left, prefix};

        primary.pratt((
            prefix(
                50,
                not_op,
                |(op, op_span): (UnaryOp, ByteSpan), r: Expr, _| Expr::Unary {
                    op,
                    expr: Box::new(r.clone()),
                    span: op_span.start..expr_span(&r).end,
                },
            ),
            prefix(
                50,
                pos_op,
                |(op, op_span): (UnaryOp, ByteSpan), r: Expr, _| Expr::Unary {
                    op,
                    expr: Box::new(r.clone()),
                    span: op_span.start..expr_span(&r).end,
                },
            ),
            prefix(
                50,
                neg_op,
                |(op, op_span): (UnaryOp, ByteSpan), r: Expr, _| Expr::Unary {
                    op,
                    expr: Box::new(r.clone()),
                    span: op_span.start..expr_span(&r).end,
                },
            ),
            infix(
                left(40),
                sql_macro::punct!(Star).to(BinaryOp::Mul),
                |l: Expr, op, r: Expr, _| Expr::Binary {
                    span: expr_span(&l).start..expr_span(&r).end,
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            ),
            infix(
                left(40),
                sql_macro::punct!(Slash).to(BinaryOp::Div),
                |l: Expr, op, r: Expr, _| Expr::Binary {
                    span: expr_span(&l).start..expr_span(&r).end,
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            ),
            infix(
                left(30),
                sql_macro::punct!(PipePipe).to(BinaryOp::Concat),
                |l: Expr, op, r: Expr, _| Expr::Binary {
                    span: expr_span(&l).start..expr_span(&r).end,
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            ),
            infix(
                left(30),
                sql_macro::punct!(Plus).to(BinaryOp::Add),
                |l: Expr, op, r: Expr, _| Expr::Binary {
                    span: expr_span(&l).start..expr_span(&r).end,
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            ),
            infix(
                left(30),
                sql_macro::punct!(Minus).to(BinaryOp::Sub),
                |l: Expr, op, r: Expr, _| Expr::Binary {
                    span: expr_span(&l).start..expr_span(&r).end,
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            ),
            chumsky::pratt::postfix(
                25,
                in_list_op,
                |l: Expr, (negated, items, op_span): (bool, Vec<Expr<'src>>, ByteSpan), _| Expr::InList {
                    span: expr_span(&l).start..op_span.end,
                    expr: Box::new(l),
                    items,
                    negated,
                },
            ),
            infix(
                left(20),
                sql_macro::punct!(Eq).to(BinaryOp::Eq),
                |l: Expr, op, r: Expr, _| Expr::Binary {
                    span: expr_span(&l).start..expr_span(&r).end,
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            ),
            infix(
                left(20),
                sql_macro::punct!(Neq).to(BinaryOp::NotEq),
                |l: Expr, op, r: Expr, _| Expr::Binary {
                    span: expr_span(&l).start..expr_span(&r).end,
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            ),
            infix(
                left(20),
                sql_macro::punct!(NullSafeEq).to(BinaryOp::NullSafeEq),
                |l: Expr, op, r: Expr, _| Expr::Binary {
                    span: expr_span(&l).start..expr_span(&r).end,
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            ),
            infix(
                left(20),
                sql_macro::punct!(Lte).to(BinaryOp::Lte),
                |l: Expr, op, r: Expr, _| Expr::Binary {
                    span: expr_span(&l).start..expr_span(&r).end,
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            ),
            infix(
                left(20),
                sql_macro::punct!(Gte).to(BinaryOp::Gte),
                |l: Expr, op, r: Expr, _| Expr::Binary {
                    span: expr_span(&l).start..expr_span(&r).end,
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            ),
            infix(
                left(20),
                sql_macro::punct!(Lt).to(BinaryOp::Lt),
                |l: Expr, op, r: Expr, _| Expr::Binary {
                    span: expr_span(&l).start..expr_span(&r).end,
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            ),
            infix(
                left(20),
                sql_macro::punct!(Gt).to(BinaryOp::Gt),
                |l: Expr, op, r: Expr, _| Expr::Binary {
                    span: expr_span(&l).start..expr_span(&r).end,
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            ),
            infix(
                left(10),
                sql_macro::keyword!(AND).to(BinaryOp::And),
                |l: Expr, op, r: Expr, _| Expr::Binary {
                    span: expr_span(&l).start..expr_span(&r).end,
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            ),
            infix(
                left(5),
                sql_macro::keyword!(OR).to(BinaryOp::Or),
                |l: Expr, op, r: Expr, _| Expr::Binary {
                    span: expr_span(&l).start..expr_span(&r).end,
                    left: Box::new(l),
                    op,
                    right: Box::new(r),
                },
            ),
        ))
    })
    .boxed()
    }
}
