use super::*;
use sql_core::ast::{AlterAction, CreateDefinition};

#[test]
fn select_literal_integer() {
    let s = select("SELECT 1");
    assert_eq!(s.projection.len(), 1);
    assert!(matches!(s.projection[0].expr, Expr::Literal { .. }));
}

#[test]
fn select_multiple_columns() {
    let s = select("SELECT id, name, email FROM users");
    assert_eq!(s.projection.len(), 3);
}

#[test]
fn select_wildcard() {
    let s = select("SELECT * FROM t");
    assert!(matches!(s.projection[0].expr, Expr::Wildcard { .. }));
}

#[test]
fn select_qualified_wildcard() {
    let s = select("SELECT t.* FROM t");
    assert!(matches!(s.projection[0].expr, Expr::QualifiedWildcard { .. }));
}

#[test]
fn select_column_alias_with_as() {
    let s = select("SELECT id AS user_id FROM users");
    let alias = s.projection[0].alias.as_ref().expect("alias present");
    assert_eq!(&*alias.0, "user_id");
}

#[test]
fn select_column_alias_without_as() {
    let s = select("SELECT id user_id FROM users");
    assert!(s.projection[0].alias.is_some());
}

#[test]
fn select_distinct() {
    let s = select("SELECT DISTINCT name FROM users");
    assert!(s.distinct);
}

#[test]
fn select_from_simple_table() {
    let s = select("SELECT 1 FROM users");
    assert_eq!(s.from.len(), 1);
    assert!(matches!(s.from[0].factor, TableFactor::Named { .. }));
}

#[test]
fn select_from_schema_qualified_table() {
    let s = select("SELECT 1 FROM mydb.public.users");
    if let TableFactor::Named { name } = &s.from[0].factor {
        assert_eq!(name.len(), 3);
    } else {
        panic!("expected Named table factor");
    }
}

#[test]
fn select_from_aliased_table() {
    let s = select("SELECT u.id FROM users u");
    assert!(s.from[0].alias.is_some());
    let alias = s.from[0].alias.as_ref().expect("alias");
    assert_eq!(&*alias.0, "u");
}

#[test]
fn select_from_aliased_table_with_as() {
    let s = select("SELECT u.id FROM users AS u");
    assert!(s.from[0].alias.is_some());
}

#[test]
fn select_from_multiple_tables_cross_join() {
    let s = select("SELECT * FROM a, b");
    assert_eq!(s.from.len(), 2);
}

#[test]
fn select_from_subquery() {
    let s = select("SELECT sub.id FROM (SELECT id FROM users) AS sub");
    assert!(matches!(
        s.from[0].factor,
        TableFactor::Derived { ref subquery } if subquery.body.as_select().is_some()
    ));
}

#[test]
fn inner_join_with_on() {
    let s = select("SELECT * FROM a INNER JOIN b ON a.id = b.a_id");
    assert_eq!(s.joins.len(), 1);
    assert_eq!(s.joins[0].operator, JoinOperator::Inner);
    assert!(matches!(s.joins[0].constraint, JoinConstraint::On(_)));
}

#[test]
fn left_join_with_using() {
    let s = select("SELECT * FROM a LEFT JOIN b USING (id)");
    assert_eq!(s.joins[0].operator, JoinOperator::Left);
    if let JoinConstraint::Using(cols) = &s.joins[0].constraint {
        assert_eq!(cols.len(), 1);
        assert_eq!(&*cols[0].0, "id");
    } else {
        panic!("expected Using constraint, got {:?}", s.joins[0].constraint);
    }
}

#[test]
fn right_join() {
    let s = select("SELECT * FROM a RIGHT JOIN b ON a.id = b.id");
    assert_eq!(s.joins[0].operator, JoinOperator::Right);
}

#[test]
fn full_outer_join() {
    let s = select("SELECT * FROM a FULL JOIN b ON a.id = b.id");
    assert_eq!(s.joins[0].operator, JoinOperator::Full);
}

#[test]
fn cross_join() {
    let s = select("SELECT * FROM a CROSS JOIN b");
    assert_eq!(s.joins[0].operator, JoinOperator::Cross);
    assert!(matches!(s.joins[0].constraint, JoinConstraint::None));
}

