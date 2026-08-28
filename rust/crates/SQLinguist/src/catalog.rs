use crate::dialect::QualifiedName;
use sqlparser::ast::DataType as SqlType;
use std::collections::HashMap;

/// Representation of a single column within a table or view
#[derive(Debug, Clone, PartialEq)]
pub struct Column {
    pub name: String,
    pub data_type: Option<SqlType>,
    pub nullable: bool,
}

impl Column {
    pub fn from_sqlparser(col: &sqlparser::ast::ColumnDef) -> Self {
        let nullable = !col.options.iter().any(|opt| {
            matches!(
                opt.option,
                sqlparser::ast::ColumnOption::NotNull
            )
        });

        Self {
            name: col.name.value.clone(),
            data_type: Some(col.data_type.clone()),
            nullable,
        }
    }
}

/// Representation of a Table or a View
#[derive(Debug, Clone, PartialEq)]
pub struct TableDef {
    pub name: QualifiedName,
    pub columns: Vec<Column>,
    pub is_view: bool,
}

/// A fully qualified catalog representing resolved schema components
#[derive(Debug, Clone, Default)]
pub struct Catalog {
    pub tables: HashMap<QualifiedName, TableDef>,
}

impl Catalog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_table(&mut self, table: TableDef) {
        self.tables.insert(table.name.clone(), table);
    }

    pub fn get_table(&self, name: &QualifiedName) -> Option<&TableDef> {
        self.tables.get(name)
    }

    /// Recursively scan the AST and register all statically defined DDLs
    pub fn parse_ddl(&mut self, statements: &[sqlparser::ast::Statement], dialect: &dyn crate::dialect::DialectRules) {
        for statement in statements {
            match statement {
                sqlparser::ast::Statement::CreateTable(create_table) => {
                    let resolved_name = dialect.resolve_qualified_name(
                        &create_table.name.0.iter().map(|ident| ident.value.clone()).collect::<Vec<_>>()
                    );

                    let mut columns = Vec::new();
                    for col in &create_table.columns {
                        columns.push(Column::from_sqlparser(col));
                    }

                    self.add_table(TableDef {
                        name: resolved_name,
                        columns,
                        is_view: false,
                    });
                }
                sqlparser::ast::Statement::CreateView(create_view) => {
                    let resolved_name = dialect.resolve_qualified_name(
                        &create_view.name.0.iter().map(|ident| ident.value.clone()).collect::<Vec<_>>()
                    );
                    
                    // We don't have columns resolved until Phase 3C semantic fixed-point
                    self.add_table(TableDef {
                        name: resolved_name,
                        columns: vec![],
                        is_view: true,
                    });
                }
                _ => {}
            }
        }
    }
}
