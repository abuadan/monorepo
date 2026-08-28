use std::fs;
use std::path::PathBuf;

use sql_parser::parse_query;

fn corpus_file(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("corpus")
        .join("upstream")
        .join(format!("{name}.jsonl"))
}

fn load_query_cases(dialect: &str) -> Vec<(String, String)> {
    let path = corpus_file(dialect);
    let contents = fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));

    contents
        .lines()
        .filter_map(|line| {
            let sql = extract_json_string_field(line, "sql")?;
            let origin = format!(
                "{}:{}",
                extract_json_string_field(line, "source_file")?,
                extract_json_number_field(line, "line")?
            );
            let normalized = sql.trim().to_string();
            is_query_candidate(&normalized).then_some((origin, normalized))
        })
        .collect()
}

fn extract_json_string_field(line: &str, key: &str) -> Option<String> {
    let needle = format!("\"{key}\": ");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    if !rest.starts_with('"') {
        return None;
    }
    let mut escaped = false;
    let mut out = String::new();
    for ch in rest[1..].chars() {
        if escaped {
            out.push(match ch {
                'n' => '\n',
                'r' => '\r',
                't' => '\t',
                '"' => '"',
                '\\' => '\\',
                other => other,
            });
            escaped = false;
            continue;
        }
        match ch {
            '\\' => escaped = true,
            '"' => return Some(out),
            other => out.push(other),
        }
    }
    None
}

fn extract_json_number_field(line: &str, key: &str) -> Option<usize> {
    let needle = format!("\"{key}\": ");
    let start = line.find(&needle)? + needle.len();
    let rest = &line[start..];
    let digits: String = rest.chars().take_while(|ch| ch.is_ascii_digit()).collect();
    digits.parse().ok()
}

fn is_query_candidate(sql: &str) -> bool {
    let upper = sql.trim_start().to_ascii_uppercase();
    (upper.starts_with("SELECT") || upper.starts_with("WITH"))
        && !sql.contains('{')
        && !sql.contains("STRUCT<")
        && !sql.contains("@@")
        && !sql.contains("TABLE(")
        && !sql.contains("FLATTEN(")
        && !sql.contains("SELECT AS STRUCT")
        && !sql.contains("SELECT AS VALUE")
        && !sql.contains("TIMESTAMP AS OF")
        && !sql.contains("VERSION AS OF")
        && !sql.contains("LATERAL VIEW")
        && !sql.contains("SORT BY")
        && !sql.contains("DISTRIBUTE BY")
        && !sql.contains("CLUSTER BY")
        && !sql.contains("TABLESAMPLE")
}

fn assert_reasonable_query_coverage(dialect: &str, minimum_passes: usize) {
    let cases = load_query_cases(dialect);
    let mut passed = 0usize;
    let mut failed = Vec::new();

    for (origin, sql) in &cases {
        match parse_query(sql) {
            Ok(_) => passed += 1,
            Err(err) => failed.push(format!("{origin}\nSQL: {sql}\n{}", err.render(origin, sql))),
        }
    }

    assert!(
        passed >= minimum_passes,
        "expected at least {minimum_passes} passing query cases for {dialect}, got {passed} / {}.\n{}",
        cases.len(),
        failed.into_iter().take(3).collect::<Vec<_>>().join("\n\n")
    );
}

#[test]
fn upstream_bigquery_query_coverage() {
    assert_reasonable_query_coverage("bigquery", 2);
}

#[test]
fn upstream_postgres_query_coverage() {
    assert_reasonable_query_coverage("postgres", 3);
}

#[test]
fn upstream_databricks_query_coverage() {
    assert_reasonable_query_coverage("databricks", 1);
}

#[test]
fn upstream_hive_query_coverage() {
    assert_reasonable_query_coverage("hive", 2);
}

#[test]
fn upstream_snowflake_query_coverage() {
    assert_reasonable_query_coverage("snowflake", 2);
}
