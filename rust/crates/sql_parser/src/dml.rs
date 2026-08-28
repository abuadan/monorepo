use chumsky::{input::ValueInput, prelude::*};

use sql_core::ast::{
    AlterAction, AlterStatement, Assignment, ColumnDef, CreateDefinition, CreateStatement, Expr,
    InsertStatement, Query, UpdateStatement,
};
use sql_core::lexer::Token;
use sql_core::span::ByteSpan;

use sql_core::helpers::{ParserExtra, aggregate_kind, any_word_parser, identifier_word_parser};

pub fn update_parser<'src, I>(
    _dialect: &'src dyn sql_core::dialect::Dialect,
) -> Boxed<'src, 'src, I, UpdateStatement<'src>, ParserExtra<'src>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = ByteSpan>,
{
    let any_word = any_word_parser();
    let comma = sql_macro::punct!(Comma).ignored();
    let dot = sql_macro::punct!(Dot).ignored();

    let object_name = any_word
        .clone()
        .separated_by(dot.clone())
        .at_least(1)
        .collect::<Vec<_>>()
        .boxed();

    let expr = simple_expr_parser().boxed();

    let assignment = any_word
        .clone()
        .then_ignore(sql_macro::punct!(Eq))
        .then(expr.clone())
        .map_with(|(column, value), extra| Assignment {
            column,
            value,
            span: extra.span(),
        })
        .boxed();

    sql_macro::seq!(
        sql_macro::keyword!(UPDATE)
            .ignore_then(object_name)
            .then_ignore(sql_macro::keyword!(SET)),
        assignment
            .separated_by(comma.clone())
            .at_least(1)
            .collect::<Vec<_>>(),
        sql_macro::keyword!(WHERE)
            .ignore_then(expr.clone())
            .or_not(),
        sql_macro::keyword!(RETURNING)
            .ignore_then(expr.separated_by(comma).at_least(1).collect::<Vec<_>>())
            .or_not()
    )
    .map_with(
        |(table, assignments, selection, returning), extra| UpdateStatement {
            table,
            assignments,
            selection,
            returning: returning.unwrap_or_default(),
            span: extra.span(),
        },
    )
    .boxed()
}

pub fn insert_parser<'src, I>(
    select_stmt: Boxed<'src, 'src, I, sql_core::ast::SelectStatement<'src>, ParserExtra<'src>>,
    dialect: &'src dyn sql_core::dialect::Dialect,
) -> Boxed<'src, 'src, I, InsertStatement<'src>, ParserExtra<'src>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = ByteSpan>,
{
    let word = identifier_word_parser(dialect);
    let any_word = any_word_parser();
    let comma = sql_macro::punct!(Comma).ignored();
    let dot = sql_macro::punct!(Dot).ignored();
    let l_paren = sql_macro::punct!(LParen).ignored();
    let r_paren = sql_macro::punct!(RParen).ignored();

    let object_name = any_word
        .clone()
        .separated_by(dot)
        .at_least(1)
        .collect::<Vec<_>>()
        .boxed();

    let leading_from = sql_macro::keyword!(FROM)
        .ignore_then(
            object_name
                .clone()
                .separated_by(comma.clone())
                .at_least(1)
                .collect::<Vec<_>>(),
        )
        .or_not();

    sql_macro::seq!(
        leading_from
            .then_ignore(sql_macro::keyword!(INSERT))
            .then_ignore(sql_macro::keyword!(INTO))
            .then_ignore(sql_macro::keyword!(TABLE)),
        object_name,
        sql_macro::keyword!(PARTITION)
            .ignore_then(
                word.separated_by(comma)
                    .at_least(1)
                    .collect::<Vec<_>>()
                    .delimited_by(l_paren, r_paren),
            )
            .or_not(),
        select_stmt
    )
    .map_with(
        |(from, target, partitioned_by, source), extra| InsertStatement {
            from: from.unwrap_or_default(),
            target,
            partitioned_by: partitioned_by.unwrap_or_default(),
            source,
            span: extra.span(),
        },
    )
    .boxed()
}

