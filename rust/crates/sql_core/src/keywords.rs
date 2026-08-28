use std::collections::HashMap;
use std::sync::OnceLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Keyword {
    NoKeyword,
    Known(&'static str),
}

const ALL: &[&str] = &[
    "ALL",
    "AND",
    "ANY",
    "APPEND_ONLY",
    "AS",
    "ASC",
    "ASOF",
    "AT",
    "AVG",
    "BEFORE",
    "BERNOULLI",
    "BLOCK",
    "BY",
    "CAST",
    "CHANGES",
    "COLUMN",
    "COUNT",
    "CREATE",
    "CROSS",
    "DEFAULT",
    "DESC",
    "DISTINCT",
    "END",
    "EXCLUDE",
    "EXISTS",
    "FALSE",
    "FN",
    "FOR",
    "FROM",
    "FULL",
    "GROUP",
    "GROUPING",
    "HAVING",
    "ID",
    "IF",
    "IGNORE",
    "ILIKE",
    "IN",
    "INNER",
    "INSERT",
    "INTO",
    "JOIN",
    "LEFT",
    "LIMIT",
    "MB",
    "MAX",
    "MIN",
    "NO",
    "NOT",
    "NULL",
    "NULLS",
    "OFFSET",
    "ON",
    "OPENJSON",
    "OR",
    "ORDER",
    "ORDINALITY",
    "OUTER",
    "OVER",
    "PARTITION",
    "PIVOT",
    "QUALIFY",
    "RENAME",
    "REPEATABLE",
    "REPLACE",
    "RESPECT",
    "RETURNING",
    "RIGHT",
    "ROW",
    "ROWS",
    "SAMPLE",
    "SEED",
    "SELECT",
    "SET",
    "SETS",
    "SQL",
    "STRUCT",
    "SUM",
    "SYSTEM",
    "TABLE",
    "TABLESAMPLE",
    "TIES",
    "TIME",
    "TO",
    "TRUE",
    "UNION",
    "UNNEST",
    "UPDATE",
    "USING",
    "VALUE",
    "VALUES",
    "VIEW",
    "WHERE",
    "WITH",
    "ALTER",
    "ADD",
];

const RESERVED_FOR_TABLE_ALIAS: &[&str] = &[
    "CROSS", "FULL", "GROUP", "HAVING", "INNER", "JOIN", "LEFT", "LIMIT", "OFFSET", "ON",
    "ORDER", "RIGHT", "USING", "WHERE", "WITH",
];

const RESERVED_FOR_COLUMN_ALIAS: &[&str] = &[
    "FROM", "GROUP", "HAVING", "LIMIT", "OFFSET", "ORDER", "WHERE",
];

const RESERVED_FOR_TABLE_FACTOR: &[&str] = &[
    "AT",
    "BEFORE",
    "CHANGES",
    "END",
    "GROUP",
    "HAVING",
    "JOIN",
    "LIMIT",
    "OFFSET",
    "ON",
    "ORDER",
    "PIVOT",
    "SAMPLE",
    "TABLESAMPLE",
    "WHERE",
];

const RESERVED_FOR_IDENTIFIER: &[&str] = &[
    "ALL",
    "AND",
    "AS",
    "ASC",
    "BY",
    "CREATE",
    "CROSS",
    "DESC",
    "DISTINCT",
    "FALSE",
    "FOR",
    "FROM",
    "FULL",
    "GROUP",
    "HAVING",
    "IN",
    "INNER",
    "INSERT",
    "INTO",
    "JOIN",
    "LEFT",
    "LIMIT",
    "NOT",
    "NULL",
    "ON",
    "OR",
    "ORDER",
    "PARTITION",
    "REPLACE",
    "RETURNING",
    "RIGHT",
    "SELECT",
    "SET",
    "TABLE",
    "TRUE",
    "UNION",
    "UPDATE",
    "USING",
    "WHERE",
    "WITH",
    "ALTER",
];

pub fn classify_keyword(value: &str) -> Keyword {
    keyword_map()
        .get(value.to_ascii_uppercase().as_str())
        .copied()
        .unwrap_or(Keyword::NoKeyword)
}

pub fn is_reserved_for_table_alias(keyword: Keyword) -> bool {
    matches!(keyword, Keyword::Known(name) if RESERVED_FOR_TABLE_ALIAS.contains(&name))
}

pub fn is_reserved_for_column_alias(keyword: Keyword) -> bool {
    matches!(keyword, Keyword::Known(name) if RESERVED_FOR_COLUMN_ALIAS.contains(&name))
}

pub fn is_reserved_for_table_factor(keyword: Keyword) -> bool {
    matches!(keyword, Keyword::Known(name) if RESERVED_FOR_TABLE_FACTOR.contains(&name))
}

pub fn is_reserved_for_identifier(keyword: Keyword) -> bool {
    matches!(keyword, Keyword::Known(name) if RESERVED_FOR_IDENTIFIER.contains(&name))
}

pub fn keyword_map() -> &'static HashMap<&'static str, Keyword> {
    static KEYWORDS: OnceLock<HashMap<&'static str, Keyword>> = OnceLock::new();
    KEYWORDS.get_or_init(|| {
        ALL.iter()
            .copied()
            .map(|kw| (kw, Keyword::Known(kw)))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::{Keyword, classify_keyword, is_reserved_for_identifier, keyword_map};

    #[test]
    fn classifies_statement_keywords_used_by_macros() {
        for keyword in ["SELECT", "INSERT", "UPDATE", "CREATE", "ALTER", "TABLE", "VIEW"] {
            assert!(matches!(classify_keyword(keyword), Keyword::Known(value) if value == keyword));
        }
    }

    #[test]
    fn reserves_core_statement_keywords_for_identifiers() {
        for keyword in ["SELECT", "INSERT", "UPDATE", "CREATE", "ALTER"] {
            assert!(is_reserved_for_identifier(classify_keyword(keyword)));
        }
    }

    #[test]
    fn keyword_map_contains_statement_keywords() {
        for keyword in ["SELECT", "INSERT", "UPDATE", "CREATE", "ALTER", "TABLE", "VIEW"] {
            assert!(keyword_map().contains_key(keyword));
        }
    }
}