#[test]
fn multiple_joins() {
    let s = select(
        "SELECT * FROM a \
         INNER JOIN b ON a.id = b.id \
         LEFT JOIN c ON b.id = c.id",
    );
    assert_eq!(s.joins.len(), 2);
}

#[test]
fn where_simple_equality() {
    let s = select("SELECT * FROM t WHERE id = 1");
    assert!(s.selection.is_some());
}

#[test]
fn where_compound_and() {
    let s = select("SELECT * FROM t WHERE a = 1 AND b = 2");
    assert!(matches!(
        s.selection.as_ref().expect("where"),
        Expr::Binary { op: BinaryOp::And, .. }
    ));
}

#[test]
fn where_compound_or() {
    let s = select("SELECT * FROM t WHERE a = 1 OR b = 2");
    assert!(matches!(
        s.selection.as_ref().expect("where"),
        Expr::Binary { op: BinaryOp::Or, .. }
    ));
}

#[test]
fn where_not_negation() {
    let s = select("SELECT * FROM t WHERE NOT active");
    assert!(matches!(
        s.selection.as_ref().expect("where"),
        Expr::Unary { op: UnaryOp::Not, .. }
    ));
}

#[test]
fn where_in_list() {
    let s = select("SELECT * FROM t WHERE id IN (1, 2, 3)");
    assert!(matches!(
        s.selection.as_ref().expect("where"),
        Expr::InList { negated: false, .. }
    ));
}

#[test]
fn where_not_in_list() {
    let s = select("SELECT * FROM t WHERE id NOT IN (1, 2, 3)");
    assert!(matches!(
        s.selection.as_ref().expect("where"),
        Expr::InList { negated: true, .. }
    ));
}

#[test]
fn where_in_subquery() {
    let s = select("SELECT * FROM t WHERE id IN (SELECT id FROM other WHERE active = true)");
    assert!(s.selection.is_some());
}

#[test]
fn group_by_single_column() {
    let s = select("SELECT dept, COUNT(*) FROM t GROUP BY dept");
    assert_eq!(s.group_by.len(), 1);
}

#[test]
fn group_by_multiple_columns() {
    let s = select("SELECT a, b, COUNT(*) FROM t GROUP BY a, b");
    assert_eq!(s.group_by.len(), 2);
}

#[test]
fn having_clause() {
    let s = select("SELECT dept, COUNT(*) AS cnt FROM t GROUP BY dept HAVING cnt > 5");
    assert!(s.having.is_some());
}

#[test]
fn order_by_asc() {
    let s = select("SELECT * FROM t ORDER BY name ASC");
    assert_eq!(s.order_by.len(), 1);
    assert_eq!(s.order_by[0].direction, Some(SortDirection::Asc));
}

#[test]
fn order_by_desc() {
    let s = select("SELECT * FROM t ORDER BY name DESC");
    assert_eq!(s.order_by[0].direction, Some(SortDirection::Desc));
}

#[test]
fn order_by_multiple_columns() {
    let s = select("SELECT * FROM t ORDER BY a ASC, b DESC");
    assert_eq!(s.order_by.len(), 2);
}

#[test]
fn order_by_without_direction() {
    let s = select("SELECT * FROM t ORDER BY name");
    assert_eq!(s.order_by[0].direction, None);
}

#[test]
fn limit_only() {
    let s = select("SELECT * FROM t LIMIT 10");
    assert!(s.limit.is_some());
    assert!(s.offset.is_none());
}

#[test]
fn limit_with_offset() {
    let s = select("SELECT * FROM t LIMIT 10 OFFSET 20");
    assert!(s.limit.is_some());
    assert!(s.offset.is_some());
}

#[test]
fn single_cte() {
    let q = parse_query("WITH cte AS (SELECT 1) SELECT * FROM cte").unwrap();
    assert_eq!(q.with.len(), 1);
    assert_eq!(&*q.with[0].name.0, "cte");
}

#[test]
fn multiple_ctes() {
    let q = parse_query("WITH a AS (SELECT 1), b AS (SELECT 2) SELECT * FROM a, b").unwrap();
    assert_eq!(q.with.len(), 2);
}

