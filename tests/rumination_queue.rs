use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::core::db::Database;

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-rumination-tests")
        .join(format!("{name}-{unique}"))
        .join("rumination.sqlite")
}

fn table_names(path: &std::path::Path) -> Vec<String> {
    let db = Database::open(path).expect("database should open");
    let mut statement = db
        .conn()
        .prepare(
            "SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .expect("table list statement should prepare");

    statement
        .query_map([], |row| row.get::<_, String>(0))
        .expect("table list query should run")
        .collect::<Result<Vec<_>, _>>()
        .expect("table names should decode")
}

fn column_names(path: &std::path::Path, table: &str) -> Vec<String> {
    let db = Database::open(path).expect("database should open");
    let mut statement = db
        .conn()
        .prepare(&format!("PRAGMA table_info({table})"))
        .expect("table info statement should prepare");

    statement
        .query_map([], |row| row.get::<_, String>(1))
        .expect("table info query should run")
        .collect::<Result<Vec<_>, _>>()
        .expect("column names should decode")
}

#[test]
fn queue_schema_keeps_explicit_spq_and_lpq_tables() {
    let path = fresh_db_path("queue-schema");
    let names = table_names(&path);

    assert!(
        names.contains(&"spq_queue_items".to_string()),
        "spq queue table should exist explicitly: {names:?}"
    );
    assert!(
        names.contains(&"lpq_queue_items".to_string()),
        "lpq queue table should exist explicitly: {names:?}"
    );
    assert!(
        !names.contains(&"rumination_queue_items".to_string()),
        "plan locks explicit dual queues instead of one mixed queue table: {names:?}"
    );

    let spq_columns = column_names(&path, "spq_queue_items");
    let lpq_columns = column_names(&path, "lpq_queue_items");
    assert_eq!(
        spq_columns, lpq_columns,
        "spq/lpq tables should share one mirrored queue item contract"
    );
}
