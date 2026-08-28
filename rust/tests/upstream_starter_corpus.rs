use sql_parser::parse_query;

fn assert_parses_all(cases: &[(&str, &str)]) {
    for (origin, sql) in cases {
        parse_query(sql).unwrap_or_else(|err| {
            panic!(
                "failed to parse upstream case from {origin}\nSQL: {sql}\n{}",
                err.render(origin, sql)
            )
        });
    }
}

#[test]
fn parses_upstream_bigquery_starter_cases() {
    let cases = [
        (
            "sqlparser_bigquery.rs:251",
            "SELECT OFFSET, EXPLAIN, ANALYZE, SORT, TOP, VIEW FROM T",
        ),
        (
            "sqlparser_bigquery.rs:254",
            "SELECT 1 AS OFFSET, 2 AS EXPLAIN, 3 AS ANALYZE FROM T",
        ),
    ];

    assert_parses_all(&cases);
}

#[test]
fn parses_upstream_postgres_starter_cases() {
    let cases = [(
        "sqlparser_postgres.rs:5400",
        "SELECT REGEXP.REGEXP AS REGEXP FROM REGEXP AS REGEXP WHERE REGEXP.REGEXP",
    )];

    assert_parses_all(&cases);
}

#[test]
fn parses_upstream_snowflake_starter_cases() {
    let cases = [(
        "sqlparser_snowflake.rs:1200",
        "SELECT * FROM (SELECT 1) AS t",
    )];

    assert_parses_all(&cases);
}

#[test]
fn parses_upstream_spark_family_starter_cases() {
    // The current upstream sqlparser-rs tree does not contain a dedicated SparkSQL test module.
    // As a starter, we pull compatible Spark-family syntax from the Hive dialect tests.
    let cases = [
        (
            "sqlparser_hive.rs:292",
            "SELECT a, b FROM db.table_name JOIN a",
        ),
        ("sqlparser_hive.rs:450", "SELECT name filter FROM region"),
    ];

    assert_parses_all(&cases);
}

#[test]
fn documents_missing_upstream_trino_suite() {
    // As of the cloned upstream snapshot used for this import, there is no dedicated
    // `sqlparser_trino.rs` file under `tests/`.
    let known_test_modules = [
        "sqlparser_bigquery.rs",
        "sqlparser_postgres.rs",
        "sqlparser_snowflake.rs",
        "sqlparser_databricks.rs",
        "sqlparser_hive.rs",
    ];

    assert!(
        known_test_modules
            .iter()
            .all(|name| !name.contains("trino"))
    );
}