#[test]
fn cte_with_select_body() {
    let q = parse_query(
        "WITH top_users AS (SELECT id FROM users ORDER BY score DESC LIMIT 10) \
         SELECT * FROM top_users",
    )
    .unwrap();
    assert!(q.with[0].query.as_select().is_some());
}

#[test]
fn nested_cte_body() {
    let q = parse_query(
        "WITH outer_cte AS (WITH inner_cte AS (SELECT 1 AS id) SELECT id FROM inner_cte) \
         SELECT * FROM outer_cte",
    )
    .unwrap();
    assert_eq!(q.with.len(), 1);
    let nested = q.with[0].query.as_query().expect("nested query body");
    assert_eq!(nested.with.len(), 1);
    assert_eq!(&*nested.with[0].name.0, "inner_cte");
}

#[test]
fn dependent_cte_chain() {
    let q = parse_query(
        "WITH a AS (SELECT 1 AS id), b AS (SELECT id FROM a), c AS (SELECT id FROM b) \
         SELECT * FROM c",
    )
    .unwrap();
    assert_eq!(q.with.len(), 3);
}

#[test]
fn count_star() {
    let s = select("SELECT COUNT(*) FROM t");
    assert!(matches!(s.projection[0].expr, Expr::Function { .. }));
}

#[test]
fn sum_function() {
    let s = select("SELECT SUM(amount) FROM orders");
    assert!(matches!(s.projection[0].expr, Expr::Function { .. }));
}

#[test]
fn nested_function_call() {
    let s = select("SELECT COALESCE(name, 'anon') FROM users");
    assert!(matches!(s.projection[0].expr, Expr::Function { .. }));
}

#[test]
fn arithmetic_addition() {
    let s = select("SELECT price + tax FROM orders");
    assert!(matches!(
        s.projection[0].expr,
        Expr::Binary { op: BinaryOp::Add, .. }
    ));
}

#[test]
fn arithmetic_multiplication_precedence() {
    let s = select("SELECT 2 + 3 * 4");
    if let Expr::Binary {
        op: BinaryOp::Add,
        right,
        ..
    } = &s.projection[0].expr
    {
        assert!(matches!(*right.as_ref(), Expr::Binary { op: BinaryOp::Mul, .. }));
    } else {
        panic!("expected Add at the top level");
    }
}

#[test]
fn string_concatenation() {
    let s = select("SELECT first_name || ' ' || last_name FROM users");
    assert!(matches!(
        s.projection[0].expr,
        Expr::Binary {
            op: BinaryOp::Concat,
            ..
        }
    ));
}

#[test]
fn cast_expression() {
    let s = select("SELECT CAST(price AS DECIMAL) FROM items");
    assert!(matches!(s.projection[0].expr, Expr::Cast { .. }));
}

#[test]
fn dotted_name_is_multi_part_identifier() {
    let s = select("SELECT address.city FROM users");
    if let Expr::Identifier { parts, .. } = &s.projection[0].expr {
        assert_eq!(parts.len(), 2);
        assert_eq!(&*parts[0].0, "address");
        assert_eq!(&*parts[1].0, "city");
    } else {
        panic!("expected Identifier, got {:?}", s.projection[0].expr);
    }
}

#[test]
fn array_index() {
    let s = select("SELECT tags[1] FROM posts");
    assert!(matches!(s.projection[0].expr, Expr::Index { .. }));
}

#[test]
fn unary_negation() {
    let s = select("SELECT -price FROM items");
    assert!(matches!(
        s.projection[0].expr,
        Expr::Unary { op: UnaryOp::Neg, .. }
    ));
}

#[test]
fn placeholder_positional_dollar() {
    let s = select("SELECT * FROM t WHERE id = $1");
    assert!(matches!(s.selection.as_ref().expect("where"), Expr::Binary { .. }));
}

#[test]
fn named_positional_placeholder() {
    let s = select("SELECT * FROM t WHERE id = $1");
    assert!(s.selection.is_some());
}

#[test]
fn subquery_in_projection() {
    let s = select("SELECT (SELECT MAX(price) FROM items) AS max_price");
    assert!(matches!(
        s.projection[0].expr,
        Expr::Subquery { .. } | Expr::Named { .. }
    ));
}

