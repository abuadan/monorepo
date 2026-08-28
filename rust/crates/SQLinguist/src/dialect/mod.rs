pub mod bigquery;
pub mod postgres;
pub mod snowflake;
pub mod spark;
pub mod trino;

use sqlparser::dialect::Dialect as SqlDialect;
use std::sync::Arc;

/// Ground-truth three-part qualified name
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct QualifiedName {
    pub database: Option<String>,
    pub schema: Option<String>,
    pub name: String,
}

/// Core trait representing semantic dialect differences beyond pure AST parsing
pub trait DialectRules: Send + Sync {
    /// Provide the sqlparser-rs Dialect instance for AST parsing
    fn parser_dialect(&self) -> Arc<dyn SqlDialect>;

    /// Resolve an arbitrary split array of strings into a standard three-part name
    fn resolve_qualified_name(&self, parts: &[String]) -> QualifiedName;

    /// Some dialects allow aliased columns in group by (Snowflake/BigQuery)
    fn allows_alias_in_group_by(&self) -> bool {
        false
    }

    /// True if the dialect supports `SELECT * EXCLUDE (col_name)`
    fn supports_exclude_clause(&self) -> bool {
        false
    }
}
