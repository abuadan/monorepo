use crate::ast::{
    AggregateKind, Cte, CteBody, Expr, InsertStatement, JoinConstraint, Query, QueryBody,
    SelectItem, SelectStatement, TableFactor,
};
use crate::span::Spanned;

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct QueryAnalysis<'a> {
    pub ctes: Vec<Spanned<crate::ast::AstStr<'a>>>,
    pub table_references: Vec<Vec<Spanned<crate::ast::AstStr<'a>>>>,
    pub table_aliases: Vec<Spanned<crate::ast::AstStr<'a>>>,
    pub column_references: Vec<Vec<Spanned<crate::ast::AstStr<'a>>>>,
    pub column_aliases: Vec<Spanned<crate::ast::AstStr<'a>>>,
    pub aggregates: Vec<AggregateKind>,
}

impl<'a> QueryAnalysis<'a> {
    pub fn from_query(query: &'a Query<'a>) -> Self {
        let mut analysis = Self::default();

        for cte in &query.with {
            analysis.push_cte(cte);
        }

        match &query.body {
            QueryBody::Select(select) => analysis.visit_select(select),
            QueryBody::Insert(insert) => analysis.visit_insert(insert),
        }
        analysis
    }

    fn push_cte(&mut self, cte: &'a Cte<'a>) {
        self.ctes.push(cte.name.clone());
        match cte.query.as_ref() {
            CteBody::Query(query) => {
                let nested = Self::from_query(query);
                self.ctes.extend(nested.ctes);
                self.table_references.extend(nested.table_references);
                self.table_aliases.extend(nested.table_aliases);
                self.column_references.extend(nested.column_references);
                self.column_aliases.extend(nested.column_aliases);
                self.aggregates.extend(nested.aggregates);
            }
            CteBody::Update(update) => {
                for assignment in &update.assignments {
                    self.visit_expr(&assignment.value);
                }
                if let Some(selection) = &update.selection {
                    self.visit_expr(selection);
                }
                for expr in &update.returning {
                    self.visit_expr(expr);
                }
            }
        }
    }

    fn visit_insert(&mut self, insert: &'a InsertStatement<'a>) {
        for source in &insert.from {
            self.table_references.push(source.clone());
        }
        self.table_references.push(insert.target.clone());
        self.visit_select(&insert.source);
    }

    fn visit_select(&mut self, select: &'a SelectStatement<'a>) {
        for item in &select.projection {
            self.visit_select_item(item);
        }

        for table in &select.from {
            match &table.factor {
                TableFactor::Named { name } => self.table_references.push(name.clone()),
                TableFactor::Stage { location } => {
                    self.table_references.push(vec![location.clone()])
                }
                TableFactor::Function { args, .. } | TableFactor::Unnest { args, .. } => {
                    for arg in args {
                        self.visit_expr(arg);
                    }
                }
                TableFactor::Derived { subquery } => {
                    let nested = Self::from_query(subquery);
                    self.ctes.extend(nested.ctes);
                    self.table_references.extend(nested.table_references);
                    self.table_aliases.extend(nested.table_aliases);
                    self.column_references.extend(nested.column_references);
                    self.column_aliases.extend(nested.column_aliases);
                    self.aggregates.extend(nested.aggregates);
                }
            }

            if let Some(alias) = &table.alias {
                self.table_aliases.push(alias.clone());
            }
        }

        for join in &select.joins {
            match &join.relation.factor {
                TableFactor::Named { name } => self.table_references.push(name.clone()),
                TableFactor::Stage { location } => {
                    self.table_references.push(vec![location.clone()])
                }
                TableFactor::Function { args, .. } | TableFactor::Unnest { args, .. } => {
                    for arg in args {
                        self.visit_expr(arg);
                    }
                }
                TableFactor::Derived { subquery } => {
                    let nested = Self::from_query(subquery);
                    self.ctes.extend(nested.ctes);
                    self.table_references.extend(nested.table_references);
                    self.table_aliases.extend(nested.table_aliases);
                    self.column_references.extend(nested.column_references);
                    self.column_aliases.extend(nested.column_aliases);
                    self.aggregates.extend(nested.aggregates);
                }
            }

            if let Some(alias) = &join.relation.alias {
                self.table_aliases.push(alias.clone());
            }

            match &join.constraint {
                JoinConstraint::On(expr) => self.visit_expr(expr),
                JoinConstraint::Using(columns) => {
                    for column in columns {
                        self.column_references.push(vec![column.clone()]);
                    }
                }
                JoinConstraint::None => {}
            }
        }

        if let Some(selection) = &select.selection {
            self.visit_expr(selection);
        }

        for expr in &select.group_by {
            self.visit_expr(expr);
        }

        if let Some(having) = &select.having {
            self.visit_expr(having);
        }

        for order in &select.order_by {
            self.visit_expr(&order.expr);
        }

        if let Some(limit) = &select.limit {
            self.visit_expr(limit);
        }

        if let Some(offset) = &select.offset {
            self.visit_expr(offset);
        }
    }