#[test]
fn comparison_lt() {
    let s = select("SELECT * FROM t WHERE a < b");
    assert!(matches!(s.selection.as_ref().expect("where"), Expr::Binary { op: BinaryOp::Lt, .. }));
}

#[test]
fn comparison_lte() {
    let s = select("SELECT * FROM t WHERE a <= b");
    assert!(matches!(
        s.selection.as_ref().expect("where"),
        Expr::Binary {
            op: BinaryOp::Lte,
            ..
        }
    ));
}

#[test]
fn comparison_gt() {
    let s = select("SELECT * FROM t WHERE a > b");
    assert!(matches!(s.selection.as_ref().expect("where"), Expr::Binary { op: BinaryOp::Gt, .. }));
}

#[test]
fn comparison_gte() {
    let s = select("SELECT * FROM t WHERE a >= b");
    assert!(matches!(
        s.selection.as_ref().expect("where"),
        Expr::Binary {
            op: BinaryOp::Gte,
            ..
        }
    ));
}

#[test]
fn comparison_not_eq() {
    let s = select("SELECT * FROM t WHERE a != b");
    assert!(matches!(
        s.selection.as_ref().expect("where"),
        Expr::Binary {
            op: BinaryOp::NotEq,
            ..
        }
    ));
}

#[test]
fn update_single_column() {
    let stmt = parse_statement("UPDATE users SET name = 'Alice' WHERE id = 1").unwrap();
    assert!(matches!(stmt, Statement::Update(_)));
}

#[test]
fn update_multiple_columns() {
    let stmt = parse_statement("UPDATE users SET name = 'Alice', active = true WHERE id = 1").unwrap();
    if let Statement::Update(u) = stmt {
        assert_eq!(u.assignments.len(), 2);
    } else {
        panic!("expected UPDATE statement");
    }
}

#[test]
fn update_with_returning() {
    let stmt =
        parse_statement("UPDATE users SET name = 'Bob' WHERE id = 2 RETURNING id, name").unwrap();
    if let Statement::Update(u) = stmt {
        assert_eq!(u.returning.len(), 2);
    } else {
        panic!("expected UPDATE statement");
    }
}

#[test]
fn insert_into_table_from_select() {
    let stmt = parse_statement("INSERT INTO TABLE target SELECT id, name FROM src").unwrap();
    assert!(matches!(stmt, Statement::Insert(_)));
}

#[test]
fn insert_with_cte_is_first_class_statement() {
    let stmt = parse_statement(
        "WITH prepared AS (SELECT id, name FROM src) INSERT INTO TABLE target SELECT id, name FROM prepared",
    )
    .unwrap();
    assert!(matches!(stmt, Statement::Insert(_)));
}

#[test]
fn create_table_statement() {
    let stmt = parse_statement("CREATE TABLE public.users (id INT, name TEXT)").unwrap();
    match stmt {
        Statement::Create(create) => {
            assert_eq!(&*create.object_type.0, "TABLE");
            assert_eq!(create.name.len(), 2);
            match create.definition {
                CreateDefinition::Columns(columns) => assert_eq!(columns.len(), 2),
                CreateDefinition::AsQuery(_) => panic!("expected column definition"),
            }
        }
        _ => panic!("expected CREATE statement"),
    }
}

#[test]
fn create_or_replace_view_statement() {
    let stmt =
        parse_statement("CREATE OR REPLACE VIEW analytics.active_users AS SELECT id FROM users")
            .unwrap();
    match stmt {
        Statement::Create(create) => {
            assert!(create.or_replace);
            assert_eq!(&*create.object_type.0, "VIEW");
            match create.definition {
                CreateDefinition::AsQuery(query) => {
                    assert!(query.body.as_select().is_some());
                }
                CreateDefinition::Columns(_) => panic!("expected AS query"),
            }
        }
        _ => panic!("expected CREATE statement"),
    }
}

