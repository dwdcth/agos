use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    cognition::{
        action::{ActionBranch, ActionCandidate, ActionKind},
        assembly::{
            ActionSeed, SelfStateProvider, WorkingMemoryAssembler, WorkingMemoryRequest,
        },
        working_memory::{
            ActiveGoal, MetacognitiveFlag, PresentFrame, SelfStateFact, SelfStateSnapshot,
            WorkingMemoryBuildError, WorkingMemoryBuilder,
        },
    },
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::{
        record::{
            ChunkAnchor, ChunkMetadata, MemoryRecord, Provenance, RecordTimestamp, RecordType,
            Scope, SourceKind, SourceRef, TruthLayer, ValidityWindow,
        },
        repository::MemoryRepository,
        truth::TruthRecord,
    },
    search::{
        ChannelContribution, Citation, ResultTrace, ScoreBreakdown, SearchRequest, SearchResult,
        SearchService,
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

fn synthetic_result(record_id: &str, source_uri: &str, query: &str, snippet: &str) -> SearchResult {
    let record = MemoryRecord {
        id: record_id.to_string(),
        source: SourceRef {
            uri: source_uri.to_string(),
            kind: SourceKind::Note,
            label: Some(source_uri.rsplit('/').next().unwrap_or("record").to_string()),
        },
        timestamp: RecordTimestamp {
            recorded_at: "2026-04-16T00:00:00Z".to_string(),
            created_at: "2026-04-16T00:00:00Z".to_string(),
            updated_at: "2026-04-16T00:00:00Z".to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Observation,
        truth_layer: TruthLayer::T2,
        provenance: Provenance {
            origin: "test".to_string(),
            imported_via: None,
            derived_from: vec!["synthetic-follow-up".to_string()],
        },
        content_text: snippet.to_string(),
        chunk: Some(ChunkMetadata {
            chunk_index: 0,
            chunk_count: 1,
            anchor: ChunkAnchor::LineRange {
                start_line: 1,
                end_line: 1,
            },
            content_hash: format!("hash-{record_id}"),
        }),
        validity: ValidityWindow::default(),
    };

    SearchResult {
        citation: Citation::from_record(&record).expect("synthetic chunk metadata should exist"),
        record,
        snippet: snippet.to_string(),
        score: ScoreBreakdown {
            lexical_raw: -1.0,
            lexical_base: 0.4,
            keyword_bonus: 0.05,
            importance_bonus: 0.0,
            recency_bonus: 0.0,
            emotion_bonus: 0.0,
            final_score: 0.45,
        },
        trace: ResultTrace {
            matched_query: query.to_string(),
            query_strategies: Vec::new(),
            channel_contribution: ChannelContribution::LexicalOnly,
            applied_filters: Default::default(),
        },
    }
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
            content:
                "working memory risk reminder keeps citations attached to under-supported branches"
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
    let request = WorkingMemoryRequest::new("working memory")
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

    assert_eq!(db.schema_version().expect("schema version"), 5);
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
    assert_eq!(decision_fragment.trace.matched_query, "working memory");

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

#[test]
fn assembler_integrates_follow_up_evidence_into_world_fragments() {
    let path = fresh_db_path("follow-up-world-fragments");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary".to_string(),
            source_label: Some("primary".to_string()),
            source_kind: None,
            content: "primary query result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up".to_string(),
            source_label: Some("follow-up".to_string()),
            source_kind: None,
            content: "follow-up only result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:05:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("follow-up ingest should succeed");

    let primary_id = primary.record_ids[0].clone();
    let follow_up_id = follow_up.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary query").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let follow_up_result = search
        .search(&SearchRequest::new("follow-up only").with_limit(1))
        .expect("follow-up search should succeed")
        .results;
    let mut integrated_results = primary_result;
    integrated_results.extend(follow_up_result);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary query")
                .with_limit(1)
                .with_active_goal("integrate follow-up evidence")
                .with_integrated_results(integrated_results),
        )
        .expect("assembly should succeed");

    let record_ids = working_memory
        .present
        .world_fragments
        .iter()
        .map(|fragment| fragment.record_id.as_str())
        .collect::<Vec<_>>();

    assert!(
        record_ids.contains(&primary_id.as_str()),
        "assembled world fragments should include the primary result: {record_ids:?}"
    );
    assert!(
        record_ids.contains(&follow_up_id.as_str()),
        "assembled world fragments should include follow-up-only evidence once integrated: {record_ids:?}"
    );
}

#[test]
fn assembler_uses_integrated_follow_up_fragments_for_branch_support() {
    let path = fresh_db_path("follow-up-branch-support");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary-branch".to_string(),
            source_label: Some("primary-branch".to_string()),
            source_kind: None,
            content: "primary branch result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up-branch".to_string(),
            source_label: Some("follow-up-branch".to_string()),
            source_kind: None,
            content: "follow-up branch support".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:05:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("follow-up ingest should succeed");

    let follow_up_id = follow_up.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary branch query").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let mut integrated_results = primary_result;
    integrated_results.push(synthetic_result(
        &follow_up_id,
        "memo://project/follow-up-branch",
        "follow-up branch support",
        "follow-up branch support",
    ));

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary branch query")
                .with_limit(1)
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "inspect follow-up evidence",
                    ))
                    .with_supporting_record_ids(vec![follow_up_id.clone()]),
                ),
        )
        .expect("assembly should succeed");

    let support_ids = working_memory.branches[0]
        .supporting_evidence
        .iter()
        .map(|fragment| fragment.record_id.as_str())
        .collect::<Vec<_>>();

    assert!(
        support_ids.contains(&follow_up_id.as_str()),
        "branch supporting evidence should include the follow-up-only fragment once integrated: {support_ids:?}"
    );
}
