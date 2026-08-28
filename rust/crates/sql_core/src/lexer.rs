use std::borrow::Cow;

use logos::Logos;

use crate::error::LexError;
use crate::keywords::{
    Keyword, classify_keyword, is_reserved_for_column_alias as keyword_reserved_for_column_alias,
    is_reserved_for_identifier as keyword_reserved_for_identifier,
    is_reserved_for_table_alias as keyword_reserved_for_table_alias,
    is_reserved_for_table_factor as keyword_reserved_for_table_factor,
};
use crate::options::ParserOptions;
use crate::span::{ByteSpan, Spanned};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Word<'a> {
    pub text: Cow<'a, str>,
    pub keyword: Keyword,
    pub quoted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token<'a> {
    Word(Word<'a>),
    Whitespace(Cow<'a, str>),
    LineComment(Cow<'a, str>),
    BlockComment(Cow<'a, str>),
    StageRef(Cow<'a, str>),
    Number(Cow<'a, str>),
    StringLiteral(Cow<'a, str>),
    Placeholder(Cow<'a, str>),
    Dot,
    Colon,
    Comma,
    PipePipe,
    Plus,
    Minus,
    Slash,
    Star,
    LParen,
    RParen,
    LBracket,
    RBracket,
    DoubleColon,
    Eq,
    Neq,
    NullSafeEq,
    Lt,
    Lte,
    Gt,
    Gte,
}

pub type LexedToken<'a> = Spanned<Token<'a>>;

pub struct Tokenizer<'a> {
    source: &'a str,
}

impl<'a> Tokenizer<'a> {
    pub fn new(source: &'a str) -> Self {
        Self { source }
    }

    pub fn tokenize(&self) -> Result<Vec<LexedToken<'a>>, LexError> {
        self.tokenize_with_options(&ParserOptions::default())
    }

    pub fn tokenize_with_options(
        &self,
        options: &ParserOptions,
    ) -> Result<Vec<LexedToken<'a>>, LexError> {
        let mut offset = 0;
        let mut tokens = Vec::new();

        while offset < self.source.len() {
            if let Some((token, end)) = scan_trivia(self.source, offset)? {
                tokens.push((token, offset..end));
                offset = end;
                continue;
            }

            let mut lexer = RawToken::lexer(&self.source[offset..]);
            let Some(next) = lexer.next() else {
                break;
            };

            let span = lexer.span();
            let absolute_span = offset + span.start..offset + span.end;
            match next {
                Ok(raw) => {
                    let slice = &self.source[absolute_span.clone()];
                    if !raw.is_allowed(options, slice) {
                        return Err(LexError::InvalidToken {
                            span: absolute_span,
                        });
                    }
                    tokens.push((raw_token_to_borrowed(raw, slice), absolute_span.clone()));
                    offset = absolute_span.end;
                }
                Err(()) => {
                    return Err(LexError::InvalidToken {
                        span: absolute_span,
                    });
                }
            }
        }

        Ok(tokens)
    }
}

#[derive(Logos, Debug, Clone, PartialEq, Eq)]
enum RawToken {
    #[regex(r"@[A-Za-z0-9_./:=-]+")]
    StageRef,

    #[regex(r"\$\$[^$]*\$\$")]
    DollarQuotedStringLiteral,

    #[regex(r#"(?i:[br])'''([^']|'')*'''"#)]
    #[regex(r#"(?i:[br])\"\"\"([^\"]|\"\")*\"\"\""#)]
    #[regex(r#"(?i:[br])'([^']|'')*'"#)]
    #[regex(r#"(?i:[br])\"([^\"]|\"\")*\""#)]
    PrefixedStringLiteral,

    #[regex(r"[A-Za-z_][A-Za-z0-9_]*")]
    BareWord,

    #[regex(r#""([^"]|"")*""#)]
    QuotedWord,

    #[regex(r"`([^`]|``)*`")]
    BacktickWord,

    #[regex(r"[0-9]+(?:\.[0-9]+)?")]
    Number,

    #[regex(r":[0-9]+")]
    #[regex(r"\$[0-9]+")]
    Placeholder,

    #[regex(r"'([^']|'')*'")]
    StringLiteral,

    #[token(".")]
    Dot,
    #[token(":")]
    Colon,
    #[token(",")]
    Comma,
    #[token("||")]
    PipePipe,
    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("/")]
    Slash,
    #[token("*")]
    Star,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token("::")]
    DoubleColon,
    #[token("<=>")]
    NullSafeEq,
    #[token("<>")]
    #[token("!=")]
    Neq,
    #[token("<=")]
    Lte,
    #[token(">=")]
    Gte,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("=")]
    Eq,
}

impl<'a> Token<'a> {
    pub fn is_trivia(&self) -> bool {
        matches!(
            self,
            Self::Whitespace(_) | Self::LineComment(_) | Self::BlockComment(_)
        )
    }
}

fn parse_prefixed_string_literal(slice: &str) -> Cow<'_, str> {
    let body = &slice[1..];

    if let Some(content) = body
        .strip_prefix("'''")
        .and_then(|rest| rest.strip_suffix("'''"))
    {
        return decode_escaped(content, "''", "'");
    }

    if let Some(content) = body
        .strip_prefix("\"\"\"")
        .and_then(|rest| rest.strip_suffix("\"\"\""))
    {
        return decode_escaped(content, "\"\"", "\"");
    }

    if let Some(content) = body
        .strip_prefix('\'')
        .and_then(|rest| rest.strip_suffix('\''))
    {
        return decode_escaped(content, "''", "'");
    }

    if let Some(content) = body
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
    {
        return decode_escaped(content, "\"\"", "\"");
    }

    Cow::Borrowed(body)
}