    fn visit_select_item(&mut self, item: &'a SelectItem<'a>) {
        self.visit_expr(&item.expr);
        for path in &item.wildcard_options.exclude {
            self.column_references.push(path.clone());
        }
        for replacement in &item.wildcard_options.replace {
            self.visit_expr(&replacement.expr);
        }
        for rename in &item.wildcard_options.rename {
            self.column_references.push(vec![rename.from.clone()]);
            self.column_aliases.push(rename.to.clone());
        }
        if let Some(alias) = &item.alias {
            self.column_aliases.push(alias.clone());
        }
    }

    fn visit_expr(&mut self, expr: &'a Expr<'a>) {
        match expr {
            Expr::Placeholder { .. } => {}
            Expr::Subquery { query, .. } => {
                let nested = Self::from_query(query);
                self.ctes.extend(nested.ctes);
                self.table_references.extend(nested.table_references);
                self.table_aliases.extend(nested.table_aliases);
                self.column_references.extend(nested.column_references);
                self.column_aliases.extend(nested.column_aliases);
                self.aggregates.extend(nested.aggregates);
            }
            Expr::Identifier { parts, .. } => self.column_references.push(parts.clone()),
            Expr::Literal { .. } | Expr::Wildcard { .. } => {}
            Expr::Function { args, kind, .. } => {
                if let Some(kind) = kind {
                    self.aggregates.push(*kind);
                }
                for arg in args {
                    self.visit_expr(arg);
                }
            }
            Expr::Named { expr, .. } => self.visit_expr(expr),
            Expr::Array { elements, .. } => {
                for element in elements {
                    self.visit_expr(element);
                }
            }
            Expr::FieldAccess { expr, field, .. } => {
                if let Some(mut parts) = expr_identifier_parts(expr) {
                    parts.push(field.clone());
                    self.column_references.push(parts);
                } else {
                    self.visit_expr(expr);
                }
            }
            Expr::Cast { expr, .. } => self.visit_expr(expr),
            Expr::OuterJoin { expr, .. } => self.visit_expr(expr),
            Expr::GroupingSets { sets, .. } => {
                for set in sets {
                    for expr in set {
                        self.visit_expr(expr);
                    }
                }
            }
            Expr::Index { expr, index, .. } => {
                self.visit_expr(expr);
                self.visit_expr(index);
            }
            Expr::QualifiedWildcard { expr, .. } => self.visit_expr(expr),
            Expr::InList { expr, items, .. } => {
                self.visit_expr(expr);
                for item in items {
                    self.visit_expr(item);
                }
            }
            Expr::Unary { expr, .. } => self.visit_expr(expr),
            Expr::Binary { left, right, .. } => {
                self.visit_expr(left);
                self.visit_expr(right);
            }
        }
    }
}

fn expr_identifier_parts<'a>(expr: &'a Expr<'a>) -> Option<Vec<Spanned<crate::ast::AstStr<'a>>>> {
    match expr {
        Expr::Identifier { parts, .. } => Some(parts.clone()),
        Expr::FieldAccess { expr, field, .. } => {
            let mut parts = expr_identifier_parts(expr)?;
            parts.push(field.clone());
            Some(parts)
        }
        _ => None,
    }
}
