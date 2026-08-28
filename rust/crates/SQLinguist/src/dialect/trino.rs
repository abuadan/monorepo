use super::{DialectRules, QualifiedName};
use sqlparser::dialect::GenericDialect as SqlGenericDialect;
use sqlparser::dialect::Dialect as SqlDialect;
use std::sync::Arc;

pub struct TrinoRules;

impl DialectRules for TrinoRules {
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
                database: Some(parts[0].clone()),
                schema: Some(parts[1].clone()),
                name: parts[2].clone(),
            },
        }
    }
}