fn parse_dollar_quoted_string_literal(slice: &str) -> Cow<'_, str> {
    slice
        .strip_prefix("$$")
        .and_then(|rest| rest.strip_suffix("$$"))
        .unwrap_or(slice)
        .into()
}

fn parse_quoted_identifier<'a>(
    slice: &'a str,
    quote: char,
    escaped: &str,
    replacement: &str,
) -> Cow<'a, str> {
    let content = slice
        .strip_prefix(quote)
        .and_then(|rest| rest.strip_suffix(quote))
        .unwrap_or(slice);
    decode_escaped(content, escaped, replacement)
}

fn parse_placeholder(slice: &str) -> Cow<'_, str> {
    Cow::Borrowed(&slice[1..])
}

fn parse_string_literal(slice: &str) -> Cow<'_, str> {
    let content = slice
        .strip_prefix('\'')
        .and_then(|rest| rest.strip_suffix('\''))
        .unwrap_or(slice);
    decode_escaped(content, "''", "'")
}

fn decode_escaped<'a>(content: &'a str, escaped: &str, replacement: &str) -> Cow<'a, str> {
    if content.contains(escaped) {
        Cow::Owned(content.replace(escaped, replacement))
    } else {
        Cow::Borrowed(content)
    }
}

pub fn tokenize(source: &str) -> Result<Vec<LexedToken<'_>>, LexError> {
    Tokenizer::new(source).tokenize()
}

pub fn tokenize_with_options<'a>(
    source: &'a str,
    options: &ParserOptions,
) -> Result<Vec<LexedToken<'a>>, LexError> {
    Tokenizer::new(source).tokenize_with_options(options)
}

pub fn token_text<'a>(source: &'a str, span: &ByteSpan) -> &'a str {
    &source[span.clone()]
}

pub fn is_reserved_for_table_alias(word: &Word) -> bool {
    keyword_reserved_for_table_alias(word.keyword)
}

pub fn is_reserved_for_column_alias(word: &Word) -> bool {
    keyword_reserved_for_column_alias(word.keyword)
}

pub fn is_reserved_for_table_factor(word: &Word) -> bool {
    keyword_reserved_for_table_factor(word.keyword)
}

pub fn is_reserved_for_identifier(word: &Word) -> bool {
    keyword_reserved_for_identifier(word.keyword)
}

pub fn keyword_map() -> &'static std::collections::HashMap<&'static str, Keyword> {
    crate::keywords::keyword_map()
}

fn scan_trivia(source: &str, offset: usize) -> Result<Option<(Token<'_>, usize)>, LexError> {
    let rest = &source[offset..];

    if let Some(ch) = rest.chars().next() {
        if ch.is_whitespace() {
            let end = offset
                + rest
                    .char_indices()
                    .take_while(|(_, ch)| ch.is_whitespace())
                    .last()
                    .map(|(idx, ch)| idx + ch.len_utf8())
                    .unwrap_or(ch.len_utf8());
            return Ok(Some((
                Token::Whitespace(Cow::Borrowed(&source[offset..end])),
                end,
            )));
        }
    }

    if let Some(line) = rest.strip_prefix("--") {
        let line_len = line.find('\n').map(|idx| idx + 2).unwrap_or(rest.len());
        let end = offset + line_len;
        return Ok(Some((
            Token::LineComment(Cow::Borrowed(&source[offset..end])),
            end,
        )));
    }

    if rest.starts_with("/*") {
        let end = scan_block_comment_end(source, offset).ok_or(LexError::InvalidToken {
            span: offset..source.len(),
        })?;
        return Ok(Some((
            Token::BlockComment(Cow::Borrowed(&source[offset..end])),
            end,
        )));
    }

    Ok(None)
}

fn scan_block_comment_end(source: &str, offset: usize) -> Option<usize> {
    let mut depth = 0usize;
    let mut idx = offset;
    while idx < source.len() {
        let rest = &source[idx..];
        if rest.starts_with("/*") {
            depth += 1;
            idx += 2;
        } else if rest.starts_with("*/") {
            depth = depth.saturating_sub(1);
            idx += 2;
            if depth == 0 {
                return Some(idx);
            }
        } else {
            idx += rest.chars().next()?.len_utf8();
        }
    }
    None
}

