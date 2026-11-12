//! `agent-memos ruminate` maintenance pass tests: the daemon's nightly
//! long-cycle entry point (short+long drain, vector index backfill).

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    cognition::rumination::{
        RuminationService, RuminationTriggerDecision, RuminationTriggerEvent, run_maintenance_pass,
    },
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::{
        record::{RecordType, Scope, TruthLayer},
        repository::MemoryRepository,
    },
};
use serde_json::json;

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-ruminate-tests")
        .join(format!("{name}-{unique}"))
        .join("ruminate.sqlite")
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-ruminate-tests")
        .join(format!("{name}-dir-{unique}"))
}

fn ingest_record(db: &Database, uri: &str, content: &str) {
    IngestService::new(db.conn())
        .ingest(IngestRequest {
            source_uri: uri.to_string(),
            source_label: Some(uri.to_string()),
            source_kind: None,
            content: content.to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-11-12T10:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should seed a searchable record");
}

fn seed_user_correction(db: &Database, subject_ref: &str) {
    let occurred_at = (time::OffsetDateTime::now_utc() - time::Duration::minutes(5))
        .format(&time::format_description::well_known::Rfc3339)
        .expect("rfc3339 timestamp");
    let budget_bucket = occurred_at.get(..10).expect("date prefix").to_string();
    let service = RuminationService::new(db.conn());
    let event = RuminationTriggerEvent::from_user_correction(
        subject_ref,
        json!({"self_state": {"nightly_pass_ran": "true"}}),
        occurred_at,
        budget_bucket,
        Some("user-correction-ruminate-1".to_string()),
    );
    assert!(matches!(
        service.schedule(event).expect("correction should enqueue"),
        RuminationTriggerDecision::Enqueued { .. }
    ));
}

fn write_cli_config(path: &Path, db_path: &Path, vector_backend: Option<&str>) {
    let parent = path.parent().expect("config parent");
    fs::create_dir_all(parent).expect("config parent should create");
    let vector_block = vector_backend
        .map(|backend| {
            format!(
                "\n[vector]\nbackend = \"{backend}\"\ntable = \"object_embeddings_vec\"\nsimilarity = \"cosine\"\n"
            )
        })
        .unwrap_or_default();
    fs::write(
        path,
        format!(
            r#"
db_path = "{}"

[retrieval]
mode = "lexical_only"

[embedding]
backend = "disabled"
{vector_block}"#,
            db_path.display()
        ),
    )
    .expect("config should write");
}

fn run_cli(config_path: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_agent-memos"))
        .arg("--config")
        .arg(config_path)
        .args(args)
        .output()
        .expect("binary should run")
}

#[test]
fn maintenance_pass_drains_short_cycle_and_reports() {
    let path = fresh_db_path("lib-pass");
    let db = Database::open(&path).expect("database should open");
    ingest_record(
        &db,
        "memo://project/pass",
        "nightly maintenance drains the rumination queues",
    );
    seed_user_correction(&db, "task://nightly/ruminate");

    let now = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .expect("rfc3339 now");
    let report = run_maintenance_pass(db.conn(), None, &now);

    assert_eq!(report.drained_short_cycle, 1, "the seeded item drains");
    assert_eq!(report.short_cycle_writebacks.len(), 1);
    assert!(
        report.short_cycle_writebacks[0].entry_count > 0,
        "writeback produces local adaptation entries"
    );
    assert_eq!(report.drained_long_cycle, 0, "nothing was queued for lpq");
    assert!(
        report.errors.is_empty(),
        "no drain errors: {:?}",
        report.errors
    );
    assert_eq!(
        report.vector_index_backfilled, 0,
        "no vector table configured"
    );

    let entries = MemoryRepository::new(db.conn())
        .list_local_adaptation_entries("task://nightly/ruminate")
        .expect("adaptations should load");
    assert!(
        entries.iter().any(|entry| entry.key == "nightly_pass_ran"),
        "the correction landed as a local adaptation: {entries:?}"
    );
}

#[test]
fn ruminate_command_drains_and_backfills_the_vector_index() {
    let dir = unique_temp_dir("cli-ruminate");
    let db_path = dir.join("ruminate.sqlite");
    let config_path = dir.join("config.toml");
    write_cli_config(&config_path, &db_path, Some("sqlite_vec"));

    {
        let db = Database::open(&db_path).expect("database should open");
        ingest_record(
            &db,
            "memo://project/ruminate",
            "ruminate backfills vectors written before the index existed",
        );
        seed_user_correction(&db, "task://nightly/ruminate");

        // One embedding row written WITHOUT the vector index (legacy path),
        // so the pass has backfill work to do.
        let repository = MemoryRepository::new(db.conn());
        repository
            .insert_record_embedding(&agent_memos::memory::repository::RecordEmbedding {
                record_id: repository
                    .list_layered_records()
                    .expect("layered records")
                    .first()
                    .map(|layered| layered.record.id.clone())
                    .expect("ingested record id"),
                backend: agent_memos::core::config::EmbeddingBackend::Builtin,
                model: "hash-16".to_string(),
                dimensions: 16,
                embedding: vec![0.1; 16],
                source_text_hash: "test".to_string(),
                created_at: "2026-11-12T10:00:00Z".to_string(),
                updated_at: "2026-11-12T10:00:00Z".to_string(),
            })
            .expect("embedding row should insert");
    }

    let output = run_cli(&config_path, &["ruminate", "--json"]);
    assert!(
        output.status.success(),
        "ruminate should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr),
    );
    let report: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("ruminate --json prints the pass report");
    assert_eq!(report["drained_short_cycle"], 1, "report: {report}");
    assert_eq!(report["vector_index_backfilled"], 1, "report: {report}");

    let db = Database::open(&db_path).expect("database should reopen");
    let vec_rows: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM object_embeddings_vec", [], |row| {
            row.get(0)
        })
        .expect("vec0 rows");
    assert_eq!(vec_rows, 1, "the legacy embedding row got mirrored");

    let spq_items = MemoryRepository::new(db.conn())
        .list_rumination_queue_items("spq")
        .expect("spq items");
    assert!(
        spq_items
            .iter()
            .all(|item| item.status.as_str() == "completed"),
        "drained items complete: {spq_items:?}"
    );
}

#[test]
fn ruminate_command_text_output_reports_counts() {
    let dir = unique_temp_dir("cli-ruminate-text");
    let db_path = dir.join("ruminate-text.sqlite");
    let config_path = dir.join("config.toml");
    write_cli_config(&config_path, &db_path, None);

    let db = Database::open(&db_path).expect("database should open");
    ingest_record(
        &db,
        "memo://project/text",
        "text output stays human readable",
    );
    drop(db);

    let output = run_cli(&config_path, &["ruminate"]);
    assert!(
        output.status.success(),
        "ruminate should succeed on an empty queue: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(text.contains("drained_short_cycle: 0"), "text: {text}");
    assert!(text.contains("drained_long_cycle: 0"), "text: {text}");
    assert!(text.contains("vector_index_backfilled: 0"), "text: {text}");
}
