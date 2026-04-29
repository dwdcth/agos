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
        skill_memory::SkillMemoryTemplate,
        value::{ProjectedScore, ValueConfig, ValueVector},
        working_memory::{
            EvidenceFragment, MetacognitiveFlag, PresentFrame, SelfStateSnapshot, TruthContext,
            WorkingMemory,
        },
    },
    core::db::Database,
    memory::{
        governance::TruthGovernanceService,
        record::{
            ChunkAnchor, ChunkMetadata, MemoryRecord, Provenance, RecordTimestamp, RecordType,
            Scope, SourceKind, SourceRef, TruthLayer, ValidityWindow,
        },
        repository::{MemoryRepository, RuminationCandidateKind},
        truth::{OntologyCandidateState, PromotionDecisionState},
    },
    search::{
        ChannelContribution, Citation, ResultTrace, ScoreBreakdown, SearchFilters, SearchResult,
    },
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
        citation: Citation::from_record(&record)
            .expect("citation should build from chunked record"),
        record,
        snippet: snippet.to_string(),
        dsl: None,
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
            channel_contribution: ChannelContribution::LexicalOnly,
            applied_filters: SearchFilters::default(),
        },
    }
}

fn sample_fragment(record_id: &str, truth_layer: TruthLayer) -> EvidenceFragment {
    let record = sample_record(record_id, truth_layer);
    let result = sample_result(
        record,
        "long-cycle rumination",
        "candidate-first durable evidence",
    );

    EvidenceFragment {
        record_id: result.record.id,
        snippet: result.snippet,
        citation: result.citation,
        provenance: result.record.provenance,
        truth_context: TruthContext {
            truth_layer,
            t3_state: None,
            open_review_ids: Vec::new(),
            open_candidate_ids: Vec::new(),
        },
        dsl: None,
        trace: result.trace,
        score: result.score,
    }
}

fn sample_agent_search_report(
    primary_record_id: &str,
    truth_layer: TruthLayer,
) -> AgentSearchReport {
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
    assert_eq!(
        candidates.len(),
        3,
        "lpq should emit one row per required long-cycle output"
    );

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
        assert_eq!(
            candidate.source_queue_item_id.as_deref(),
            Some(
                "lpq:evidence_accumulation:task://rumination/long-cycle:lpq-report-1:2026-04-16T16:00:00Z"
            )
        );
        assert_eq!(candidate.status.as_str(), "pending");
        assert_eq!(
            candidate.evidence_refs,
            vec!["t3-signal".to_string()],
            "candidate should preserve LPQ evidence lineage"
        );
        assert!(
            candidate.payload.get("source_report").is_some(),
            "candidate payload should keep the prior report instead of re-running retrieval: {:?}",
            candidate.payload
        );
    }

    let skill_candidates = repository
        .list_skill_template_candidates_for_subject(subject_ref)
        .expect("skill template candidates should load through the typed helper");
    assert_eq!(skill_candidates.len(), 1);
    let skill_candidate = &skill_candidates[0];
    assert_eq!(skill_candidate.payload.payload_version, 1);
    assert_eq!(
        skill_candidate.payload.action.kind, "instrumental",
        "LPQ skill payload should preserve the selected branch action kind"
    );
    assert_eq!(
        skill_candidate.payload.action.summary,
        "stabilize the next step"
    );
    assert_eq!(
        skill_candidate.payload.template_summary,
        "stabilize the next step"
    );
    assert_eq!(
        skill_candidate.payload.boundaries.supporting_record_ids,
        vec!["t3-signal".to_string()]
    );
    assert_eq!(skill_candidate.payload.evidence_count, 1);
    assert!(
        skill_candidate
            .payload
            .source_report
            .get("decision")
            .is_some(),
        "typed skill payload should preserve the original source report lineage"
    );

    let reconstructed = SkillMemoryTemplate::from_rumination_candidate(skill_candidate)
        .expect("typed skill candidate payload should reconstruct a skill memory template");
    assert_eq!(reconstructed.action.summary, "stabilize the next step");
    assert_eq!(
        reconstructed.boundaries.supporting_record_ids,
        vec!["t3-signal".to_string()]
    );
}

#[test]
fn lpq_bridges_to_governance_without_auto_approval() {
    let path = fresh_db_path("lpq-governance-bridge");
    let db = Database::open(&path).expect("database should open");
    let repository = MemoryRepository::new(db.conn());
    let governance = TruthGovernanceService::new(db.conn());
    let service = RuminationService::new(db.conn());
    let subject_ref = "task://rumination/governance";

    repository
        .insert_record(&sample_record("t3-hypothesis", TruthLayer::T3))
        .expect("t3 hypothesis should insert");
    repository
        .insert_record(&sample_record("t2-pattern", TruthLayer::T2))
        .expect("t2 pattern should insert");

    let t3_report = sample_agent_search_report("t3-hypothesis", TruthLayer::T3);
    let t3_event = RuminationTriggerEvent::from_agent_search_report(
        RuminationTriggerKind::EvidenceAccumulation,
        subject_ref,
        &t3_report,
        "2026-04-16T17:00:00Z",
        "2026-04-16",
        None,
        Some("lpq-t3".to_string()),
    )
    .expect("t3 lpq event should normalize");
    service
        .schedule(t3_event)
        .expect("t3 lpq event should enqueue");

    let t2_report = sample_agent_search_report("t2-pattern", TruthLayer::T2);
    let t2_event = RuminationTriggerEvent::from_agent_search_report(
        RuminationTriggerKind::IdleWindow,
        subject_ref,
        &t2_report,
        "2026-04-16T17:10:00Z",
        "2026-04-16",
        None,
        Some("lpq-t2".to_string()),
    )
    .expect("t2 lpq event should normalize");
    service
        .schedule(t2_event)
        .expect("t2 lpq event should enqueue");

    service
        .drain_long_cycle("2026-04-16T17:20:00Z")
        .expect("t3 long-cycle drain should succeed")
        .expect("t3 long-cycle work should drain");
    service
        .drain_long_cycle("2026-04-16T17:21:00Z")
        .expect("t2 long-cycle drain should succeed")
        .expect("t2 long-cycle work should drain");

    let candidates = repository
        .list_rumination_candidates()
        .expect("rumination candidates should load");
    let promotion_candidates = candidates
        .iter()
        .filter(|candidate| candidate.candidate_kind == RuminationCandidateKind::PromotionCandidate)
        .collect::<Vec<_>>();
    assert_eq!(promotion_candidates.len(), 2);
    assert!(
        promotion_candidates
            .iter()
            .all(|candidate| candidate.governance_ref_id.is_some()),
        "promotion candidates should persist canonical governance refs: {promotion_candidates:?}"
    );

    let pending_reviews = governance
        .list_pending_reviews()
        .expect("pending promotion reviews should load");
    assert_eq!(pending_reviews.len(), 1);
    assert_eq!(pending_reviews[0].source_record_id, "t3-hypothesis");
    assert_eq!(
        pending_reviews[0].decision_state,
        PromotionDecisionState::Pending
    );

    let pending_candidates = governance
        .list_pending_candidates()
        .expect("pending ontology candidates should load");
    assert_eq!(pending_candidates.len(), 1);
    assert_eq!(pending_candidates[0].source_record_id, "t2-pattern");
    assert_eq!(
        pending_candidates[0].candidate_state,
        OntologyCandidateState::Pending
    );
}
