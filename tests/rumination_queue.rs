use std::{
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    agent::orchestration::{AgentSearchReport, RetrievalStepReport},
    cognition::{
        action::{ActionBranch, ActionCandidate, ActionKind},
        metacog::GateDecision,
        report::{DecisionReport, GateReport, ScoredBranchReport},
        rumination::{
            RuminationQueueTier, RuminationService, RuminationTriggerDecision,
            RuminationTriggerEvent, RuminationTriggerKind,
        },
        value::{ProjectedScore, ValueConfig, ValueVector},
        working_memory::{
            EvidenceFragment, MetacognitiveFlag, PresentFrame, SelfStateSnapshot, TruthContext,
            WorkingMemory,
        },
    },
    core::db::Database,
    memory::{
        record::{
            ChunkAnchor, ChunkMetadata, MemoryRecord, Provenance, RecordTimestamp, RecordType,
            Scope, SourceKind, SourceRef, TruthLayer, ValidityWindow,
        },
        repository::MemoryRepository,
    },
    search::{ChannelContribution, Citation, ResultTrace, ScoreBreakdown, SearchFilters},
};

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

fn column_names(path: &Path, table: &str) -> Vec<String> {
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

fn sample_record(id: &str, source_uri: &str) -> MemoryRecord {
    MemoryRecord {
        id: id.to_string(),
        source: SourceRef {
            uri: source_uri.to_string(),
            kind: SourceKind::Note,
            label: Some(format!("label-{id}")),
        },
        timestamp: RecordTimestamp {
            recorded_at: "2026-04-16T00:00:00Z".to_string(),
            created_at: "2026-04-16T00:00:00Z".to_string(),
            updated_at: "2026-04-16T00:00:00Z".to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Decision,
        truth_layer: TruthLayer::T2,
        provenance: Provenance {
            origin: "test".to_string(),
            imported_via: None,
            derived_from: vec!["seed".to_string()],
        },
        content_text: format!("content for {id}"),
        chunk: Some(ChunkMetadata {
            chunk_index: 0,
            chunk_count: 1,
            anchor: ChunkAnchor::LineRange {
                start_line: 1,
                end_line: 3,
            },
            content_hash: format!("hash-{id}"),
        }),
        validity: ValidityWindow {
            valid_from: Some("2026-04-15T00:00:00Z".to_string()),
            valid_to: None,
        },
    }
}

fn sample_fragment(record_id: &str, source_uri: &str) -> EvidenceFragment {
    let record = sample_record(record_id, source_uri);
    let citation = Citation::from_record(&record).expect("chunk metadata should exist");
    EvidenceFragment {
        record_id: record.id,
        snippet: "rig stays orchestration only".to_string(),
        citation,
        provenance: record.provenance,
        truth_context: TruthContext {
            truth_layer: TruthLayer::T2,
            t3_state: None,
            open_review_ids: Vec::new(),
            open_candidate_ids: Vec::new(),
        },
        dsl: None,
        trace: ResultTrace {
            matched_query: "rig boundary".to_string(),
            query_strategies: Vec::new(),
            channel_contribution: ChannelContribution::LexicalOnly,
            applied_filters: SearchFilters::default(),
            attention: None,
        },
        score: ScoreBreakdown {
            lexical_raw: -2.0,
            lexical_base: 0.33,
            keyword_bonus: 0.02,
            importance_bonus: 0.08,
            recency_bonus: 0.03,
            attention_bonus: 0.0,
            final_score: 0.46,
        },
    }
}

fn sample_working_memory() -> WorkingMemory {
    let fragment = sample_fragment("record-primary", "memo://project/rig-boundary");
    let branch = ActionBranch::new(
        ActionCandidate::new(ActionKind::Epistemic, "collect more evidence")
            .with_intent("retrieve more support before acting"),
    )
    .with_supporting_evidence(vec![fragment.clone()]);

    WorkingMemory {
        present: PresentFrame {
            world_fragments: vec![fragment],
            self_state: SelfStateSnapshot {
                task_context: Some("protect the thin rig boundary".to_string()),
                capability_flags: vec!["lexical_search_ready".to_string()],
                readiness_flags: vec!["agent_search_ready".to_string()],
                facts: Vec::new(),
            },
            active_goal: None,
            active_risks: vec!["ungated output".to_string()],
            metacog_flags: vec![MetacognitiveFlag {
                code: "trace_required".to_string(),
                detail: Some("all agent output needs citations".to_string()),
            }],
        },
        branches: vec![branch],
    }
}

fn sample_agent_search_report() -> AgentSearchReport {
    let working_memory = sample_working_memory();
    let selected_branch = ScoredBranchReport {
        branch: working_memory.branches[0].clone(),
        value: ValueVector {
            goal_progress: 0.40,
            information_gain: 0.95,
            risk_avoidance: 0.60,
            resource_efficiency: 0.50,
            agent_robustness: 0.75,
        },
        projected: ProjectedScore {
            final_score: 0.71,
            weight_snapshot: ValueConfig::default(),
        },
    };

    let primary = sample_record("record-primary", "memo://project/rig-boundary");
    let secondary = sample_record("record-secondary", "memo://project/gate-diagnostics");

    AgentSearchReport {
        working_memory,
        decision: DecisionReport {
            scored_branches: vec![selected_branch.clone()],
            selected_branch: Some(selected_branch),
            gate: GateReport {
                decision: GateDecision::Warning,
                diagnostics: vec!["bounded local orchestration".to_string()],
                rejected_branch: None,
                regulative_branch: None,
                safe_response: None,
                autonomy_paused: false,
            },
            active_risks: vec!["ungated output".to_string()],
            metacog_flags: vec![MetacognitiveFlag {
                code: "trace_required".to_string(),
                detail: Some("all agent output needs citations".to_string()),
            }],
        },
        retrieval_steps: vec![
            RetrievalStepReport {
                query: "rig boundary".to_string(),
                applied_filters: SearchFilters::default(),
                result_count: 1,
                citations: vec![Citation::from_record(&primary).expect("citation should build")],
            },
            RetrievalStepReport {
                query: "gate diagnostics".to_string(),
                applied_filters: SearchFilters::default(),
                result_count: 1,
                citations: vec![Citation::from_record(&secondary).expect("citation should build")],
            },
        ],
        citations: vec![
            Citation::from_record(&primary).expect("citation should build"),
            Citation::from_record(&secondary).expect("citation should build"),
        ],
        executed_steps: 2,
        step_limit: 3,
    }
}

fn sample_veto_decision_report() -> DecisionReport {
    DecisionReport {
        scored_branches: Vec::new(),
        selected_branch: None,
        gate: GateReport {
            decision: GateDecision::HardVeto,
            diagnostics: vec!["unsafe_action".to_string()],
            rejected_branch: None,
            regulative_branch: None,
            safe_response: Some("pause execution".to_string()),
            autonomy_paused: false,
        },
        active_risks: vec!["unsafe_action".to_string()],
        metacog_flags: vec![MetacognitiveFlag {
            code: "human_review_required".to_string(),
            detail: Some("vetoed by metacognition".to_string()),
        }],
    }
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

#[test]
fn scheduler_routes_locked_trigger_classes_into_explicit_queues() {
    let path = fresh_db_path("queue-routing");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());
    let service = RuminationService::new(db.conn());

    let veto = RuminationTriggerEvent::from_decision_report(
        RuminationTriggerKind::MetacogVeto,
        "task://unsafe-change",
        &sample_veto_decision_report(),
        "2026-04-16T10:00:00Z",
        "2026-04-16T10",
        Some("2026-04-16T10:05:00Z".to_string()),
        Some("decision-report-1".to_string()),
    )
    .expect("decision report should normalize");
    let session_boundary = RuminationTriggerEvent::from_agent_search_report(
        RuminationTriggerKind::SessionBoundary,
        "session://alpha",
        &sample_agent_search_report(),
        "2026-04-16T10:01:00Z",
        "2026-04-16T10",
        None,
        Some("agent-report-1".to_string()),
    )
    .expect("agent report should normalize");

    let spq_decision = service.schedule(veto).expect("spq event should schedule");
    let lpq_decision = service
        .schedule(session_boundary)
        .expect("lpq event should schedule");

    assert!(matches!(
        spq_decision,
        RuminationTriggerDecision::Enqueued {
            tier: RuminationQueueTier::Spq,
            ..
        }
    ));
    assert!(matches!(
        lpq_decision,
        RuminationTriggerDecision::Enqueued {
            tier: RuminationQueueTier::Lpq,
            ..
        }
    ));

    let spq_items = repo
        .list_rumination_queue_items("spq")
        .expect("spq items should load");
    let lpq_items = repo
        .list_rumination_queue_items("lpq")
        .expect("lpq items should load");

    assert_eq!(spq_items.len(), 1);
    assert_eq!(lpq_items.len(), 1);
    assert_eq!(spq_items[0].trigger_kind, "metacog_veto");
    assert_eq!(lpq_items[0].trigger_kind, "session_boundary");
    assert_eq!(spq_items[0].status.as_str(), "queued");
    assert_eq!(lpq_items[0].status.as_str(), "queued");

    let spq_state = repo
        .get_rumination_trigger_state("spq", &spq_items[0].dedupe_key)
        .expect("spq trigger state should load")
        .expect("spq trigger state should exist");
    let lpq_state = repo
        .get_rumination_trigger_state("lpq", &lpq_items[0].dedupe_key)
        .expect("lpq trigger state should load")
        .expect("lpq trigger state should exist");

    assert_eq!(spq_state.last_decision, "enqueued");
    assert_eq!(lpq_state.last_decision, "enqueued");
    assert_eq!(
        spq_state.last_item_id.as_deref(),
        Some(spq_items[0].item_id.as_str())
    );
    assert_eq!(
        lpq_state.last_item_id.as_deref(),
        Some(lpq_items[0].item_id.as_str())
    );
}

#[test]
fn scheduler_dedupes_repeated_active_triggers() {
    let path = fresh_db_path("queue-dedupe");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());
    let service = RuminationService::new(db.conn());
    let report = sample_agent_search_report();

    let first = RuminationTriggerEvent::from_agent_search_report(
        RuminationTriggerKind::ActionFailure,
        "task://same-dedupe",
        &report,
        "2026-04-16T10:00:00Z",
        "2026-04-16T10",
        Some("2026-04-16T10:05:00Z".to_string()),
        Some("agent-report-dup".to_string()),
    )
    .expect("event should normalize");
    let second = RuminationTriggerEvent::from_agent_search_report(
        RuminationTriggerKind::ActionFailure,
        "task://same-dedupe",
        &report,
        "2026-04-16T10:01:00Z",
        "2026-04-16T10",
        Some("2026-04-16T10:05:00Z".to_string()),
        Some("agent-report-dup".to_string()),
    )
    .expect("event should normalize");

    let first_decision = service
        .schedule(first)
        .expect("first event should schedule");
    let second_decision = service
        .schedule(second)
        .expect("second event should return a dedupe decision");

    assert!(matches!(
        first_decision,
        RuminationTriggerDecision::Enqueued {
            tier: RuminationQueueTier::Spq,
            ..
        }
    ));
    assert!(matches!(
        second_decision,
        RuminationTriggerDecision::Deduped {
            tier: RuminationQueueTier::Spq,
            ..
        }
    ));

    let items = repo
        .list_rumination_queue_items("spq")
        .expect("spq items should load");
    assert_eq!(items.len(), 1, "duplicate trigger should not enqueue twice");

    let state = repo
        .get_rumination_trigger_state("spq", &items[0].dedupe_key)
        .expect("trigger state should load")
        .expect("trigger state should exist");
    assert_eq!(state.last_decision, "deduped");
}

#[test]
fn scheduler_enforces_cooldown_and_budget_caps_durably() {
    let path = fresh_db_path("queue-throttle");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());
    let service = RuminationService::with_budget_limits(db.conn(), 1, 1);
    let report = sample_agent_search_report();

    let first = RuminationTriggerEvent::from_agent_search_report(
        RuminationTriggerKind::ActionFailure,
        "task://cooldown-1",
        &report,
        "2026-04-16T10:00:00Z",
        "2026-04-16T10",
        Some("2026-04-16T10:05:00Z".to_string()),
        Some("report-a".to_string()),
    )
    .expect("first event should normalize")
    .with_cooldown_key("task://shared-cooldown");
    let cooldown_blocked = RuminationTriggerEvent::from_agent_search_report(
        RuminationTriggerKind::UserCorrection,
        "task://cooldown-2",
        &report,
        "2026-04-16T10:01:00Z",
        "2026-04-16T10",
        Some("2026-04-16T10:06:00Z".to_string()),
        Some("report-b".to_string()),
    )
    .expect("cooldown event should normalize")
    .with_cooldown_key("task://shared-cooldown");
    let budget_blocked = RuminationTriggerEvent::from_agent_search_report(
        RuminationTriggerKind::UserCorrection,
        "task://budget-hit",
        &report,
        "2026-04-16T10:02:00Z",
        "2026-04-16T10",
        None,
        Some("report-c".to_string()),
    )
    .expect("budget event should normalize")
    .with_cooldown_key("task://fresh-cooldown");
    let next_bucket = RuminationTriggerEvent::from_agent_search_report(
        RuminationTriggerKind::UserCorrection,
        "task://next-bucket",
        &report,
        "2026-04-16T11:00:00Z",
        "2026-04-16T11",
        None,
        Some("report-d".to_string()),
    )
    .expect("next bucket event should normalize")
    .with_cooldown_key("task://fresh-cooldown");

    let first_decision = service.schedule(first).expect("first event should enqueue");
    let cooldown_decision = service
        .schedule(cooldown_blocked)
        .expect("cooldown should be evaluated");
    let budget_decision = service
        .schedule(budget_blocked)
        .expect("budget cap should be evaluated");
    let next_bucket_decision = service
        .schedule(next_bucket)
        .expect("next bucket should enqueue");

    assert!(matches!(
        first_decision,
        RuminationTriggerDecision::Enqueued { .. }
    ));
    assert!(matches!(
        cooldown_decision,
        RuminationTriggerDecision::CooldownBlocked {
            tier: RuminationQueueTier::Spq,
            ..
        }
    ));
    assert!(matches!(
        budget_decision,
        RuminationTriggerDecision::BudgetBlocked {
            tier: RuminationQueueTier::Spq,
            ..
        }
    ));
    assert!(matches!(
        next_bucket_decision,
        RuminationTriggerDecision::Enqueued {
            tier: RuminationQueueTier::Spq,
            ..
        }
    ));

    let spq_items = repo
        .list_rumination_queue_items("spq")
        .expect("spq items should load");
    assert_eq!(
        spq_items.len(),
        2,
        "only the first and next-bucket items should enqueue"
    );

    let blocked_state = repo
        .get_rumination_trigger_state("spq", "user_correction:task://budget-hit:report-c")
        .expect("blocked state should load")
        .expect("blocked state should exist");
    assert_eq!(blocked_state.last_decision, "budget_blocked");
}

