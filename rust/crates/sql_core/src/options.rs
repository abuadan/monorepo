#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Dialect {
    Generic,
    Postgres,
    BigQuery,
    Snowflake,
    Databricks,
    MySQL,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ParserOptions {
    pub dialect: Dialect,
    pub allow_dollar_quoted_strings: bool,
    pub allow_triple_quoted_strings: bool,
    pub allow_stage_refs: bool,
    pub allow_null_safe_eq: bool,
}

impl ParserOptions {
    pub fn for_dialect(dialect: Dialect) -> Self {
        match dialect {
            Dialect::Generic => Self {
                dialect,
                allow_dollar_quoted_strings: true,
                allow_triple_quoted_strings: true,
                allow_stage_refs: true,
                allow_null_safe_eq: true,
            },
            Dialect::Postgres => Self {
                dialect,
                allow_dollar_quoted_strings: true,
                allow_triple_quoted_strings: false,
                allow_stage_refs: false,
                allow_null_safe_eq: false,
            },
            Dialect::BigQuery => Self {
                dialect,
                allow_dollar_quoted_strings: false,
                allow_triple_quoted_strings: true,
                allow_stage_refs: false,
                allow_null_safe_eq: false,
            },
            Dialect::Snowflake => Self {
                dialect,
                allow_dollar_quoted_strings: true,
                allow_triple_quoted_strings: false,
                allow_stage_refs: true,
                allow_null_safe_eq: false,
            },
            Dialect::Databricks => Self {
                dialect,
                allow_dollar_quoted_strings: false,
                allow_triple_quoted_strings: false,
                allow_stage_refs: false,
                allow_null_safe_eq: false,
            },
            Dialect::MySQL => Self {
                dialect,
                allow_dollar_quoted_strings: false,
                allow_triple_quoted_strings: false,
                allow_stage_refs: false,
                allow_null_safe_eq: true,
            },
        }
    }
}

impl Default for ParserOptions {
    fn default() -> Self {
        Self::for_dialect(Dialect::Generic)
    }
}
