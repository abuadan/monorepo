use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use sql_core::analysis::QueryAnalysis;
use sql_core::lexer::tokenize;
use sql_parser::parse_query;

fn baseline_query() -> &'static str {
    "WITH regional_sales AS (
        SELECT region, SUM(amount) AS total
        FROM orders
        WHERE status = 'paid'
        GROUP BY region
    )
    SELECT c.id, c.region, rs.total
    FROM customers c
    LEFT JOIN regional_sales rs ON c.region = rs.region
    WHERE c.active = TRUE
    ORDER BY rs.total DESC
    LIMIT 100"
}

fn nested_cte_heavy_query(depth: usize) -> String {
    let mut sql = String::new();
    sql.push_str("WITH ");
    for idx in 0..depth {
        if idx > 0 {
            sql.push_str(", ");
        }
        if idx == 0 {
            sql.push_str(&format!("cte_{idx} AS (SELECT {idx} AS id)"));
        } else {
            sql.push_str(&format!(
                "cte_{idx} AS (WITH nested_{idx} AS (SELECT id FROM cte_{prev}) SELECT id FROM nested_{idx})",
                prev = idx - 1
            ));
        }
    }
    sql.push_str(&format!(" SELECT * FROM cte_{}", depth - 1));
    sql
}

fn comment_heavy_query(blocks: usize) -> String {
    let mut sql = String::from("SELECT /* header */ id, name");
    for idx in 0..blocks {
        sql.push_str(&format!(
            "\n/* block comment {idx} start\n   detail {idx}\n   nested marker text\n*/\n-- line comment {idx}"
        ));
    }
    sql.push_str("\nFROM users\nWHERE active = TRUE OR deleted = FALSE");
    sql
}

fn bench_pipeline(c: &mut Criterion) {
    let baseline = baseline_query();
    let nested_cte = nested_cte_heavy_query(24);
    let comment_heavy = comment_heavy_query(160);

    let scenarios = [
        ("baseline", baseline),
        ("nested_cte_heavy", nested_cte.as_str()),
        ("comment_heavy", comment_heavy.as_str()),
    ];

    for (label, sql) in scenarios {
        let mut group = c.benchmark_group(format!("sql_pipeline/{label}"));
        group.throughput(Throughput::Bytes(sql.len() as u64));

        group.bench_function(BenchmarkId::new("lex_only", sql.len()), |b| {
            b.iter(|| {
                let tokens = tokenize(black_box(sql)).expect("lex");
                black_box(tokens);
            });
        });

        group.bench_function(BenchmarkId::new("parse_only", sql.len()), |b| {
            b.iter(|| {
                let query = parse_query(black_box(sql)).expect("parse");
                black_box(query);
            });
        });

        group.bench_function(BenchmarkId::new("parse_and_analysis", sql.len()), |b| {
            b.iter(|| {
                let query = parse_query(black_box(sql)).expect("parse");
                let analysis = QueryAnalysis::from_query(&query);
                black_box(analysis);
            });
        });

        group.finish();
    }
}

criterion_group!(benches, bench_pipeline);
criterion_main!(benches);