#[test]
fn scheduler_claims_spq_before_ready_lpq_items() {
    let path = fresh_db_path("queue-priority");
    let db = Database::open(&path).expect("database should open");
    let service = RuminationService::new(db.conn());
    let report = sample_agent_search_report();

    let lpq = RuminationTriggerEvent::from_agent_search_report(
        RuminationTriggerKind::SessionBoundary,
        "session://priority",
        &report,
        "2026-04-16T10:00:00Z",
        "2026-04-16T10",
        None,
        Some("lpq-report".to_string()),
    )
    .expect("lpq event should normalize");
    let spq = RuminationTriggerEvent::from_agent_search_report(
        RuminationTriggerKind::ActionFailure,
        "task://priority",
        &report,
        "2026-04-16T10:01:00Z",
        "2026-04-16T10",
        None,
        Some("spq-report".to_string()),
    )
    .expect("spq event should normalize");

    service.schedule(lpq).expect("lpq event should enqueue");
    service.schedule(spq).expect("spq event should enqueue");

    let first_claim = service
        .claim_next_ready("2026-04-16T10:02:00Z")
        .expect("first claim should succeed")
        .expect("a queue item should be ready");
    assert_eq!(first_claim.queue_tier, RuminationQueueTier::Spq);
    service
        .complete(&first_claim, "2026-04-16T10:03:00Z")
        .expect("spq item should complete");

    let second_claim = service
        .claim_next_ready("2026-04-16T10:04:00Z")
        .expect("second claim should succeed")
        .expect("lpq item should remain");
    assert_eq!(second_claim.queue_tier, RuminationQueueTier::Lpq);
}