#[test]
fn alter_table_statement() {
    let stmt = parse_statement("ALTER TABLE users ADD COLUMN email TEXT").unwrap();
    match stmt {
        Statement::Alter(alter) => {
            assert_eq!(&*alter.object_type.0, "TABLE");
            match alter.action {
                AlterAction::AddColumn(column) => {
                    assert_eq!(&*column.name.0, "email");
                    assert_eq!(&*column.data_type.0, "TEXT");
                }
                AlterAction::RenameTo(_) => panic!("expected add-column action"),
            }
        }
        _ => panic!("expected ALTER statement"),
    }
}

#[test]
fn alter_view_rename_statement() {
    let stmt = parse_statement("ALTER VIEW reporting.old_name RENAME TO reporting.new_name").unwrap();
    match stmt {
        Statement::Alter(alter) => match alter.action {
            AlterAction::RenameTo(name) => assert_eq!(&*name[1].0, "new_name"),
            AlterAction::AddColumn(_) => panic!("expected rename action"),
        },
        _ => panic!("expected ALTER statement"),
    }
}

#[test]
fn handles_leading_and_trailing_whitespace() {
    parse_query("   SELECT 1   ").expect("whitespace-padded query");
}

#[test]
fn handles_mixed_case_keywords() {
    parse_query("select * from t where id = 1").expect("lowercase keywords");
    parse_query("SELECT * FROM t WHERE id = 1").expect("uppercase keywords");
    parse_query("Select * From t Where id = 1").expect("mixed case keywords");
}

#[test]
fn handles_multiline_query() {
    parse_query("SELECT\n    id,\n    name\nFROM\n    users\nWHERE\n    active = true")
        .expect("multiline query");
}

#[test]
fn handles_inline_comments() {
    parse_query("SELECT id -- get the id\nFROM users").expect("inline comment");
}

#[test]
fn handles_block_comments() {
    parse_query("SELECT /* all columns */ * FROM t").expect("block comment");
}

#[test]
fn handles_nested_block_comments() {
    parse_query("SELECT /* outer /* inner */ still */ 1").expect("nested block comments");
}

#[test]
fn insert_with_partition() {
    let stmt = parse_statement("INSERT INTO TABLE target PARTITION (year, month) SELECT id FROM src").unwrap();
    if let Statement::Insert(q) = stmt {
        let insert = match &q.body {
            sql_core::ast::QueryBody::Insert(i) => i,
            _ => panic!("expected insert body"),
        };
        assert_eq!(insert.partitioned_by.len(), 2);
        assert_eq!(&*insert.partitioned_by[0].0, "year");
        assert_eq!(&*insert.partitioned_by[1].0, "month");
    } else {
        panic!("expected Insert statement");
    }
}

#[test]
fn create_table_if_not_exists() {
    let stmt = parse_statement("CREATE TABLE IF NOT EXISTS public.users (id INT)").unwrap();
    match stmt {
        Statement::Create(create) => {
            assert!(create.if_not_exists);
            assert_eq!(&*create.object_type.0, "TABLE");
            assert_eq!(create.name.len(), 2);
        }
        _ => panic!("expected CREATE statement"),
    }
}

#[test]
fn alter_table_rename_statement() {
    let stmt = parse_statement("ALTER TABLE old_name RENAME TO new_name").unwrap();
    match stmt {
        Statement::Alter(alter) => {
            assert_eq!(&*alter.object_type.0, "TABLE");
            match alter.action {
                AlterAction::RenameTo(name) => assert_eq!(&*name[0].0, "new_name"),
                AlterAction::AddColumn(_) => panic!("expected rename action"),
            }
        }
        _ => panic!("expected ALTER statement"),
    }
}

#[test]
fn cte_with_update_body() {
    let q = parse_query(
        "WITH updated AS (UPDATE users SET active = true RETURNING id) \
         SELECT * FROM updated",
    )
    .unwrap();
    assert_eq!(q.with.len(), 1);
    assert!(q.with[0].query.as_select().is_none());
}

#[test]
fn cte_with_insert_body() {
    let q = parse_query(
        "WITH cte AS (INSERT INTO TABLE target SELECT 1) SELECT * FROM cte",
    )
    .unwrap();
    assert_eq!(q.with.len(), 1);
    let inner_query = q.with[0].query.as_query().expect("nested query body");
    assert!(matches!(inner_query.body, sql_core::ast::QueryBody::Insert(_)));
}
