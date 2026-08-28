use chumsky::{input::ValueInput, prelude::*};

use sql_core::ast::{
    Join, JoinConstraint, JoinOperator, OrderByExpr, Query, RenameItem, ReplaceItem, SelectItem,
    SelectStatement, SortDirection, TableFactor, TableReference, WildcardOptions,
};
use sql_core::lexer::Token;
use sql_core::span::ByteSpan;

use sql_core::helpers::{
    ParserExtra, any_word_parser, column_alias_word_parser, identifier_word_parser,
    numeric_prefixed_alias_parser, punct, table_alias_word_parser, table_factor_word_parser,
};

use crate::expr::ExprParserBuilder;

pub fn select_parser<'src, I>(
    query: Boxed<'src, 'src, I, Query<'src>, ParserExtra<'src>>,
    dialect: &'src dyn sql_core::dialect::Dialect,
) -> Boxed<'src, 'src, I, SelectStatement<'src>, ParserExtra<'src>>
where
    I: ValueInput<'src, Token = sql_core::lexer::Token<'src>, Span = ByteSpan>,
{
    recursive(|_select| {
        let word = identifier_word_parser(dialect);
        let any_word = any_word_parser();
        let column_alias_word = column_alias_word_parser(dialect);
        let table_alias_word = table_alias_word_parser(dialect);
        let table_factor_word = table_factor_word_parser(dialect);
        let numeric_alias_word = numeric_prefixed_alias_parser();
        let comma = punct::<I>(Token::Comma).ignored();
        let dot = punct::<I>(Token::Dot).ignored();
        let minus = punct::<I>(Token::Minus).ignored();
        let l_paren = punct::<I>(Token::LParen).ignored();
        let r_paren = punct::<I>(Token::RParen).ignored();

        let table_name_part = table_factor_word
            .clone()
            .then(
                minus
                    .ignore_then(table_factor_word.clone())
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .map(|(first, rest)| {
                let mut text = first.0.into_owned();
                let start = first.1.start;
                let mut end = first.1.end;
                for part in rest {
                    text.push('-');
                    text.push_str(&part.0);
                    end = part.1.end;
                }
                (std::borrow::Cow::Owned(text), start..end)
            })
            .boxed();

        let table_name = table_name_part
            .clone()
            .then(
                sql_macro::punct!(Dot)
                    .then(table_name_part.clone().or_not())
                    .repeated()
                    .collect::<Vec<_>>(),
            )
            .map(|(first, rest)| {
                let mut parts = vec![first];
                for (dot_span, part) in rest {
                    parts.push(part.unwrap_or_else(|| (std::borrow::Cow::Owned(String::new()), dot_span.end..dot_span.end)));
                }
                parts
            })
            .boxed();

        let expr = ExprParserBuilder::new(dialect).build(query.clone());

        let column_alias = sql_macro::keyword!(AS)
            .ignore_then(word.clone().or(numeric_alias_word.clone()))
            .or(column_alias_word.clone())
            .or_not();

        let object_name = any_word
            .clone()
            .separated_by(dot.clone())
            .at_least(1)
            .collect::<Vec<_>>()
            .boxed();

        let exclude_item = object_name
            .clone()
            .separated_by(comma.clone())
            .at_least(1)
            .collect::<Vec<_>>()
            .delimited_by(l_paren.clone(), r_paren.clone())
            .or(object_name.clone().map(|item| vec![item]))
            .boxed();

        let replace_item = expr
            .clone()
            .then(sql_macro::keyword!(AS).ignore_then(word.clone()))
            .map_with(|(expr, column), extra| ReplaceItem {
                expr,
                column,
                span: extra.span(),
            })
            .boxed();

        let rename_item = word
            .clone()
            .then_ignore(sql_macro::keyword!(AS))
            .then(word.clone())
            .map_with(|(from, to), extra| RenameItem {
                from,
                to,
                span: extra.span(),
            })
            .boxed();

        let wildcard_options = sql_macro::keyword!(ILIKE)
            .ignore_then(select! { sql_core::lexer::Token::StringLiteral(value) => value })
            .map(|pattern| WildcardOptions {
                ilike: Some(pattern),
                ..WildcardOptions::default()
            })
            .or(sql_macro::keyword!(EXCLUDE)
                .ignore_then(exclude_item.clone())
                .or_not()
                .then(
                    sql_macro::keyword!(REPLACE)
                        .ignore_then(
                            replace_item
                                .clone()
                                .separated_by(comma.clone())
                                .at_least(1)
                                .collect::<Vec<_>>()
                                .delimited_by(l_paren.clone(), r_paren.clone()),
                        )
                        .or_not(),
                )
                .then(
                    sql_macro::keyword!(RENAME)
                        .ignore_then(
                            rename_item
                                .clone()
                                .separated_by(comma.clone())
                                .at_least(1)
                                .collect::<Vec<_>>()
                                .delimited_by(l_paren.clone(), r_paren.clone())
                                .or(rename_item.clone().map(|item| vec![item])),
                        )
                        .or_not(),
                )
                .map(|((exclude, replace), rename)| WildcardOptions {
                    ilike: None,
                    exclude: exclude.unwrap_or_default(),
                    replace: replace.unwrap_or_default(),
                    rename: rename.unwrap_or_default(),
                }))
            .or_not()
            .map(|options| options.unwrap_or_default())
            .boxed();

        let table_alias_follower = choice((
            end().ignored(),
            sql_macro::punct!(Comma).ignored(),
            sql_macro::keyword!(AT).ignored(),
            sql_macro::keyword!(BEFORE).ignored(),
            sql_macro::keyword!(CHANGES).ignored(),
            sql_macro::keyword!(END).ignored(),
            sql_macro::keyword!(INNER).ignored(),
            sql_macro::keyword!(LEFT).ignored(),
            sql_macro::keyword!(RIGHT).ignored(),
            sql_macro::keyword!(FULL).ignored(),
            sql_macro::keyword!(CROSS).ignored(),
            sql_macro::keyword!(JOIN).ignored(),
            sql_macro::keyword!(PIVOT).ignored(),
            sql_macro::keyword!(SAMPLE).ignored(),
            sql_macro::keyword!(TABLESAMPLE).ignored(),
            sql_macro::keyword!(WHERE).ignored(),
            sql_macro::keyword!(GROUP).ignored(),
            sql_macro::keyword!(HAVING).ignored(),
            sql_macro::keyword!(ORDER).ignored(),
            sql_macro::keyword!(LIMIT).ignored(),
            sql_macro::keyword!(OFFSET).ignored(),
        ))
        .rewind();

        let limit_alias =
            select! { sql_core::lexer::Token::Word(word) if word.text.eq_ignore_ascii_case("LIMIT") => word.text }
                .map_with(|text, extra| (text, extra.span()))
                .then_ignore(table_alias_follower.clone());

        let table_alias = sql_macro::keyword!(AS)
            .ignore_then(word.clone())
            .or(limit_alias)
            .or(table_alias_word.clone())
            .or_not();

        let alias_column_list = word
            .clone()
            .separated_by(comma.clone())
            .at_least(1)
            .collect::<Vec<_>>()
            .delimited_by(l_paren.clone(), r_paren.clone())
            .or_not()
            .ignored()
            .boxed();

        let named_clause_arg = any_word
            .clone()
            .then_ignore(sql_macro::punct!(Eq))
            .then_ignore(sql_macro::punct!(Gt))
            .then(expr.clone())
            .ignored()
            .boxed();

        let named_clause_args = named_clause_arg
            .clone()
            .separated_by(comma.clone())
            .at_least(1)
            .collect::<Vec<_>>()
            .delimited_by(l_paren.clone(), r_paren.clone())
            .ignored()
            .boxed();

        let non_paren_token = select! {
            sql_core::lexer::Token::Word(_) => (),
            sql_core::lexer::Token::Whitespace(_) => (),
            sql_core::lexer::Token::LineComment(_) => (),
            sql_core::lexer::Token::BlockComment(_) => (),
            sql_core::lexer::Token::StageRef(_) => (),
            sql_core::lexer::Token::Number(_) => (),
            sql_core::lexer::Token::StringLiteral(_) => (),
            sql_core::lexer::Token::Placeholder(_) => (),
            sql_core::lexer::Token::Dot => (),
            sql_core::lexer::Token::Colon => (),
            sql_core::lexer::Token::Comma => (),
            sql_core::lexer::Token::PipePipe => (),
            sql_core::lexer::Token::Plus => (),
            sql_core::lexer::Token::Minus => (),
            sql_core::lexer::Token::Slash => (),
            sql_core::lexer::Token::Star => (),
            sql_core::lexer::Token::LBracket => (),
            sql_core::lexer::Token::RBracket => (),
            sql_core::lexer::Token::DoubleColon => (),
            sql_core::lexer::Token::Eq => (),
            sql_core::lexer::Token::Neq => (),
            sql_core::lexer::Token::NullSafeEq => (),
            sql_core::lexer::Token::Lt => (),
            sql_core::lexer::Token::Lte => (),
            sql_core::lexer::Token::Gt => (),
            sql_core::lexer::Token::Gte => (),
        }
        .ignored()
        .boxed();

        let balanced_parens = recursive(|balanced| {
            choice((
                sql_macro::punct!(LParen)
                    .ignore_then(balanced.repeated())
                    .then_ignore(sql_macro::punct!(RParen))
                    .ignored(),
                non_paren_token.clone(),
            ))
        })
        .boxed();

        let pivot_clause = sql_macro::keyword!(PIVOT)
            .ignore_then(
                sql_macro::punct!(LParen)
                    .ignore_then(balanced_parens.clone().repeated())
                    .then_ignore(sql_macro::punct!(RParen)),
            )
            .ignored()
            .boxed();

        let sample_clause = choice((sql_macro::keyword!(SAMPLE), sql_macro::keyword!(TABLESAMPLE)))
            .ignore_then(
                any_word.clone().or_not().ignored().then(
                    sql_macro::punct!(LParen)
                        .ignore_then(balanced_parens.clone().repeated())
                        .then_ignore(sql_macro::punct!(RParen)),
                ),
            )
            .then(
                choice((sql_macro::keyword!(SEED), sql_macro::keyword!(REPEATABLE)))
                    .ignore_then(
                        sql_macro::punct!(LParen)
                            .ignore_then(balanced_parens.clone().repeated())
                            .then_ignore(sql_macro::punct!(RParen)),
                    )
                    .or_not(),
            )
            .ignored()
            .boxed();

        let pre_alias_modifier = choice((
            sql_macro::keyword!(AT).ignore_then(named_clause_args.clone()),
            sql_macro::keyword!(BEFORE).ignore_then(named_clause_args.clone()),
            sql_macro::keyword!(END).ignore_then(named_clause_args.clone()),
            sql_macro::keyword!(CHANGES).ignore_then(named_clause_args.clone()),
            pivot_clause.clone(),
        ))
        .ignored()
        .boxed();

        let post_alias_modifier = choice((
            sql_macro::keyword!(AT).ignore_then(named_clause_args.clone()),
            sql_macro::keyword!(BEFORE).ignore_then(named_clause_args.clone()),
            sql_macro::keyword!(END).ignore_then(named_clause_args.clone()),
            sql_macro::keyword!(CHANGES).ignore_then(named_clause_args.clone()),
            sample_clause,
        ))
        .ignored()
        .boxed();

        let select_item = expr
            .clone()
            .then(wildcard_options)
            .then(column_alias)
            .map_with(|((expr, wildcard_options), alias), extra| SelectItem {
                expr,
                alias,
                wildcard_options,
                span: extra.span(),
            })
            .boxed();

        let table_function_args = expr
            .clone()
            .separated_by(comma.clone())
            .collect::<Vec<_>>()
            .delimited_by(l_paren.clone(), r_paren.clone());

        let with_ordinality = sql_macro::keyword!(WITH)
            .ignore_then(sql_macro::keyword!(ORDINALITY))
            .to(true)
            .or_not()
            .map(|value| value.unwrap_or(false));

        let table_factor = choice((
            query
                .clone()
                .delimited_by(l_paren.clone(), r_paren.clone())
                .map(|subquery| TableFactor::Derived {
                    subquery: Box::new(subquery),
                }),
            select! { sql_core::lexer::Token::StageRef(value) => value }
                .then(named_clause_args.clone().or_not())
                .map_with(|(location, _options), extra| TableFactor::Stage {
                    location: (location, extra.span()),
                }),
            sql_macro::keyword!(UNNEST)
                .ignore_then(table_function_args.clone())
                .then(with_ordinality)
                .map(|(args, with_ordinality)| TableFactor::Unnest {
                    args,
                    with_ordinality,
                }),
            any_word
                .clone()
                .then(table_function_args.clone())
                .then(with_ordinality)
                .map(|((name, args), with_ordinality)| TableFactor::Function {
                    name,
                    args,
                    with_ordinality,
                }),
            table_name.map(|name| TableFactor::Named { name }),
        ));

        let table_reference = table_factor
            .then(pre_alias_modifier.repeated().collect::<Vec<_>>())
            .then(table_alias.clone())
            .then(alias_column_list)
            .then(post_alias_modifier.repeated().collect::<Vec<_>>())
            .map_with(
                |(
                    (((factor, _pre_alias_modifiers), alias), _alias_columns),
                    _post_alias_modifiers,
                ),
                 extra| TableReference {
                    factor,
                    alias,
                    span: extra.span(),
                },
            )
            .boxed();

        let join_operator = choice((
            sql_macro::keyword!(INNER)
                .ignore_then(sql_macro::keyword!(JOIN))
                .to(JoinOperator::Inner),
            sql_macro::keyword!(LEFT)
                .ignore_then(sql_macro::keyword!(JOIN))
                .to(JoinOperator::Left),
            sql_macro::keyword!(RIGHT)
                .ignore_then(sql_macro::keyword!(JOIN))
                .to(JoinOperator::Right),
            sql_macro::keyword!(FULL)
                .ignore_then(sql_macro::keyword!(JOIN))
                .to(JoinOperator::Full),
            sql_macro::keyword!(CROSS)
                .ignore_then(sql_macro::keyword!(JOIN))
                .to(JoinOperator::Cross),
            sql_macro::keyword!(JOIN).to(JoinOperator::Inner),
        ));

        let join = join_operator
            .then(table_reference.clone())
            .then(
                choice((
                    sql_macro::keyword!(ON)
                        .ignore_then(expr.clone())
                        .map(JoinConstraint::On),
                    sql_macro::keyword!(USING)
                        .ignore_then(
                            word.clone()
                                .separated_by(comma.clone())
                                .at_least(1)
                                .collect::<Vec<_>>()
                                .delimited_by(l_paren.clone(), r_paren.clone()),
                        )
                        .map(JoinConstraint::Using),
                ))
                .or_not(),
            )
            .map_with(|((operator, relation), constraint), extra| Join {
                operator,
                relation,
                constraint: constraint.unwrap_or(JoinConstraint::None),
                span: extra.span(),
            })
            .boxed();

        let order_direction = choice((
            sql_macro::keyword!(ASC).to(SortDirection::Asc),
            sql_macro::keyword!(DESC).to(SortDirection::Desc),
        ))
        .or_not();

        let order_by_expr =
            expr.clone()
                .then(order_direction)
                .map_with(|(expr, direction), extra| OrderByExpr {
                    expr,
                    direction,
                    span: extra.span(),
                });

        let projection = sql_macro::keyword!(SELECT)
            .then(sql_macro::keyword!(DISTINCT).or_not())
            .then(
                select_item
                    .separated_by(comma.clone())
                    .at_least(1)
                    .collect::<Vec<_>>(),
            )
            .map(|((_, distinct), projection)| (distinct.is_some(), projection));

        sql_macro::seq!(
            projection,
            sql_macro::keyword!(FROM)
                .ignore_then(
                    table_reference
                        .separated_by(comma.clone())
                        .at_least(1)
                        .collect::<Vec<_>>(),
                )
                .or_not(),
            join.repeated().collect::<Vec<_>>(),
            sql_macro::keyword!(WHERE).ignore_then(expr.clone()).or_not(),
            sql_macro::keyword!(GROUP)
                .ignore_then(sql_macro::keyword!(BY))
                .ignore_then(
                    expr.clone()
                        .separated_by(comma.clone())
                        .at_least(1)
                        .collect::<Vec<_>>(),
                )
                .or_not(),
            sql_macro::keyword!(HAVING).ignore_then(expr.clone()).or_not(),
            sql_macro::keyword!(ORDER)
                .ignore_then(sql_macro::keyword!(BY))
                .ignore_then(
                    order_by_expr
                        .separated_by(comma.clone())
                        .at_least(1)
                        .collect::<Vec<_>>(),
                )
                .or_not(),
            sql_macro::keyword!(LIMIT).ignore_then(expr.clone()).or_not(),
            sql_macro::keyword!(OFFSET).ignore_then(expr.clone()).or_not()
        )
        .map_with(
            |(distinct_and_projection, from, joins, selection, group_by, having, order_by, limit, offset), extra| {
                let (distinct, projection) = distinct_and_projection;
                    SelectStatement {
                        distinct,
                        projection,
                        from: from.unwrap_or_default(),
                        joins,
                        selection,
                        group_by: group_by.unwrap_or_default(),
                        having,
                        order_by: order_by.unwrap_or_default(),
                        limit,
                        offset,
                        span: extra.span(),
                    }
                },
            )
            .boxed()
    })
    .boxed()
}
