use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::db::Database,
    memory::{
        record::{
            ChunkAnchor, ChunkMetadata, MemoryRecord, Provenance, RecordTimestamp, RecordType,
            Scope, SourceKind, SourceRef, TruthLayer, ValidityWindow,
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

fn object_names(path: &Path, object_type: &str) -> Vec<String> {
    let db = Database::open(path).expect("database should open");
    let mut statement = db
        .conn()
        .prepare(
            "SELECT name FROM sqlite_master WHERE type = ?1 AND name NOT LIKE 'sqlite_%' ORDER BY name",
        )
        .expect("object list statement should prepare");

    statement
        .query_map([object_type], |row| row.get::<_, String>(0))
        .expect("object list query should run")
        .collect::<Result<Vec<_>, _>>()
        .expect("object names should decode")
}

fn table_columns(path: &Path, table: &str) -> Vec<String> {
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

fn table_indexes(path: &Path, table: &str) -> Vec<String> {
    let db = Database::open(path).expect("database should open");
    let mut statement = db
        .conn()
        .prepare(&format!("PRAGMA index_list({table})"))
        .expect("index list statement should prepare");

    statement
        .query_map([], |row| row.get::<_, String>(1))
        .expect("index list query should run")
        .collect::<Result<Vec<_>, _>>()
        .expect("index names should decode")
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
        chunk: Some(ChunkMetadata {
            chunk_index: 0,
            chunk_count: 2,
            anchor: ChunkAnchor::LineRange {
                start_line: 1,
                end_line: 2,
            },
            content_hash: "sha256:abc123".to_string(),
        }),
        validity: ValidityWindow {
            valid_from: Some("2026-04-01T00:00:00Z".to_string()),
            valid_to: None,
        },
    }
}

#[test]
fn foundation_migration_bootstraps_clean_db() {
    let path = fresh_db_path("bootstrap");
    let parent = path.parent().expect("database path should have parent");
    assert!(!parent.exists(), "test parent directory should start absent");

    let db = Database::open(&path).expect("fresh database should bootstrap");

    assert!(parent.exists(), "open should create parent directories");
    assert_eq!(db.schema_version().expect("schema version"), 3);

    let names = table_names(&path);
    assert!(
        names.contains(&"memory_records".to_string()),
        "authority table should exist: {names:?}"
    );
    assert!(
        names.contains(&"memory_records_fts".to_string()),
        "lexical sidecar should exist: {names:?}"
    );
    assert!(
        !names.iter().any(|name| {
            name.contains("vec") || name.contains("rig")
        }),
        "lexical plan should not introduce semantic or agent tables: {names:?}"
    );
}

#[test]
fn foundation_migration_reopen_is_idempotent() {
    let path = fresh_db_path("reopen");
    let first = Database::open(&path).expect("first open should succeed");
    assert_eq!(first.schema_version().expect("first schema version"), 3);
    drop(first);

    let second = Database::open(&path).expect("second open should succeed");
    assert_eq!(second.schema_version().expect("second schema version"), 3);
    let names = table_names(&path);
    assert!(names.contains(&"memory_records".to_string()));
    assert!(names.contains(&"memory_records_fts".to_string()));
}

#[test]
fn foundation_schema_stays_additive_with_ingest_columns_and_indexes() {
    let path = fresh_db_path("phase-one-only");
    let names = table_names(&path);
    let columns = table_columns(&path, "memory_records");
    let indexes = table_indexes(&path, "memory_records");
    let triggers = object_names(&path, "trigger");

    assert!(
        names.contains(&"memory_records".to_string()),
        "foundation schema should include memory_records"
    );
    assert!(
        columns.contains(&"chunk_index".to_string())
            && columns.contains(&"chunk_count".to_string())
            && columns.contains(&"chunk_anchor_json".to_string())
            && columns.contains(&"content_hash".to_string())
            && columns.contains(&"valid_from".to_string())
            && columns.contains(&"valid_to".to_string()),
        "memory_records should expose additive ingest columns: {columns:?}"
    );
    assert!(
        indexes.contains(&"idx_memory_records_scope_recorded_at".to_string())
            && indexes.contains(&"idx_memory_records_truth_layer_recorded_at".to_string())
            && indexes.contains(&"idx_memory_records_source_chunk_order".to_string())
            && indexes.contains(&"idx_memory_records_validity_window".to_string()),
        "memory_records should retain phase 1 indexes and add ingest indexes: {indexes:?}"
    );
    assert!(
        triggers.contains(&"memory_records_ai".to_string())
            && triggers.contains(&"memory_records_ad".to_string())
            && triggers.contains(&"memory_records_au".to_string()),
        "lexical sidecar should stay synchronized via triggers: {triggers:?}"
    );

    let schema_dump = fs::read_to_string(&path).err();
    assert!(
        schema_dump.is_some(),
        "sqlite file should remain binary; schema must be inspected through sqlite metadata"
    );
}

#[test]
fn lexical_sidecar_rebuilds_from_authority_rows() {
    let path = fresh_db_path("lexical-rebuild");
    let db = Database::open(&path).expect("database should bootstrap");
    let repo = MemoryRepository::new(db.conn());
    let record = sample_record();

    repo.insert_record(&record)
        .expect("record should insert cleanly");

    let initial_count: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM memory_records_fts", [], |row| row.get(0))
        .expect("fts row count should load");
    assert_eq!(initial_count, 1, "triggers should index inserted authority rows");

    db.conn()
        .execute("DELETE FROM memory_records_fts", [])
        .expect("fts rows should be clearable for rebuild");
    let cleared_count: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM memory_records_fts", [], |row| row.get(0))
        .expect("cleared fts row count should load");
    assert_eq!(cleared_count, 0, "fts rows should be cleared before rebuild");

    db.conn()
        .execute("INSERT INTO memory_records_fts(memory_records_fts) VALUES('rebuild')", [])
        .expect("fts rebuild helper should succeed");

    let rebuilt_count: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM memory_records_fts", [], |row| row.get(0))
        .expect("rebuilt fts row count should load");
    assert_eq!(rebuilt_count, 1, "rebuild should repopulate from authority rows");
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
    assert!(matches!(
        record.chunk.as_ref().expect("chunk metadata should exist").anchor,
        ChunkAnchor::LineRange { .. }
    ));
    assert_eq!(
        record.validity.valid_from.as_deref(),
        Some("2026-04-01T00:00:00Z")
    );
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
