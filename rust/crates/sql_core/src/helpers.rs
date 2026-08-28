use chumsky::{error::Rich, extra, input::ValueInput, prelude::*};

use crate::ast::{AggregateKind, AstStr, Expr};
use crate::dialect::Dialect;
use crate::error::{DiagnosticLabel, ParseError};
use crate::keywords::{Keyword, keyword_map};
use crate::lexer::{Token, Word};
use crate::span::{ByteSpan, Spanned};

pub type RichParseError<'src> = Rich<'src, Token<'src>, ByteSpan>;
pub type ParserExtra<'src> = extra::Err<RichParseError<'src>>;

pub fn identifier_word_parser<'src, I>(
    dialect: &'src dyn Dialect,
) -> impl Parser<'src, I, Spanned<AstStr<'src>>, ParserExtra<'src>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = ByteSpan>,
{
    select! { Token::Word(word) if is_identifier_word(&word, dialect) => word.text }
        .map_with(|text, extra| (text, extra.span()))
}

pub fn any_word_parser<'src, I>()
-> impl Parser<'src, I, Spanned<AstStr<'src>>, ParserExtra<'src>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = ByteSpan>,
{
    select! { Token::Word(word) => word.text }.map_with(|text, extra| (text, extra.span()))
}

pub fn column_alias_word_parser<'src, I>(
    dialect: &'src dyn Dialect,
) -> impl Parser<'src, I, Spanned<AstStr<'src>>, ParserExtra<'src>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = ByteSpan>,
{
    select! { Token::Word(word) if !dialect.is_reserved_for_column_alias(&word.text) => word.text }
        .map_with(|text, extra| (text, extra.span()))
}

pub fn numeric_prefixed_alias_parser<'src, I>()
-> impl Parser<'src, I, Spanned<AstStr<'src>>, ParserExtra<'src>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = ByteSpan>,
{
    select! { Token::Number(value) => value }
        .then(select! { Token::Word(word) if word.text.starts_with('_') => word.text })
        .map_with(|(number, suffix), extra| {
            (
                std::borrow::Cow::Owned(format!("{number}{suffix}")),
                extra.span(),
            )
        })
}

pub fn table_alias_word_parser<'src, I>(
    dialect: &'src dyn Dialect,
) -> impl Parser<'src, I, Spanned<AstStr<'src>>, ParserExtra<'src>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = ByteSpan>,
{
    select! { Token::Word(word)
        if !dialect.is_reserved_for_table_alias(&word.text)
            && !word.text.eq_ignore_ascii_case("AT")
            && !word.text.eq_ignore_ascii_case("BEFORE")
            && !word.text.eq_ignore_ascii_case("CHANGES")
            && !word.text.eq_ignore_ascii_case("END")
            && !word.text.eq_ignore_ascii_case("PIVOT")
            && !word.text.eq_ignore_ascii_case("SAMPLE")
            && !word.text.eq_ignore_ascii_case("TABLESAMPLE")
        => word.text
    }
    .map_with(|text, extra| (text, extra.span()))
}

pub fn table_factor_word_parser<'src, I>(
    dialect: &'src dyn Dialect,
) -> impl Parser<'src, I, Spanned<AstStr<'src>>, ParserExtra<'src>> + Clone
where
    I: ValueInput<'src, Token = Token<'src>, Span = ByteSpan>,
{
    select! { Token::Word(word)
        if (!dialect.is_reserved_for_identifier(&word.text) && !dialect.is_reserved_for_table_factor(&word.text))
            || word.text.eq_ignore_ascii_case("LIMIT")
        => word.text
    }
    .map_with(|text, extra| (text, extra.span()))
}

pub fn punct<'src, I>(
    token: crate::lexer::Token<'static>,
) -> impl Parser<'src, I, ByteSpan, ParserExtra<'src>> + Clone + 'src
where
    I: ValueInput<'src, Token = Token<'src>, Span = ByteSpan>,
{
    any()
        .filter(move |t: &Token<'src>| *t == token)
        .map_with(|_, extra| extra.span())
}

pub fn aggregate_kind(name: &str) -> Option<AggregateKind> {
    if name.eq_ignore_ascii_case("count") {
        Some(AggregateKind::Count)
    } else if name.eq_ignore_ascii_case("sum") {
        Some(AggregateKind::Sum)
    } else if name.eq_ignore_ascii_case("avg") {
        Some(AggregateKind::Avg)
    } else if name.eq_ignore_ascii_case("min") {
        Some(AggregateKind::Min)
    } else if name.eq_ignore_ascii_case("max") {
        Some(AggregateKind::Max)
    } else {
        None
    }
}

pub fn expr_span<'src>(expr: &Expr<'src>) -> ByteSpan {
    match expr {
        Expr::Placeholder { span, .. }
        | Expr::Subquery { span, .. }
        | Expr::Identifier { span, .. }
        | Expr::Literal { span, .. }
        | Expr::Wildcard { span }
        | Expr::Function { span, .. }
        | Expr::Named { span, .. }
        | Expr::Array { span, .. }
        | Expr::FieldAccess { span, .. }
        | Expr::Cast { span, .. }
        | Expr::OuterJoin { span, .. }
        | Expr::GroupingSets { span, .. }
        | Expr::Index { span, .. }
        | Expr::QualifiedWildcard { span, .. }
        | Expr::InList { span, .. }
        | Expr::Unary { span, .. }
        | Expr::Binary { span, .. } => span.clone(),
    }
}

pub fn is_identifier_word(word: &Word<'_>, dialect: &dyn Dialect) -> bool {
    !dialect.is_reserved_for_identifier(&word.text) || word.text.eq_ignore_ascii_case("interval")
}

pub fn first_error(errors: Vec<RichParseError<'_>>, tokens: &[Spanned<Token<'_>>]) -> ParseError {
    let labels = typo_labels(tokens);
    match errors.into_iter().next() {
        Some(error) if error.found().is_none() => ParseError::Eof,
        Some(error) => ParseError::UnexpectedToken {
            found: format!("{:?}", error.found()),
            span: error.span().clone(),
            labels,
        },
        None => ParseError::Eof,
    }
}

fn typo_labels(tokens: &[Spanned<Token<'_>>]) -> Vec<DiagnosticLabel> {
    tokens
        .iter()
        .filter_map(|(token, span)| match token {
            Token::Word(word) if word.keyword == Keyword::NoKeyword => nearest_keyword(&word.text)
                .map(|keyword| DiagnosticLabel {
                    span: span.clone(),
                    message: format!("`{}` looks like the SQL keyword `{keyword}`", word.text),
                }),
            _ => None,
        })
        .collect()
}

fn nearest_keyword(word: &str) -> Option<&'static str> {
    let upper = word.to_ascii_uppercase();
    keyword_map()
        .keys()
        .copied()
        .filter_map(|keyword| {
            let distance = edit_distance(&upper, keyword);
            (distance > 0 && distance <= 2).then_some((keyword, distance))
        })
        .min_by_key(|(_, distance)| *distance)
        .map(|(keyword, _)| keyword)
}

fn edit_distance(left: &str, right: &str) -> usize {
    let left: Vec<char> = left.chars().collect();
    let right: Vec<char> = right.chars().collect();
    let mut prev: Vec<usize> = (0..=right.len()).collect();
    let mut curr = vec![0; right.len() + 1];

    for (i, lch) in left.iter().enumerate() {
        curr[0] = i + 1;
        for (j, rch) in right.iter().enumerate() {
            let cost = usize::from(lch != rch);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }

    prev[right.len()]
}
