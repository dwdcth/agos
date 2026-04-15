use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::db::Database,
    memory::{
        record::{
            MemoryRecord, Provenance, RecordTimestamp, RecordType, Scope, SourceKind, SourceRef,
            TruthLayer,
        },
        repository::MemoryRepository,
    },
};

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-tests")
        .join(format!("{name}-{unique}"))
        .join("nested")
        .join("foundation.sqlite")
}

fn table_names(path: &Path) -> Vec<String> {
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

fn sample_record() -> MemoryRecord {
    MemoryRecord {
        id: "rec-001".to_string(),
        source: SourceRef {
            uri: "file:///tmp/meeting-notes.md".to_string(),
            kind: SourceKind::Document,
            label: Some("meeting-notes".to_string()),
        },
        timestamp: RecordTimestamp {
            recorded_at: "2026-04-15T09:00:00Z".to_string(),
            created_at: "2026-04-15T09:00:00Z".to_string(),
            updated_at: "2026-04-15T09:00:00Z".to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Observation,
        truth_layer: TruthLayer::T2,
        provenance: Provenance {
            origin: "import".to_string(),
            imported_via: Some("cli".to_string()),
            derived_from: vec!["src-17".to_string()],
        },
        content_text: "SQLite bootstrap decisions stay local-first.".to_string(),
    }
}

#[test]
fn foundation_migration_bootstraps_clean_db() {
    let path = fresh_db_path("bootstrap");
    let parent = path.parent().expect("database path should have parent");
    assert!(!parent.exists(), "test parent directory should start absent");

    let db = Database::open(&path).expect("fresh database should bootstrap");

    assert!(parent.exists(), "open should create parent directories");
    assert_eq!(db.schema_version().expect("schema version"), 1);

    let names = table_names(&path);
    assert_eq!(names, vec!["memory_records"]);
    assert!(
        !names.iter().any(|name| {
            name.contains("fts")
                || name.contains("vec")
                || name.contains("rig")
                || name.contains("search")
        }),
        "phase 1 schema must not introduce later-phase tables: {names:?}"
    );
}

#[test]
fn foundation_migration_reopen_is_idempotent() {
    let path = fresh_db_path("reopen");
    let first = Database::open(&path).expect("first open should succeed");
    assert_eq!(first.schema_version().expect("first schema version"), 1);
    drop(first);

    let second = Database::open(&path).expect("second open should succeed");
    assert_eq!(second.schema_version().expect("second schema version"), 1);
    assert_eq!(table_names(&path), vec!["memory_records"]);
}

#[test]
fn foundation_schema_stays_additive_and_phase_one_only() {
    let path = fresh_db_path("phase-one-only");
    let names = table_names(&path);

    assert!(
        names.contains(&"memory_records".to_string()),
        "foundation schema should include memory_records"
    );
    assert_eq!(names.len(), 1, "phase 1 should keep only base tables");

    let schema_dump = fs::read_to_string(&path).err();
    assert!(
        schema_dump.is_some(),
        "sqlite file should remain binary; schema must be inspected through sqlite metadata"
    );
}

#[test]
fn memory_record_round_trip_preserves_foundation_metadata() {
    let path = fresh_db_path("round-trip");
    let db = Database::open(&path).expect("database should bootstrap");
    let repo = MemoryRepository::new(db.conn());
    let record = sample_record();

    repo.insert_record(&record)
        .expect("record should insert cleanly");

    let loaded = repo
        .get_record(&record.id)
        .expect("record lookup should succeed")
        .expect("record should exist");

    assert_eq!(loaded, record);
}

#[test]
fn memory_record_types_stay_strongly_typed() {
    let record = sample_record();

    assert!(matches!(record.scope, Scope::Project));
    assert!(matches!(record.record_type, RecordType::Observation));
    assert!(matches!(record.truth_layer, TruthLayer::T2));
}

#[test]
fn memory_repository_reads_preserve_metadata_shape() {
    let path = fresh_db_path("metadata-shape");
    let db = Database::open(&path).expect("database should bootstrap");
    let repo = MemoryRepository::new(db.conn());
    let record = sample_record();

    repo.insert_record(&record)
        .expect("record should insert cleanly");

    let listed = repo.list_records().expect("listing should succeed");
    assert_eq!(listed, vec![record.clone()]);

    let counts = repo.scope_counts().expect("scope counts should succeed");
    assert_eq!(counts.len(), 1);
    assert!(matches!(counts[0].scope, Scope::Project));
    assert_eq!(counts[0].count, 1);
    assert_eq!(repo.count_records().expect("count should succeed"), 1);
}
