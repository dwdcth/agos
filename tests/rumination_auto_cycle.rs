//! Rumination auto-loop tests: the default agent-search path feeds its
//! report into the rumination queues (short-cycle writebacks included) and
//! the loop closes back into later assemblies through the adaptive overlay.

use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    agent::orchestration::{AgentSearchOrchestrator, AgentSearchRequest},
    cognition::{
        assembly::{AdaptiveSelfStateProvider, MinimalSelfStateProvider},
        rumination::{
            RuminationService, RuminationTriggerDecision, RuminationTriggerEvent,
            RuminationTriggerKind,
        },
        value::ValueConfig,
    },
    core::config::{Config, RetrievalMode},
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::{
        record::{RecordType, Scope, TruthLayer},
        repository::MemoryRepository,
    },
};
use serde_json::json;

const CLI_SUBJECT: &str = "task://cli/agent-search";

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-rumination-auto-tests")
        .join(format!("{name}-{unique}"))
        .join("rumination-auto.sqlite")
}

fn unique_temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-rumination-auto-tests")
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

/// Seed a pending short-cycle item exactly the way the daemon would after a
/// user correction, so the agent-search drain has real work to write back.
/// Timestamps are relative to the wall clock so the item is claim-ready
/// whenever the test runs.
fn seed_user_correction(db: &Database, subject_ref: &str) {
    let occurred_at = (time::OffsetDateTime::now_utc() - time::Duration::minutes(5))
        .format(&time::format_description::well_known::Rfc3339)
        .expect("rfc3339 timestamp");
    let budget_bucket = occurred_at.get(..10).expect("date prefix").to_string();

    let service = RuminationService::new(db.conn());
    let event = RuminationTriggerEvent::from_user_correction(
        subject_ref,
        json!({
            "self_state": {
                "preferred_answer_style": "cite_before_answer"
            }
        }),
        occurred_at,
        budget_bucket,
        Some("user-correction-auto-1".to_string()),
    );
    assert!(
        matches!(
            service.schedule(event).expect("correction should enqueue"),
            RuminationTriggerDecision::Enqueued { .. }
        ),
        "seeded correction should land in the spq"
    );
}

fn queue_count(db: &Database, tier: &str) -> usize {
    MemoryRepository::new(db.conn())
        .list_rumination_queue_items(tier)
        .expect("queue items should load")
        .len()
}

