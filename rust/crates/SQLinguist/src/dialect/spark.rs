use super::{DialectRules, QualifiedName};
use sqlparser::dialect::GenericDialect as SqlGenericDialect;
use sqlparser::dialect::Dialect as SqlDialect;
use std::sync::Arc;

pub struct SparkRules;

impl DialectRules for SparkRules {
    fn parser_dialect(&self) -> Arc<dyn SqlDialect> {
        Arc::new(SqlGenericDialect {})
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
                database: None, // Ignore >3 for spark unless configured
                schema: Some(parts[0].clone()),
                name: parts[1].clone(),
            },
        }
    }
}
