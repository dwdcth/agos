use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    agent::orchestration::{AgentSearchReport, RetrievalStepReport},
    cognition::{
        action::{ActionBranch, ActionCandidate, ActionKind},
        metacog::GateDecision,
        report::{DecisionReport, GateReport, ScoredBranchReport},
        rumination::{RuminationService, RuminationTriggerEvent, RuminationTriggerKind},
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
    search::{Citation, ResultTrace, ScoreBreakdown, SearchFilters, SearchResult},
};

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-rumination-governance-tests")
        .join(format!("{name}-{unique}"))
        .join("rumination-governance.sqlite")
}

fn sample_record(id: &str, truth_layer: TruthLayer) -> MemoryRecord {
    MemoryRecord {
        id: id.to_string(),
        source: SourceRef {
            uri: format!("memo://rumination/{id}"),
            kind: SourceKind::Note,
            label: Some(format!("rumination-{id}")),
        },
        timestamp: RecordTimestamp {
            recorded_at: "2026-04-16T15:00:00Z".to_string(),
            created_at: "2026-04-16T15:00:00Z".to_string(),
            updated_at: "2026-04-16T15:00:00Z".to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Observation,
        truth_layer,
        provenance: Provenance {
            origin: "test".to_string(),
            imported_via: None,
            derived_from: vec!["seed".to_string()],
        },
        content_text: format!("record {id} should stay auditable"),
        chunk: Some(ChunkMetadata {
            chunk_index: 0,
            chunk_count: 1,
            anchor: ChunkAnchor::LineRange {
                start_line: 1,
                end_line: 2,
            },
            content_hash: format!("hash-{id}"),
        }),
        validity: ValidityWindow::default(),
    }
}

fn sample_result(record: MemoryRecord, query: &str, snippet: &str) -> SearchResult {
    SearchResult {
        citation: Citation::from_record(&record).expect("citation should build from chunked record"),
        record,
        snippet: snippet.to_string(),
        score: ScoreBreakdown {
            lexical_raw: -2.0,
            lexical_base: 0.30,
            keyword_bonus: 0.05,
            importance_bonus: 0.10,
            recency_bonus: 0.02,
            emotion_bonus: 0.0,
            final_score: 0.47,
        },
        trace: ResultTrace {
            matched_query: query.to_string(),
            query_strategies: Vec::new(),
            applied_filters: SearchFilters::default(),
        },
    }
}

fn sample_fragment(record_id: &str, truth_layer: TruthLayer) -> EvidenceFragment {
    let record = sample_record(record_id, truth_layer);
    let result = sample_result(record, "long-cycle rumination", "candidate-first durable evidence");

    EvidenceFragment {
        record_id: result.record.id,
        snippet: result.snippet,
        citation: result.citation,
        truth_context: TruthContext {
            truth_layer,
            t3_state: None,
            open_review_ids: Vec::new(),
            open_candidate_ids: Vec::new(),
        },
        trace: result.trace,
        score: result.score,
    }
}

fn sample_agent_search_report(primary_record_id: &str, truth_layer: TruthLayer) -> AgentSearchReport {
    let fragment = sample_fragment(primary_record_id, truth_layer);
    let branch = ActionBranch::new(
        ActionCandidate::new(ActionKind::Instrumental, "stabilize the next step")
            .with_intent("convert accumulated evidence into a slower candidate"),
    )
    .with_supporting_evidence(vec![fragment.clone()]);
    let working_memory = WorkingMemory {
        present: PresentFrame {
            world_fragments: vec![fragment.clone()],
            self_state: SelfStateSnapshot {
                task_context: Some("long-cycle learning".to_string()),
                capability_flags: vec!["rumination_ready".to_string()],
                readiness_flags: vec!["governance_ready".to_string()],
                facts: Vec::new(),
            },
            active_goal: None,
            active_risks: vec!["shared truth requires review".to_string()],
            metacog_flags: vec![MetacognitiveFlag {
                code: "candidate_only".to_string(),
                detail: Some("long-cycle outputs must remain pending".to_string()),
            }],
        },
        branches: vec![branch.clone()],
    };
    let selected_branch = ScoredBranchReport {
        branch,
        value: ValueVector {
            goal_progress: 0.82,
            information_gain: 0.66,
            risk_avoidance: 0.71,
            resource_efficiency: 0.58,
            agent_robustness: 0.79,
        },
        projected: ProjectedScore {
            final_score: 0.73,
            weight_snapshot: ValueConfig::default(),
        },
    };
    let citation = sample_result(
        sample_record(primary_record_id, truth_layer),
        "long-cycle rumination",
        "candidate-first durable evidence",
    )
    .citation;

    AgentSearchReport {
        working_memory,
        decision: DecisionReport {
            scored_branches: vec![selected_branch.clone()],
            selected_branch: Some(selected_branch),
            gate: GateReport {
                decision: GateDecision::Warning,
                diagnostics: vec!["slow-cycle output stays pending".to_string()],
                rejected_branch: None,
                regulative_branch: None,
                safe_response: None,
                autonomy_paused: false,
            },
            active_risks: vec!["shared truth requires review".to_string()],
            metacog_flags: vec![MetacognitiveFlag {
                code: "candidate_only".to_string(),
                detail: Some("long-cycle outputs must remain pending".to_string()),
            }],
        },
        retrieval_steps: vec![RetrievalStepReport {
            query: "long-cycle rumination".to_string(),
            applied_filters: SearchFilters::default(),
            result_count: 1,
            citations: vec![citation.clone()],
        }],
        citations: vec![citation],
        executed_steps: 1,
        step_limit: 2,
    }
}

#[test]
fn lpq_generates_unified_candidates_from_accumulated_evidence() {
    let path = fresh_db_path("lpq-unified-candidates");
    let db = Database::open(&path).expect("database should open");
    let repository = MemoryRepository::new(db.conn());
    let service = RuminationService::new(db.conn());
    let subject_ref = "task://rumination/long-cycle";

    repository
        .insert_record(&sample_record("t3-signal", TruthLayer::T3))
        .expect("t3 signal should insert");
    repository
        .insert_record(&sample_record("t2-pattern", TruthLayer::T2))
        .expect("t2 pattern should insert");

    let report = sample_agent_search_report("t3-signal", TruthLayer::T3);
    let event = RuminationTriggerEvent::from_agent_search_report(
        RuminationTriggerKind::EvidenceAccumulation,
        subject_ref,
        &report,
        "2026-04-16T16:00:00Z",
        "2026-04-16",
        None,
        Some("lpq-report-1".to_string()),
    )
    .expect("lpq event should normalize");

    service.schedule(event).expect("lpq event should enqueue");
    let _report = service
        .drain_long_cycle("2026-04-16T16:05:00Z")
        .expect("long-cycle drain should succeed")
        .expect("ready lpq work should drain");

    let candidates = repository
        .list_rumination_candidates()
        .expect("long-cycle candidates should load");
    assert_eq!(candidates.len(), 3, "lpq should emit one row per required long-cycle output");

    let kinds = candidates
        .iter()
        .map(|candidate| candidate.candidate_kind.as_str().to_string())
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            "promotion_candidate".to_string(),
            "skill_template".to_string(),
            "value_adjustment_candidate".to_string()
        ],
        "lpq should emit only the three required candidate kinds"
    );

    for candidate in candidates {
        assert_eq!(candidate.subject_ref, subject_ref);
        assert_eq!(candidate.source_queue_item_id.as_deref(), Some("lpq:evidence_accumulation:task://rumination/long-cycle:lpq-report-1:2026-04-16T16:00:00Z"));
        assert_eq!(candidate.status.as_str(), "pending");
        assert_eq!(
            candidate.evidence_refs,
            vec!["t3-signal".to_string()],
            "candidate should preserve LPQ evidence lineage"
        );
        assert!(
            candidate
                .payload
                .get("source_report")
                .is_some(),
            "candidate payload should keep the prior report instead of re-running retrieval: {:?}",
            candidate.payload
        );
    }
}