impl RawToken {
    fn is_allowed(&self, options: &ParserOptions, slice: &str) -> bool {
        match self {
            Self::StageRef => options.allow_stage_refs,
            Self::DollarQuotedStringLiteral => options.allow_dollar_quoted_strings,
            Self::PrefixedStringLiteral => {
                !slice.contains('\n') || options.allow_triple_quoted_strings
            }
            Self::NullSafeEq => options.allow_null_safe_eq,
            _ => true,
        }
    }
}

fn raw_token_to_borrowed<'a>(value: RawToken, slice: &'a str) -> Token<'a> {
    match value {
        RawToken::StageRef => Token::StageRef(Cow::Borrowed(slice)),
        RawToken::DollarQuotedStringLiteral => {
            Token::StringLiteral(parse_dollar_quoted_string_literal(slice))
        }
        RawToken::PrefixedStringLiteral => {
            Token::StringLiteral(parse_prefixed_string_literal(slice))
        }
        RawToken::BareWord => Token::Word(Word {
            text: Cow::Borrowed(slice),
            keyword: classify_keyword(slice),
            quoted: false,
        }),
        RawToken::QuotedWord => Token::Word(Word {
            text: parse_quoted_identifier(slice, '"', "\"\"", "\""),
            keyword: Keyword::NoKeyword,
            quoted: true,
        }),
        RawToken::BacktickWord => Token::Word(Word {
            text: parse_quoted_identifier(slice, '`', "``", "`"),
            keyword: Keyword::NoKeyword,
            quoted: true,
        }),
        RawToken::Number => Token::Number(Cow::Borrowed(slice)),
        RawToken::Placeholder => Token::Placeholder(parse_placeholder(slice)),
        RawToken::StringLiteral => Token::StringLiteral(parse_string_literal(slice)),
        RawToken::Dot => Token::Dot,
        RawToken::Colon => Token::Colon,
        RawToken::Comma => Token::Comma,
        RawToken::PipePipe => Token::PipePipe,
        RawToken::Plus => Token::Plus,
        RawToken::Minus => Token::Minus,
        RawToken::Slash => Token::Slash,
        RawToken::Star => Token::Star,
        RawToken::LParen => Token::LParen,
        RawToken::RParen => Token::RParen,
        RawToken::LBracket => Token::LBracket,
        RawToken::RBracket => Token::RBracket,
        RawToken::DoubleColon => Token::DoubleColon,
        RawToken::Eq => Token::Eq,
        RawToken::Neq => Token::Neq,
        RawToken::NullSafeEq => Token::NullSafeEq,
        RawToken::Lt => Token::Lt,
        RawToken::Lte => Token::Lte,
        RawToken::Gt => Token::Gt,
        RawToken::Gte => Token::Gte,
    }
}

#[cfg(test)]
mod tests {
    use std::borrow::Cow;

    use crate::options::{Dialect, ParserOptions};

    use super::{Token, tokenize, tokenize_with_options};

    #[test]
    fn tokenizes_nested_block_comments_as_trivia() {
        let source = "SELECT /* outer /* inner */ still outer */ 1";
        let tokens = tokenize(source).expect("tokenizes");

        assert!(
            tokens
                .iter()
                .any(|(token, _)| matches!(token, Token::BlockComment(_)))
        );
    }

    #[test]
    fn round_trips_source_via_spans() {
        let source = "SELECT -- hello\n/* block */ 1";
        let tokens = tokenize(source).expect("tokenizes");
        let rebuilt = tokens
            .iter()
            .map(|(_, span)| &source[span.clone()])
            .collect::<String>();

        assert_eq!(rebuilt, source);
    }

    #[test]
    fn borrows_unescaped_token_text() {
        let tokens: Vec<_> = tokenize(r#"SELECT "col", 'value', $$body$$"#)
            .expect("tokenizes")
            .into_iter()
            .filter(|(t, _)| !t.is_trivia())
            .collect();

        assert!(
            matches!(&tokens[0].0, Token::Word(word) if matches!(word.text, Cow::Borrowed("SELECT")))
        );
        assert!(
            matches!(&tokens[1].0, Token::Word(word) if matches!(word.text, Cow::Borrowed("col")))
        );
        assert!(
            matches!(&tokens[3].0, Token::StringLiteral(value) if matches!(value, Cow::Borrowed("value")))
        );
        assert!(
            matches!(&tokens[5].0, Token::StringLiteral(value) if matches!(value, Cow::Borrowed("body")))
        );
    }

    #[test]
    fn allocates_only_when_escape_normalization_is_needed() {
        let tokens: Vec<_> = tokenize(r#"SELECT "co""l", 'va''lue'"#)
            .expect("tokenizes")
            .into_iter()
            .filter(|(t, _)| !t.is_trivia())
            .collect();

        assert!(matches!(&tokens[1].0, Token::Word(word) if matches!(word.text, Cow::Owned(_))));
        assert!(
            matches!(&tokens[3].0, Token::StringLiteral(value) if matches!(value, Cow::Owned(_)))
        );
    }

    #[test]
    fn gates_stage_refs_by_dialect_options() {
        let options = ParserOptions::for_dialect(Dialect::Postgres);
        tokenize_with_options("SELECT * FROM @stage/file.csv", &options)
            .expect_err("stage refs should be rejected");
    }
}