#[test]
fn claimed_items_can_be_retried_after_backoff() {
    let path = fresh_db_path("queue-retry");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());
    let service = RuminationService::new(db.conn());
    let report = sample_agent_search_report();

    let event = RuminationTriggerEvent::from_agent_search_report(
        RuminationTriggerKind::ActionFailure,
        "task://retry",
        &report,
        "2026-04-16T10:00:00Z",
        "2026-04-16T10",
        None,
        Some("retry-report".to_string()),
    )
    .expect("event should normalize");
    service.schedule(event).expect("event should enqueue");

    let claimed = service
        .claim_next_ready("2026-04-16T10:00:30Z")
        .expect("claim should succeed")
        .expect("spq item should be ready");
    service
        .retry(
            &claimed,
            "2026-04-16T10:05:00Z",
            "transient failure",
            "2026-04-16T10:01:00Z",
        )
        .expect("retry should persist");

    let before_backoff = service
        .claim_next_ready("2026-04-16T10:04:59Z")
        .expect("claim before backoff expiry should succeed");
    assert!(
        before_backoff.is_none(),
        "item should wait until retry backoff expires"
    );

    let after_backoff = service
        .claim_next_ready("2026-04-16T10:05:00Z")
        .expect("claim after backoff should succeed")
        .expect("item should become ready again");
    assert_eq!(after_backoff.queue_tier, RuminationQueueTier::Spq);

    let persisted = repo
        .list_rumination_queue_items("spq")
        .expect("spq items should load");
    assert_eq!(persisted[0].attempt_count, 1);
    assert_eq!(
        persisted[0].last_error.as_deref(),
        Some("transient failure")
    );
}
