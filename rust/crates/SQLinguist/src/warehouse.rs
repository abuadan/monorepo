use crate::catalog::TableDef;
use crate::dialect::QualifiedName;
use std::collections::HashMap;
use async_trait::async_trait;

#[derive(Debug)]
pub enum WarehouseError {
    TablesNotFound,
    QueryFailed(String),
    ConnectionFailed(String),
}

/// A result capturing inferred namespaces from a warehouse query
#[derive(Debug, Clone)]
pub struct NamespaceResult {
    pub database: Option<String>,
    pub schema: Option<String>,
}

impl NamespaceResult {
    pub fn is_consistent(&self) -> bool {
        true // Simplified for the trait definition
    }
    
    pub fn majority_schema(&self) -> NamespaceResult {
        self.clone()
    }
}

/// The core interface for Live Schema Metadata integrations
#[async_trait]
pub trait WarehouseConnector: Send + Sync {
    /// Request the metadata of an explicitly named table
    async fn describe_table(&self, name: &QualifiedName) -> Result<TableDef, WarehouseError>;
    
    /// Bulk-fetch metadata for multiple tables in the same schema
    async fn describe_tables_in_schema(
        &self,
        schema: &str,
        tables: &[String],
    ) -> Result<HashMap<QualifiedName, TableDef>, WarehouseError>;
        
    /// Resolve namespaces given an arbitrary list of tables (useful for un-applied migrations)
    async fn find_tables_namespace(
        &self,
        table_names: &[String],
    ) -> Result<NamespaceResult, WarehouseError>;
    
    /// Smoke test connection
    async fn test_connection(&self) -> Result<(), WarehouseError>;
}