pub fn create_parser<'src, I>(
    query: Boxed<'src, 'src, I, Query<'src>, ParserExtra<'src>>,
    _dialect: &'src dyn sql_core::dialect::Dialect,
) -> Boxed<'src, 'src, I, CreateStatement<'src>, ParserExtra<'src>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = ByteSpan>,
{
    let any_word = any_word_parser();
    let comma = sql_macro::punct!(Comma).ignored();
    let dot = sql_macro::punct!(Dot).ignored();
    let l_paren = sql_macro::punct!(LParen).ignored();
    let r_paren = sql_macro::punct!(RParen).ignored();

    let object_name = any_word
        .clone()
        .separated_by(dot)
        .at_least(1)
        .collect::<Vec<_>>()
        .boxed();

    let data_type = any_word
        .clone()
        .then(
            select! { Token::Number(value) => value }
                .separated_by(comma.clone())
                .at_least(1)
                .collect::<Vec<_>>()
                .delimited_by(l_paren.clone(), r_paren.clone())
                .or_not(),
        )
        .map_with(|(name, precision), extra| {
            let text = match precision {
                Some(parts) => {
                    let joined = parts
                        .into_iter()
                        .map(|part| part.into_owned())
                        .collect::<Vec<_>>()
                        .join(", ");
                    std::borrow::Cow::Owned(format!("{}({joined})", name.0))
                }
                None => name.0,
            };
            (text, extra.span())
        })
        .boxed();

    let column_def = any_word
        .clone()
        .then(data_type)
        .map_with(|(name, data_type), extra| ColumnDef {
            name,
            data_type,
            span: extra.span(),
        })
        .boxed();

    let table_definition = column_def
        .clone()
        .separated_by(comma.clone())
        .at_least(1)
        .collect::<Vec<_>>()
        .delimited_by(l_paren.clone(), r_paren.clone())
        .map(CreateDefinition::Columns)
        .or(sql_macro::keyword!(AS)
            .ignore_then(query.clone())
            .map(|query| CreateDefinition::AsQuery(Box::new(query))))
        .boxed();

    let view_definition = sql_macro::keyword!(AS)
        .ignore_then(query)
        .map(|query| CreateDefinition::AsQuery(Box::new(query)))
        .boxed();

    let prefix = sql_macro::keyword!(CREATE)
        .ignore_then(
            sql_macro::keyword!(OR)
                .ignore_then(sql_macro::keyword!(REPLACE))
                .to(true)
                .or_not()
                .map(|value| value.unwrap_or(false)),
        );

    let if_not_exists = sql_macro::keyword!(IF)
        .ignore_then(sql_macro::keyword!(NOT))
        .ignore_then(sql_macro::keyword!(EXISTS))
        .to(true)
        .or_not()
        .map(|value| value.unwrap_or(false));

    let create_table = prefix
        .clone()
        .then(sql_macro::keyword!(TABLE).map(|span| (std::borrow::Cow::Borrowed("TABLE"), span)))
        .then(if_not_exists.clone())
        .then(object_name.clone())
        .then(table_definition)
        .map_with(
            |((((or_replace, object_type), if_not_exists), name), definition), extra| CreateStatement {
                or_replace,
                object_type,
                if_not_exists,
                name,
                definition,
                span: extra.span(),
            },
        )
        .boxed();

    let create_view = prefix
        .then(sql_macro::keyword!(VIEW).map(|span| (std::borrow::Cow::Borrowed("VIEW"), span)))
        .then(if_not_exists)
        .then(object_name)
        .then(view_definition)
        .map_with(
            |((((or_replace, object_type), if_not_exists), name), definition), extra| CreateStatement {
                or_replace,
                object_type,
                if_not_exists,
                name,
                definition,
                span: extra.span(),
            },
        )
        .boxed();

    choice((create_table, create_view)).boxed()
}

pub fn alter_parser<'src, I>(
    _dialect: &'src dyn sql_core::dialect::Dialect,
) -> Boxed<'src, 'src, I, AlterStatement<'src>, ParserExtra<'src>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = ByteSpan>,
{
    let any_word = any_word_parser();
    let comma = sql_macro::punct!(Comma).ignored();
    let dot = sql_macro::punct!(Dot).ignored();
    let l_paren = sql_macro::punct!(LParen).ignored();
    let r_paren = sql_macro::punct!(RParen).ignored();

    let object_name = any_word
        .clone()
        .separated_by(dot)
        .at_least(1)
        .collect::<Vec<_>>()
        .boxed();

    let data_type = any_word
        .clone()
        .then(
            select! { Token::Number(value) => value }
                .separated_by(comma)
                .at_least(1)
                .collect::<Vec<_>>()
                .delimited_by(l_paren, r_paren)
                .or_not(),
        )
        .map_with(|(name, precision), extra| {
            let text = match precision {
                Some(parts) => {
                    let joined = parts
                        .into_iter()
                        .map(|part| part.into_owned())
                        .collect::<Vec<_>>()
                        .join(", ");
                    std::borrow::Cow::Owned(format!("{}({joined})", name.0))
                }
                None => name.0,
            };
            (text, extra.span())
        })
        .boxed();

    let column_def = any_word
        .clone()
        .then(data_type)
        .map_with(|(name, data_type), extra| ColumnDef {
            name,
            data_type,
            span: extra.span(),
        })
        .boxed();

    let add_column = sql_macro::keyword!(ADD)
        .ignore_then(sql_macro::keyword!(COLUMN).or_not())
        .ignore_then(column_def.clone())
        .map(AlterAction::AddColumn);

    let rename_to = sql_macro::keyword!(RENAME)
        .ignore_then(sql_macro::keyword!(TO))
        .ignore_then(object_name.clone())
        .map(AlterAction::RenameTo);

    sql_macro::keyword!(ALTER)
        .ignore_then(
            sql_macro::keyword!(TABLE)
                .map(|span| (std::borrow::Cow::Borrowed("TABLE"), span))
                .or(sql_macro::keyword!(VIEW).map(|span| (std::borrow::Cow::Borrowed("VIEW"), span))),
        )
        .then(object_name)
        .then(choice((add_column, rename_to)))
        .map_with(|((object_type, name), action), extra| AlterStatement {
            object_type,
            name,
            action,
            span: extra.span(),
        })
        .boxed()
}

