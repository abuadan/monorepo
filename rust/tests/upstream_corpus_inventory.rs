use std::fs;
use std::path::PathBuf;

fn corpus_file(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("corpus")
        .join("upstream")
        .join(format!("{name}.jsonl"))
}

#[test]
fn harvested_upstream_corpora_exist_and_are_non_empty() {
    for dialect in ["bigquery", "postgres", "databricks", "hive", "snowflake"] {
        let path = corpus_file(dialect);
        let contents = fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("failed to read {}: {err}", path.display()));
        assert!(
            !contents.trim().is_empty(),
            "expected {} to contain harvested upstream cases",
            path.display()
        );
    }
}
