use std::collections::{BTreeSet, HashSet};

use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use sql_core::keywords::{classify_keyword, keyword_map};

fn probe_words() -> Vec<String> {
    vec![
        "SELECT".into(),
        "FROM".into(),
        "WHERE".into(),
        "JOIN".into(),
        "TABLE".into(),
        "VIEW".into(),
        "create".into(),
        "alter".into(),
        "identifier".into(),
        "customer_id".into(),
        "coalesce".into(),
        "lateral".into(),
        "qualify".into(),
        "rename".into(),
        "offset".into(),
        "window_fn".into(),
    ]
}

fn vec_lookup(keywords: &[&str], value: &str) -> bool {
    let upper = value.to_ascii_uppercase();
    keywords.iter().any(|keyword| *keyword == upper)
}

fn btree_lookup(keywords: &BTreeSet<&'static str>, value: &str) -> bool {
    let upper = value.to_ascii_uppercase();
    keywords.contains(upper.as_str())
}

fn hashset_lookup(keywords: &HashSet<&'static str>, value: &str) -> bool {
    let upper = value.to_ascii_uppercase();
    keywords.contains(upper.as_str())
}

fn bench_keyword_lookup(c: &mut Criterion) {
    let keyword_map = keyword_map();
    let keyword_set: HashSet<_> = keyword_map.keys().copied().collect();
    let mut keyword_vec: Vec<_> = keyword_map.keys().copied().collect();
    keyword_vec.sort_unstable();
    let keyword_btree: BTreeSet<_> = keyword_vec.iter().copied().collect();
    let probes = probe_words();

    let mut group = c.benchmark_group("keyword_lookup");
    group.throughput(Throughput::Elements(probes.len() as u64));

    group.bench_function(BenchmarkId::new("hashmap_classify_keyword", probes.len()), |b| {
        b.iter(|| {
            for probe in &probes {
                black_box(matches!(classify_keyword(black_box(probe)), sql_core::keywords::Keyword::Known(_)));
            }
        });
    });

    group.bench_function(BenchmarkId::new("hashset_membership", probes.len()), |b| {
        b.iter(|| {
            for probe in &probes {
                black_box(hashset_lookup(&keyword_set, black_box(probe)));
            }
        });
    });

    group.bench_function(BenchmarkId::new("btree_membership", probes.len()), |b| {
        b.iter(|| {
            for probe in &probes {
                black_box(btree_lookup(&keyword_btree, black_box(probe)));
            }
        });
    });

    group.bench_function(BenchmarkId::new("vec_linear_scan", probes.len()), |b| {
        b.iter(|| {
            for probe in &probes {
                black_box(vec_lookup(&keyword_vec, black_box(probe)));
            }
        });
    });

    group.finish();
}

criterion_group!(benches, bench_keyword_lookup);
criterion_main!(benches);