fn write_cli_config(path: &Path, db_path: &Path, auto_rumination: Option<bool>) {
    let parent = path.parent().expect("config parent");
    fs::create_dir_all(parent).expect("config parent should create");
    let cognition_block = auto_rumination
        .map(|enabled| format!("\n[cognition]\nauto_rumination = {enabled}\n"))
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
{cognition_block}"#,
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
fn cycle_schedules_session_boundary_and_drains_pending_short_cycle() {
    let path = fresh_db_path("lib-cycle");
    let db = Database::open(&path).expect("database should open");
    ingest_record(
        &db,
        "memo://project/rumination",
        "adaptive local state remains explainable through the overlay",
    );
    seed_user_correction(&db, CLI_SUBJECT);

    let config = Config {
        db_path: path.display().to_string(),
        ..Default::default()
    };
    let orchestrator = AgentSearchOrchestrator::with_runtime_config(
        db.conn(),
        AdaptiveSelfStateProvider::new(MinimalSelfStateProvider),
        ValueConfig::default(),
        &config,
    )
    .with_rumination(Box::new(
        agent_memos::agent::orchestration::ServiceRuminationPort::new(db.conn()),
    ));

    let request = AgentSearchRequest::developer_defaults("adaptive local state")
        .with_working_memory_limit(10);
    // Cycle "now" follows the wall clock so the seeded (five minutes old)
    // short-cycle item is claim-ready.
    let now = time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .expect("rfc3339 now");
    let bucket = now.get(..10).expect("date prefix").to_string();
    let result = orchestrator
        .cycle(&request, CLI_SUBJECT, &now, &bucket)
        .expect("cycle should complete");

    // The report gate is a Warning for a well-supported search, so the
    // trigger is the long-cycle session boundary...
    assert!(
        result
            .rumination_triggered
            .iter()
            .any(|trigger| trigger.kind == RuminationTriggerKind::SessionBoundary),
        "cycle should schedule a session-boundary trigger: {:?}",
        result
            .rumination_triggered
            .iter()
            .map(|trigger| trigger.kind)
            .collect::<Vec<_>>()
    );
    assert!(
        queue_count(&db, "lpq") >= 1,
        "the session-boundary trigger must be enqueued persistently"
    );

    // ...while the pending short-cycle user correction drains into local
    // adaptations during the same cycle.
    assert_eq!(
        result.short_cycle_writebacks.len(),
        1,
        "the pre-seeded spq item should drain inside the cycle"
    );
    let repository = MemoryRepository::new(db.conn());
    let entries = repository
        .list_local_adaptation_entries(CLI_SUBJECT)
        .expect("adaptation entries should load");
    assert!(
        entries
            .iter()
            .any(|entry| entry.key == "preferred_answer_style"),
        "the correction should be written back as a local adaptation: {entries:?}"
    );
    let spq_items = repository
        .list_rumination_queue_items("spq")
        .expect("spq items should load");
    assert!(
        spq_items
            .iter()
            .all(|item| item.status.as_str() == "completed"),
        "drained spq items complete: {spq_items:?}"
    );
}

#[test]
fn cli_agent_search_auto_ruminates_and_closes_the_loop() {
    let dir = unique_temp_dir("cli-auto");
    let db_path = dir.join("rumination-auto.sqlite");
    let config_path = dir.join("config.toml");
    // No [cognition] block: auto_rumination defaults to on.
    write_cli_config(&config_path, &db_path, None);

    // Seed: one searchable record plus a pending user correction under the
    // CLI subject, exactly like a daemon-driven correction would.
    {
        let db = Database::open(&db_path).expect("database should open");
        ingest_record(
            &db,
            "memo://project/auto",
            "agent search closes the rumination loop by default",
        );
        seed_user_correction(&db, CLI_SUBJECT);
    }

    let first = run_cli(&config_path, &["agent-search", "rumination loop", "--json"]);
    assert!(
        first.status.success(),
        "agent-search should succeed: stdout={} stderr={}",
        String::from_utf8_lossy(&first.stdout),
        String::from_utf8_lossy(&first.stderr),
    );
    let report: serde_json::Value =
        serde_json::from_slice(&first.stdout).expect("stdout must stay a single report JSON");

    // The report contract is untouched (the rumination summary rides stderr).
    assert!(report["decision"]["gate"].is_object());
    assert!(report["citations"].is_array());
    let stderr_json: serde_json::Value = serde_json::from_slice(&first.stderr)
        .expect("json mode emits the rumination summary as one stderr JSON object");
    assert!(
        stderr_json["triggered"]
            .as_array()
            .is_some_and(|kinds| !kinds.is_empty()),
        "stderr summary lists the rumination trigger: {stderr_json}"
    );

    {
        let db = Database::open(&db_path).expect("database should reopen");
        assert!(
            queue_count(&db, "lpq") >= 1,
            "agent-search enqueued a long-cycle trigger"
        );
        let repository = MemoryRepository::new(db.conn());
        let entries = repository
            .list_local_adaptation_entries(CLI_SUBJECT)
            .expect("adaptations should load");
        assert!(
            entries
                .iter()
                .any(|entry| entry.key == "preferred_answer_style"),
            "the seeded correction drained during agent-search: {entries:?}"
        );
    }

    // Closed loop: a second run assembles through the adaptive overlay, so
    // the writeback surfaces as a self-state fact in the next report.
    let second = run_cli(&config_path, &["agent-search", "rumination loop", "--json"]);
    assert!(second.status.success());
    let second_text = String::from_utf8_lossy(&second.stdout).to_string();
    assert!(
        second_text.contains("self_state:preferred_answer_style"),
        "the drained correction must surface in the next working memory: {second_text}"
    );
}

#[test]
fn cli_agent_search_respects_auto_rumination_off() {
    let dir = unique_temp_dir("cli-off");
    let db_path = dir.join("rumination-off.sqlite");
    let config_path = dir.join("config.toml");
    write_cli_config(&config_path, &db_path, Some(false));

    {
        let db = Database::open(&db_path).expect("database should open");
        ingest_record(
            &db,
            "memo://project/off",
            "rumination stays manual when disabled in config",
        );
        seed_user_correction(&db, CLI_SUBJECT);
    }

    let output = run_cli(
        &config_path,
        &["agent-search", "manual rumination", "--json"],
    );
    assert!(
        output.status.success(),
        "agent-search should still work with auto_rumination off"
    );
    assert!(
        output.stderr.is_empty() || !String::from_utf8_lossy(&output.stderr).contains("triggered"),
        "no rumination summary when the loop is disabled"
    );

    let db = Database::open(&db_path).expect("database should reopen");
    assert_eq!(
        queue_count(&db, "lpq"),
        0,
        "no trigger may be enqueued while auto_rumination is off"
    );
    let repository = MemoryRepository::new(db.conn());
    let spq_items = repository
        .list_rumination_queue_items("spq")
        .expect("spq items should load");
    assert!(
        spq_items
            .iter()
            .all(|item| item.status.as_str() == "queued"),
        "pending spq items must stay queued (no drain): {spq_items:?}"
    );
    assert!(
        repository
            .list_local_adaptation_entries(CLI_SUBJECT)
            .expect("adaptations should load")
            .is_empty(),
        "no short-cycle writeback may happen while disabled"
    );

    // RetrievalMode import stays exercised for config building parity.
    assert_eq!(Config::default().retrieval.mode, RetrievalMode::LexicalOnly);
}
