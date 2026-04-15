use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    cognition::{
        action::{ActionBranch, ActionCandidate, ActionKind},
        assembly::{ActionSeed, SelfStateProvider, WorkingMemoryAssembler, WorkingMemoryRequest},
        working_memory::{
            ActiveGoal, MetacognitiveFlag, PresentFrame, SelfStateFact, SelfStateSnapshot,
            WorkingMemoryBuildError, WorkingMemoryBuilder,
        },
    },
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::{
        record::{RecordType, Scope, TruthLayer},
        repository::MemoryRepository,
        truth::TruthRecord,
    },
};

fn sample_present_frame() -> PresentFrame {
    PresentFrame {
        world_fragments: Vec::new(),
        self_state: SelfStateSnapshot {
            task_context: Some("stabilize working-memory contracts".to_string()),
            capability_flags: vec!["lexical_search_ready".to_string()],
            readiness_flags: vec!["truth_governance_ready".to_string()],
            facts: vec![SelfStateFact {
                key: "current_focus".to_string(),
                value: "phase_04_plan_01".to_string(),
                source_record_id: None,
            }],
        },
        active_goal: Some(ActiveGoal {
            summary: "assemble an immutable working-memory frame".to_string(),
        }),
        active_risks: vec!["terminology drift".to_string()],
        metacog_flags: vec![MetacognitiveFlag {
            code: "trace_required".to_string(),
            detail: Some("citations must remain attached".to_string()),
        }],
    }
}

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-working-memory-tests")
        .join(format!("{name}-{unique}"))
        .join("working-memory.sqlite")
}

struct TestSelfStateProvider;

impl SelfStateProvider for TestSelfStateProvider {
    fn snapshot(
        &self,
        request: &WorkingMemoryRequest,
        truths: &[TruthRecord],
    ) -> SelfStateSnapshot {
        let mut facts = request.selected_truth_facts(truths);
        facts.push(SelfStateFact {
            key: "provider".to_string(),
            value: "test-self-state-provider".to_string(),
            source_record_id: None,
        });

        SelfStateSnapshot {
            task_context: request.task_context.clone(),
            capability_flags: request.capability_flags.clone(),
            readiness_flags: request.readiness_flags.clone(),
            facts,
        }
    }
}

#[test]
fn working_memory_builder_requires_present_frame_and_uses_phase4_action_labels() {
    let branch = ActionBranch::new(ActionCandidate::new(
        ActionKind::Instrumental,
        "assemble the present frame",
    ));

    let err = WorkingMemoryBuilder::default()
        .push_branch(branch.clone())
        .build()
        .expect_err("builder should reject incomplete working-memory state");
    assert_eq!(err, WorkingMemoryBuildError::MissingPresentFrame);

    assert_eq!(ActionKind::Epistemic.as_str(), "epistemic");
    assert_eq!(ActionKind::Instrumental.as_str(), "instrumental");
    assert_eq!(ActionKind::Regulative.as_str(), "regulative");
    assert_eq!(ActionKind::parse("operational"), None);
    assert_eq!(ActionKind::parse("regulatory"), None);

    let working_memory = WorkingMemoryBuilder::default()
        .present(sample_present_frame())
        .push_branch(branch)
        .build()
        .expect("builder should produce immutable working memory once present exists");

    assert_eq!(
        working_memory.present.active_goal.unwrap().summary,
        "assemble an immutable working-memory frame"
    );
    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(
        working_memory.branches[0].candidate.kind,
        ActionKind::Instrumental
    );
}

#[test]
fn assembler_preserves_citations_truth_context_and_in_memory_runtime_only() {
    let path = fresh_db_path("assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let decision = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/decision".to_string(),
            source_label: Some("decision".to_string()),
            source_kind: None,
            content: "working memory must preserve citations and traceable truth context"
                .to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:00:00Z".to_string(),
            valid_from: Some("2026-04-16T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("decision ingest should succeed");
    let risk = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/risk".to_string(),
            source_label: Some("risk".to_string()),
            source_kind: None,
            content: "under-supported action branches should trigger a cited risk reminder"
                .to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T3,
            recorded_at: "2026-04-16T09:30:00Z".to_string(),
            valid_from: Some("2026-04-16T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("risk ingest should succeed");

    let decision_id = decision.record_ids[0].clone();
    let risk_id = risk.record_ids[0].clone();

    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let request = WorkingMemoryRequest::new("working memory citations")
        .with_limit(5)
        .with_task_context("assemble a cited working-memory frame")
        .with_active_goal("decide how to proceed from retrieved evidence")
        .with_capability_flag("lexical_search_ready")
        .with_readiness_flag("truth_governance_ready")
        .with_active_risk("under-supported branch")
        .with_metacog_flag(MetacognitiveFlag {
            code: "citation_required".to_string(),
            detail: Some("working memory must stay explainable".to_string()),
        })
        .with_action_seed(
            ActionSeed::new(ActionCandidate::new(
                ActionKind::Epistemic,
                "inspect the retrieved evidence before acting",
            ))
            .with_supporting_record_ids(vec![decision_id.clone(), risk_id.clone()]),
        );

    let first = assembler
        .assemble(&request)
        .expect("assembly should succeed over retrieval and truth seams");
    let second = assembler
        .assemble(&request)
        .expect("rebuilding should create a fresh working-memory frame");

    assert_eq!(db.schema_version().expect("schema version"), 4);
    assert_eq!(
        MemoryRepository::new(db.conn())
            .count_records()
            .expect("record count should load"),
        2,
        "assembly should not persist runtime working-memory rows"
    );
    assert_eq!(first.present.world_fragments.len(), 2);
    assert_eq!(
        first.present.self_state.task_context.as_deref(),
        Some("assemble a cited working-memory frame")
    );
    assert_eq!(
        first.present.self_state.capability_flags,
        vec!["lexical_search_ready".to_string()]
    );
    assert!(
        first
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.source_record_id.as_deref() == Some(decision_id.as_str())),
        "self-state facts should come through the provider seam from selected truth records"
    );
    assert!(
        first
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.value == "test-self-state-provider"),
        "provider-specific facts should survive assembly"
    );

    let decision_fragment = first
        .present
        .world_fragments
        .iter()
        .find(|fragment| fragment.record_id == decision_id)
        .expect("decision fragment should exist");
    assert_eq!(decision_fragment.citation.record_id, decision_id);
    assert_eq!(decision_fragment.truth_context.truth_layer, TruthLayer::T2);
    assert_eq!(
        decision_fragment.citation.validity.valid_from.as_deref(),
        Some("2026-04-16T00:00:00Z")
    );
    assert_eq!(decision_fragment.trace.matched_query, "working memory citations");

    let risk_fragment = first
        .present
        .world_fragments
        .iter()
        .find(|fragment| fragment.record_id == risk_id)
        .expect("risk fragment should exist");
    assert_eq!(risk_fragment.truth_context.truth_layer, TruthLayer::T3);
    assert!(
        risk_fragment.truth_context.t3_state.is_some(),
        "t3 fragments should carry revocable truth context into the present frame"
    );

    assert_eq!(first.branches.len(), 1);
    assert_eq!(first.branches[0].candidate.kind, ActionKind::Epistemic);
    assert_eq!(first.branches[0].supporting_evidence.len(), 2);
    assert_eq!(
        first.branches[0].supporting_evidence[0].citation.record_id,
        first.branches[0].supporting_evidence[0].record_id
    );
    assert!(
        !std::ptr::eq(&first.present, &second.present),
        "rebuilds should return fresh working-memory values instead of mutating stored state"
    );
}