pub fn simple_expr_parser<'src, I>() -> Boxed<'src, 'src, I, Expr<'src>, ParserExtra<'src>>
where
    I: ValueInput<'src, Token = Token<'src>, Span = ByteSpan>,
{
    recursive(|expr| {
        let any_word = any_word_parser();
        let comma = sql_macro::punct!(Comma).ignored();
        let dot = sql_macro::punct!(Dot).ignored();
        let l_paren = sql_macro::punct!(LParen).ignored();
        let r_paren = sql_macro::punct!(RParen).ignored();

        let identifier = any_word
            .clone()
            .separated_by(dot)
            .at_least(1)
            .collect::<Vec<_>>()
            .map_with(|parts, extra| Expr::Identifier {
                parts,
                span: extra.span(),
            });

        let literal = choice((
            select! { Token::Number(value) => value },
            select! { Token::StringLiteral(value) => value },
            select! { Token::Word(word) if word.text.eq_ignore_ascii_case("NULL") => word.text },
        ))
        .map_with(|value, extra| Expr::Literal {
            value,
            span: extra.span(),
        });

        let placeholder = select! { Token::Placeholder(value) => value }.map_with(|name, extra| {
            Expr::Placeholder {
                name,
                span: extra.span(),
            }
        });

        let function = any_word
            .clone()
            .then(
                expr.clone()
                    .separated_by(comma)
                    .collect::<Vec<_>>()
                    .delimited_by(l_paren.clone(), r_paren.clone()),
            )
            .map_with(|(name, args), extra| {
                let kind = aggregate_kind(&name.0);
                Expr::Function {
                    name,
                    args,
                    kind,
                    span: extra.span(),
                }
            });

        let primary = choice((
            function,
            identifier,
            literal,
            placeholder,
            expr.clone().delimited_by(l_paren, r_paren),
        ))
        .boxed();

        let unary = choice((
            sql_macro::punct!(Plus)
                .ignore_then(primary.clone())
                .map_with(|expr, extra| Expr::Unary {
                    op: sql_core::ast::UnaryOp::Pos,
                    expr: Box::new(expr),
                    span: extra.span(),
                }),
            sql_macro::punct!(Minus)
                .ignore_then(primary.clone())
                .map_with(|expr, extra| Expr::Unary {
                    op: sql_core::ast::UnaryOp::Neg,
                    expr: Box::new(expr),
                    span: extra.span(),
                }),
            primary,
        ))
        .boxed();

        let add = unary.clone().foldl(
            choice((
                sql_macro::punct!(PipePipe).to(sql_core::ast::BinaryOp::Concat),
                sql_macro::punct!(Plus).to(sql_core::ast::BinaryOp::Add),
            ))
            .then(unary.clone())
            .repeated(),
            |left, (op, right)| Expr::Binary {
                span: sql_core::helpers::expr_span(&left).start
                    ..sql_core::helpers::expr_span(&right).end,
                left: Box::new(left),
                op,
                right: Box::new(right),
            },
        );

        add.clone()
            .then(
                choice((
                    sql_macro::punct!(Eq).to(sql_core::ast::BinaryOp::Eq),
                    sql_macro::punct!(NullSafeEq).to(sql_core::ast::BinaryOp::NullSafeEq),
                ))
                .then(add)
                .or_not(),
            )
            .map(|(left, rest)| match rest {
                Some((op, right)) => Expr::Binary {
                    span: sql_core::helpers::expr_span(&left).start
                        ..sql_core::helpers::expr_span(&right).end,
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                },
                None => left,
            })
            .boxed()
    })
    .boxed()
}
