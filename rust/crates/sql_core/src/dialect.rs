//! Dialect-specific SQL parsing rules.
//!
//! Different SQL dialects (PostgreSQL, Snowflake, BigQuery, etc.) have varying
//! rules for what constitutes a reserved keyword. For example, `STRING` is reserved
//! in BigQuery but a valid identifier in PostgreSQL. The [`Dialect`] trait lets you
//! inject dialect-specific keyword logic into the core parser at runtime.
//!
//! # Built-in Dialects
//!
//! | Dialect             | Type              |
//! |---------------------|-------------------|
//! | ANSI / Generic SQL  | [`GenericDialect`] |
//!
//! # Creating a Custom Dialect
//!
//! Implement the [`Dialect`] trait and pass it to [`sql_parser`] entry-points.
//!
//! ```ignore
//! use sql_core::dialect::{Dialect, GenericDialect};
//!
//! /// A dialect tailored for PostgreSQL syntax.
//! #[derive(Debug)]
//! pub struct PostgresDialect;
//!
//! impl Dialect for PostgresDialect {
//!     fn is_reserved_for_identifier(&self, text: &str) -> bool {
//!         // Inherit ANSI reservations, plus lock-down JSONB/RETURNING.
//!         GenericDialect.is_reserved_for_identifier(text)
//!             || text.eq_ignore_ascii_case("JSONB")
//!             || text.eq_ignore_ascii_case("RETURNING")
//!     }
//!
//!     fn is_reserved_for_table_alias(&self, text: &str) -> bool {
//!         GenericDialect.is_reserved_for_table_alias(text)
//!     }
//!
//!     fn is_reserved_for_column_alias(&self, text: &str) -> bool {
//!         GenericDialect.is_reserved_for_column_alias(text)
//!     }
//!
//!     fn is_reserved_for_table_factor(&self, text: &str) -> bool {
//!         GenericDialect.is_reserved_for_table_factor(text)
//!     }
//! }
//!
//! // Then use it with the parser:
//! // use sql_parser::query::parse_query_with_options;
//! // let ast = parse_query_with_options("SELECT id FROM users", &Default::default());
//! ```

/// Defines dialect-specific keyword-reservation rules for the SQL parser.
///
/// SQL dialects diverge significantly on what words are reserved — a keyword
/// illegal as an alias in one dialect may be perfectly valid in another.
/// Implementing this trait allows the core parser to classify tokens correctly
/// for the target dialect **without modifying parser internals**.
///
/// # Contract
///
/// Each method receives the **raw, case-sensitive token text** exactly as it
/// appeared in the source SQL. Implementations should use
/// [`str::eq_ignore_ascii_case`] for case-insensitive comparisons.
///
/// # Example: Snowflake Dialect with Extra Reserved Words
///
/// ```rust
/// use sql_core::dialect::{Dialect, GenericDialect};
///
/// #[derive(Debug)]
/// pub struct SnowflakeDialect;
///
/// impl Dialect for SnowflakeDialect {
///     fn is_reserved_for_identifier(&self, text: &str) -> bool {
///         // Snowflake reserves IFF, QUALIFY, and ILIKE as keywords.
///         GenericDialect.is_reserved_for_identifier(text)
///             || text.eq_ignore_ascii_case("IFF")
///             || text.eq_ignore_ascii_case("QUALIFY")
///             || text.eq_ignore_ascii_case("ILIKE")
///     }
///
///     fn is_reserved_for_table_alias(&self, text: &str) -> bool {
///         GenericDialect.is_reserved_for_table_alias(text)
///     }
///
///     fn is_reserved_for_column_alias(&self, text: &str) -> bool {
///         GenericDialect.is_reserved_for_column_alias(text)
///     }
///
///     fn is_reserved_for_table_factor(&self, text: &str) -> bool {
///         GenericDialect.is_reserved_for_table_factor(text)
///     }
/// }
/// ```
///
/// # Implementing an Entirely Custom Grammar
///
/// For a fully custom SQL variant, override all methods and return `false`
/// by default — then allowlist only the specific reserved words your dialect needs:
///
/// ```rust
/// use sql_core::dialect::Dialect;
///
/// #[derive(Debug)]
/// pub struct MinimalDialect;
///
/// impl Dialect for MinimalDialect {
///     fn is_reserved_for_identifier(&self, text: &str) -> bool {
///         // Only `SELECT` and `FROM` are reserved identifiers
///         matches!(text.to_ascii_uppercase().as_str(), "SELECT" | "FROM")
///     }
///     fn is_reserved_for_table_alias(&self, _text: &str) -> bool { false }
///     fn is_reserved_for_column_alias(&self, _text: &str) -> bool { false }
///     fn is_reserved_for_table_factor(&self, _text: &str) -> bool { false }
/// }
/// ```
pub trait Dialect: std::fmt::Debug + Sync + Send {
    /// Returns `true` if `text` is a keyword that cannot appear as an identifier
    /// (e.g., column name, CTE name) in this dialect.
    fn is_reserved_for_identifier(&self, text: &str) -> bool;

    /// Returns `true` if `text` is a keyword that cannot appear as a table alias
    /// in this dialect (e.g., `FROM t1 AS JOIN` would be rejected).
    fn is_reserved_for_table_alias(&self, text: &str) -> bool;

    /// Returns `true` if `text` is a keyword that cannot appear as a column alias
    /// in this dialect (e.g., `SELECT 1 AS FROM` would be rejected).
    fn is_reserved_for_column_alias(&self, text: &str) -> bool;

    /// Returns `true` if `text` is a keyword that cannot appear immediately after
    /// a table reference in the `FROM` clause of this dialect.
    fn is_reserved_for_table_factor(&self, text: &str) -> bool;
}

/// The default ANSI SQL dialect.
///
/// `GenericDialect` enforces the standard SQL-92 reserved keyword set. It is
/// the dialect used by [`parse_query`][sql_parser::query::parse_query] and
/// [`parse_statement`][sql_parser::query::parse_statement] when no dialect is
/// explicitly provided.
///
/// Use this as a base in custom implementations — delegate calls to
/// `GenericDialect` and then layer your dialect-specific overrides on top.
///
/// # Example
///
/// ```rust
/// use sql_core::dialect::{Dialect, GenericDialect};
///
/// // `SELECT` is a reserved identifier in the generic dialect.
/// assert!(GenericDialect.is_reserved_for_identifier("SELECT"));
///
/// // A plain column name like `id` is never reserved.
/// assert!(!GenericDialect.is_reserved_for_identifier("id"));
/// ```
#[derive(Debug)]
pub struct GenericDialect;

impl Dialect for GenericDialect {
    fn is_reserved_for_identifier(&self, text: &str) -> bool {
        crate::keywords::is_reserved_for_identifier(crate::keywords::classify_keyword(text))
    }

    fn is_reserved_for_table_alias(&self, text: &str) -> bool {
        crate::keywords::is_reserved_for_table_alias(crate::keywords::classify_keyword(text))
    }

    fn is_reserved_for_column_alias(&self, text: &str) -> bool {
        crate::keywords::is_reserved_for_column_alias(crate::keywords::classify_keyword(text))
    }

    fn is_reserved_for_table_factor(&self, text: &str) -> bool {
        crate::keywords::is_reserved_for_table_factor(crate::keywords::classify_keyword(text))
    }
}

/// A zero-cost static reference to the [`GenericDialect`].
///
/// Use this when you need a `&'static dyn Dialect` without heap allocation,
/// for example when calling [`parse_query`][sql_parser::query::parse_query]
/// internally or in tests.
pub static GENERIC_DIALECT: GenericDialect = GenericDialect;
