use super::{DialectRules, QualifiedName};
use sqlparser::dialect::SnowflakeDialect as SqlSnowflakeDialect;
use sqlparser::dialect::Dialect as SqlDialect;
use std::sync::Arc;

pub struct SnowflakeRules;

impl DialectRules for SnowflakeRules {
    fn parser_dialect(&self) -> Arc<dyn SqlDialect> {
        Arc::new(SqlSnowflakeDialect {})
    }

    fn resolve_qualified_name(&self, parts: &[String]) -> QualifiedName {
        match parts.len() {
            1 => QualifiedName {
                database: None,
                schema: None,
                name: parts[0].clone(),
            },
            2 => QualifiedName {
                database: None,
                schema: Some(parts[0].clone()),
                name: parts[1].clone(),
            },
            _ => QualifiedName {
                database: Some(parts[0].clone()),
                schema: Some(parts[1].clone()),
                name: parts[2].clone(),
            },
        }
    }

    fn allows_alias_in_group_by(&self) -> bool {
        true
    }

    fn supports_exclude_clause(&self) -> bool {
        true
    }
}
