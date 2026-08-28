use super::{DialectRules, QualifiedName};
use sqlparser::dialect::BigQueryDialect as SqlBigQueryDialect;
use sqlparser::dialect::Dialect as SqlDialect;
use std::sync::Arc;

pub struct BigQueryRules;

impl DialectRules for BigQueryRules {
    fn parser_dialect(&self) -> Arc<dyn SqlDialect> {
        Arc::new(SqlBigQueryDialect {})
    }

    fn resolve_qualified_name(&self, parts: &[String]) -> QualifiedName {
        // BigQuery uses project.dataset.table
        match parts.len() {
            1 => QualifiedName {
                database: None,
                schema: None,
                name: parts[0].clone(),
            },
            2 => QualifiedName {
                database: None,
                schema: Some(parts[0].clone()), // dataset
                name: parts[1].clone(),         // table
            },
            _ => QualifiedName {
                database: Some(parts[0].clone()), // project
                schema: Some(parts[1].clone()),   // dataset
                name: parts[2].clone(),           // table
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
