use std::fs;
use std::path::PathBuf;

use sql_parser::parse_query;

fn corpus_file(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("corpus")
        .join("upstream")
        .join(format!("{name}.jsonl"))
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

#[derive(Debug)]
struct Case {
    origin: String,
    sql: String,
}

#[derive(Debug)]
struct DialectReport {
    dialect: &'static str,
    total_cases: usize,
    query_candidates: usize,
    passed: usize,
    failed: usize,
    failures: Vec<String>,
}

fn load_query_cases(dialect: &'static str) -> Vec<Case> {
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
            is_query_candidate(&normalized).then_some(Case {
                origin,
                sql: normalized,
            })
        })
        .collect()
}

fn count_total_cases(dialect: &'static str) -> usize {
    let path = corpus_file(dialect);
    fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()))
        .lines()
        .count()
}

fn collect_report(dialect: &'static str) -> DialectReport {
    let cases = load_query_cases(dialect);
    let mut passed = 0usize;
    let mut failures = Vec::new();

    for case in &cases {
        match parse_query(&case.sql) {
            Ok(_) => passed += 1,
            Err(err) => failures.push(format!(
                "{}\nSQL: {}\n{}",
                case.origin,
                case.sql,
                err.render(&case.origin, &case.sql)
            )),
        }
    }

    DialectReport {
        dialect,
        total_cases: count_total_cases(dialect),
        query_candidates: cases.len(),
        passed,
        failed: failures.len(),
        failures,
    }
}

fn main() {
    let dialects = ["bigquery", "postgres", "databricks", "hive", "snowflake"];
    let reports: Vec<_> = dialects.into_iter().map(collect_report).collect();

    println!("# Upstream Query Coverage");
    println!();
    println!("| Dialect | Harvested | Query Candidates | Passed | Failed | Pass Rate |");
    println!("| --- | ---: | ---: | ---: | ---: | ---: |");
    for report in &reports {
        let pass_rate = if report.query_candidates == 0 {
            0.0
        } else {
            (report.passed as f64 / report.query_candidates as f64) * 100.0
        };
        println!(
            "| {} | {} | {} | {} | {} | {:.1}% |",
            report.dialect,
            report.total_cases,
            report.query_candidates,
            report.passed,
            report.failed,
            pass_rate
        );
    }

    for report in &reports {
        println!();
        println!("## {}", report.dialect);
        println!();
        println!(
            "Harvested: {}. Query candidates: {}. Passed: {}. Failed: {}.",
            report.total_cases, report.query_candidates, report.passed, report.failed
        );
        if report.failures.is_empty() {
            println!();
            println!("All current query candidates pass.");
            continue;
        }
        println!();
        println!("Representative failures:");
        for failure in report.failures.iter().take(3) {
            println!();
            println!("```text");
            println!("{failure}");
            println!("```");
        }
    }
}
