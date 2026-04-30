use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    cognition::{
        action::{ActionBranch, ActionCandidate, ActionKind, ActionSource},
        assembly::{
            ActionSeed, AdaptiveSelfStateProvider, MinimalSelfStateProvider, SelfStateProvider,
            WorkingMemoryAssembler, WorkingMemoryRequest,
        },
        self_model::{
            ProjectedSelfModel, RuntimeSelfState, SelfModelReadModel, StableSelfKnowledge,
            compact_self_model_subject,
        },
        skill_memory::{
            ActionTemplate, Boundaries, ExpectedOutcome, Preconditions, SkillMemoryTemplate,
        },
        working_memory::{
            ActiveGoal, MetacognitiveFlag, PresentFrame, SelfStateFact, SelfStateSnapshot,
            WorkingMemoryBuildError, WorkingMemoryBuilder,
        },
        world_model::{
            CURRENT_WORLD_KEY, CurrentWorldSlice, ProjectedWorldModel, WorldFragmentProjection,
        },
    },
    core::config::{
        Config, EmbeddingBackend, RetrievalConfig, RetrievalMode, RootVectorConfig, VectorBackend,
    },
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::{
        governance::{
            CreateOntologyCandidateRequest, CreatePromotionReviewRequest, TruthGovernanceService,
        },
        record::{
            ChunkAnchor, ChunkMetadata, MemoryRecord, Provenance, RecordTimestamp, RecordType,
            Scope, SourceKind, SourceRef, TruthLayer, ValidityWindow,
        },
        repository::{
            LocalAdaptationEntry, LocalAdaptationPayload, LocalAdaptationTargetKind,
            MemoryRepository, PersistedSelfModelSnapshot, PersistedSelfModelSnapshotEntry,
            RuminationCandidate, RuminationCandidateKind, RuminationCandidateStatus,
            SelfModelGovernanceMetadata, SelfModelResolutionState,
        },
        truth::{T3Confidence, T3RevocationState, TruthRecord},
    },
    search::{
        ChannelContribution, Citation, ResultTrace, ScoreBreakdown, SearchRequest, SearchResult,
        SearchService,
    },
};
use serde_json::json;

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
            label: Some(
                source_uri
                    .rsplit('/')
                    .next()
                    .unwrap_or("record")
                    .to_string(),
            ),
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
        dsl: None,
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
    fn project(
        &self,
        request: &WorkingMemoryRequest,
        truths: &[TruthRecord],
    ) -> ProjectedSelfModel {
        ProjectedSelfModel::new(
            StableSelfKnowledge {
                capability_flags: request.capability_flags.clone(),
                facts: request.selected_truth_facts(truths),
            },
            RuntimeSelfState {
                task_context: request.task_context.clone(),
                readiness_flags: request.readiness_flags.clone(),
                facts: vec![SelfStateFact {
                    key: "provider".to_string(),
                    value: "test-self-state-provider".to_string(),
                    source_record_id: None,
                }],
            },
        )
    }
}

fn persist_current_world_model_snapshot(
    conn: &rusqlite::Connection,
    subject_ref: &str,
    result: SearchResult,
    snapshot_id: &str,
) -> ProjectedWorldModel {
    let repository = MemoryRepository::new(conn);
    let record_id = result.record.id.clone();
    let truth = repository
        .get_truth_record(&record_id)
        .expect("truth lookup should succeed")
        .expect("truth should exist for the snapshot-backed fragment");
    let repository_dsl = result.dsl.is_none().then(|| {
        repository
            .list_layered_records_for_ids(std::slice::from_ref(&record_id))
            .expect("layered records should load")
            .into_iter()
            .find(|record| record.record.id == record_id)
            .and_then(|record| record.dsl.map(|dsl| dsl.payload))
    });
    let world_model = ProjectedWorldModel::new(CurrentWorldSlice::new(vec![
        WorldFragmentProjection::from_search_result(
            result,
            &truth,
            repository_dsl.as_ref().and_then(Option::as_ref),
        ),
    ]));

    repository
        .replace_world_model_snapshot(&world_model.to_snapshot(
            subject_ref,
            CURRENT_WORLD_KEY,
            snapshot_id,
            "2026-04-20T12:00:00Z",
            "2026-04-20T12:00:00Z",
        ))
        .expect("world-model snapshot should persist");

    world_model
}

fn ready_embedding_config(mode: RetrievalMode) -> Config {
    Config {
        retrieval: RetrievalConfig { mode },
        embedding: agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            ..Default::default()
        },
        ..Default::default()
    }
}

fn persisted_skill_template_candidate(
    candidate_id: &str,
    subject_ref: &str,
    status: RuminationCandidateStatus,
    created_at: &str,
    template: &SkillMemoryTemplate,
) -> RuminationCandidate {
    RuminationCandidate {
        candidate_id: candidate_id.to_string(),
        source_queue_item_id: Some(format!("lpq:{candidate_id}")),
        candidate_kind: RuminationCandidateKind::SkillTemplate,
        subject_ref: subject_ref.to_string(),
        payload: serde_json::to_value(template.to_candidate_payload(
            "evidence_accumulation",
            json!({
                "decision": {
                    "selected_branch": template.action.summary.clone(),
                }
            }),
            template.boundaries.supporting_record_ids.len(),
        ))
        .expect("skill template payload should serialize"),
        evidence_refs: template.boundaries.supporting_record_ids.clone(),
        governance_ref_id: None,
        status,
        created_at: created_at.to_string(),
        updated_at: created_at.to_string(),
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
            .with_supporting_record_ids(vec![decision_id.clone(), risk_id.clone()])
            .with_risk_marker("clarification_required"),
        );

    let first = assembler
        .assemble(&request)
        .expect("assembly should succeed over retrieval and truth seams");
    let second = assembler
        .assemble(&request)
        .expect("rebuilding should create a fresh working-memory frame");

    assert_eq!(db.schema_version().expect("schema version"), 10);
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
    let decision_dsl = decision_fragment
        .dsl
        .as_ref()
        .expect("layered dsl should be attached to retrieved decision fragment");
    assert_eq!(decision_dsl.domain, "project");
    assert_eq!(decision_dsl.kind, "decision");
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
    assert!(
        risk_fragment.dsl.is_some(),
        "layered dsl should also be attached to retrieved risk fragment"
    );

    assert_eq!(first.branches.len(), 1);
    assert_eq!(first.branches[0].candidate.kind, ActionKind::Epistemic);
    assert_eq!(first.branches[0].supporting_evidence.len(), 2);
    assert!(
        first.branches[0]
            .risk_markers
            .contains(&"clarification_required".to_string()),
        "branch risk markers should be preserved from action seeds"
    );
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
fn assembler_preserves_present_control_state_from_request() {
    let path = fresh_db_path("present-control-state");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/present-control-state".to_string(),
            source_label: Some("present-control-state".to_string()),
            source_kind: None,
            content: "control state should survive working-memory assembly".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:12:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("control-state ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("control state")
                .with_active_goal("select the safest cited next step")
                .with_active_risk("clarification required")
                .with_active_risk("under-supported evidence")
                .with_metacog_flag(MetacognitiveFlag {
                    code: "trace_required".to_string(),
                    detail: Some("preserve citations for decision gating".to_string()),
                }),
        )
        .expect("assembly should preserve present control state");

    assert_eq!(
        working_memory
            .present
            .active_goal
            .as_ref()
            .map(|goal| goal.summary.as_str()),
        Some("select the safest cited next step")
    );
    assert_eq!(
        working_memory.present.active_risks,
        vec![
            "clarification required".to_string(),
            "under-supported evidence".to_string()
        ]
    );
    assert_eq!(working_memory.present.metacog_flags.len(), 1);
    assert_eq!(
        working_memory.present.metacog_flags[0].code,
        "trace_required"
    );
    assert_eq!(
        working_memory.present.metacog_flags[0].detail.as_deref(),
        Some("preserve citations for decision gating")
    );
}

#[test]
fn assembler_preserves_self_state_readiness_flags_from_request() {
    let path = fresh_db_path("self-state-readiness-flags");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/self-state-readiness-flags".to_string(),
            source_label: Some("self-state-readiness-flags".to_string()),
            source_kind: None,
            content: "self state readiness flags should survive assembly".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:13:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("self-state-readiness ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("readiness flags")
                .with_readiness_flag("truth_governance_ready")
                .with_readiness_flag("tooling_ready"),
        )
        .expect("assembly should preserve readiness flags");

    assert_eq!(
        working_memory.present.self_state.readiness_flags,
        vec![
            "truth_governance_ready".to_string(),
            "tooling_ready".to_string()
        ]
    );
}

#[test]
fn assembler_preserves_self_state_task_context_from_request() {
    let path = fresh_db_path("self-state-task-context");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/self-state-task-context".to_string(),
            source_label: Some("self-state-task-context".to_string()),
            source_kind: None,
            content: "self state task context should survive assembly".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:13:15Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("self-state-task-context ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("task context")
                .with_task_context("stabilize the next cited decision"),
        )
        .expect("assembly should preserve task context");

    assert_eq!(
        working_memory.present.self_state.task_context.as_deref(),
        Some("stabilize the next cited decision")
    );
}

#[test]
fn assembler_preserves_self_state_capability_flags_from_request() {
    let path = fresh_db_path("self-state-capability-flags");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/self-state-capability-flags".to_string(),
            source_label: Some("self-state-capability-flags".to_string()),
            source_kind: None,
            content: "self state capability flags should survive assembly".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:13:30Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("self-state-capability ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("capability flags")
                .with_capability_flag("lexical_search_ready")
                .with_capability_flag("local_sqlite_ready"),
        )
        .expect("assembly should preserve capability flags");

    assert_eq!(
        working_memory.present.self_state.capability_flags,
        vec![
            "lexical_search_ready".to_string(),
            "local_sqlite_ready".to_string()
        ]
    );
}

#[test]
fn assembler_injects_subject_local_adaptation_entries_into_self_state() {
    let path = fresh_db_path("subject-local-adaptation-self-state");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let repository = MemoryRepository::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/subject-local-adaptation".to_string(),
            source_label: Some("subject-local-adaptation".to_string()),
            source_kind: None,
            content: "subject-specific local adaptations should survive assembly".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("subject adaptation ingest should succeed");

    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode".to_string(),
            subject_ref: "subject://agent/demo".to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("conservative"),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec!["memo://project/subject-local-adaptation".to_string()],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:30Z".to_string(),
            updated_at: "2026-04-16T10:14:30Z".to_string(),
        })
        .expect("local adaptation entry should insert");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("subject-specific adaptation")
                .with_subject_ref("subject://agent/demo"),
        )
        .expect("assembly should inject subject local adaptation facts");

    assert!(
        working_memory
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.key == "self_state:preferred_mode" && fact.value == "conservative"),
        "adaptive self-state provider should inject subject-scoped local adaptation facts"
    );
}

#[test]
fn assembler_preserves_local_adaptation_fact_source_as_none() {
    let path = fresh_db_path("local-adaptation-source-none");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let repository = MemoryRepository::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/local-adaptation-source-none".to_string(),
            source_label: Some("local-adaptation-source-none".to_string()),
            source_kind: None,
            content: "local adaptation facts should remain source-local".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:01Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("local adaptation source-none ingest should succeed");

    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode".to_string(),
            subject_ref: "subject://agent/demo".to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("conservative"),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec!["memo://project/local-adaptation-source-none".to_string()],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:30Z".to_string(),
            updated_at: "2026-04-16T10:14:30Z".to_string(),
        })
        .expect("local adaptation entry should insert");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("subject-specific adaptation")
                .with_subject_ref("subject://agent/demo"),
        )
        .expect("assembly should inject local adaptation facts");

    let fact = working_memory
        .present
        .self_state
        .facts
        .iter()
        .find(|fact| fact.key == "self_state:preferred_mode")
        .expect("local adaptation fact should exist");
    assert_eq!(fact.source_record_id, None);
}

#[test]
fn assembler_injects_request_local_adaptation_entries_into_self_state() {
    let path = fresh_db_path("request-local-adaptation-self-state");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/request-local-adaptation".to_string(),
            source_label: Some("request-local-adaptation".to_string()),
            source_kind: None,
            content: "request-scoped local adaptations should survive assembly".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:05Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("request adaptation ingest should succeed");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("request-specific adaptation")
                .with_local_adaptation_entries(vec![
                    LocalAdaptationEntry {
                        entry_id: "request-local-adaptation:self-state:preferred_mode".to_string(),
                        subject_ref: "subject://agent/demo".to_string(),
                        target_kind: LocalAdaptationTargetKind::SelfState,
                        key: "preferred_mode".to_string(),
                        payload: LocalAdaptationPayload {
                            value: json!("conservative"),
                            trigger_kind: "request_override".to_string(),
                            evidence_refs: vec![
                                "memo://project/request-local-adaptation".to_string(),
                            ],
                        },
                        source_queue_item_id: None,
                        created_at: "2026-04-16T10:14:31Z".to_string(),
                        updated_at: "2026-04-16T10:14:31Z".to_string(),
                    },
                    LocalAdaptationEntry {
                        entry_id: "request-local-adaptation:risk-boundary:deploy".to_string(),
                        subject_ref: "subject://agent/demo".to_string(),
                        target_kind: LocalAdaptationTargetKind::RiskBoundary,
                        key: "deploy".to_string(),
                        payload: LocalAdaptationPayload {
                            value: json!("requires_human_review"),
                            trigger_kind: "request_override".to_string(),
                            evidence_refs: vec![
                                "memo://project/request-local-adaptation".to_string(),
                            ],
                        },
                        source_queue_item_id: None,
                        created_at: "2026-04-16T10:14:32Z".to_string(),
                        updated_at: "2026-04-16T10:14:32Z".to_string(),
                    },
                    LocalAdaptationEntry {
                        entry_id: "request-local-adaptation:private-t3:hypothesis".to_string(),
                        subject_ref: "subject://agent/demo".to_string(),
                        target_kind: LocalAdaptationTargetKind::PrivateT3,
                        key: "hypothesis".to_string(),
                        payload: LocalAdaptationPayload {
                            value: json!("prefer_regulative_first"),
                            trigger_kind: "request_override".to_string(),
                            evidence_refs: vec![
                                "memo://project/request-local-adaptation".to_string(),
                            ],
                        },
                        source_queue_item_id: None,
                        created_at: "2026-04-16T10:14:33Z".to_string(),
                        updated_at: "2026-04-16T10:14:33Z".to_string(),
                    },
                ]),
        )
        .expect("assembly should inject request local adaptation facts");

    assert!(
        working_memory
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.key == "self_state:preferred_mode" && fact.value == "conservative"),
        "adaptive self-state provider should inject request-scoped self-state facts"
    );
    assert!(
        working_memory
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.key == "risk_boundary:deploy" && fact.value == "requires_human_review"),
        "adaptive self-state provider should inject request-scoped risk-boundary facts"
    );
    assert!(
        working_memory
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.key == "private_t3:hypothesis"
                && fact.value == "prefer_regulative_first"),
        "adaptive self-state provider should inject request-scoped private-t3 facts"
    );
}

#[test]
fn assembler_preserves_request_local_adaptation_fact_source_as_none() {
    let path = fresh_db_path("request-local-adaptation-source-none");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/request-local-adaptation-source-none".to_string(),
            source_label: Some("request-local-adaptation-source-none".to_string()),
            source_kind: None,
            content: "request local adaptation facts should remain source-local".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:06Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("request local adaptation source-none ingest should succeed");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("request-specific adaptation source none")
                .with_local_adaptation_entries(vec![LocalAdaptationEntry {
                    entry_id: "request-local-adaptation:self-state:preferred_mode".to_string(),
                    subject_ref: "subject://agent/demo".to_string(),
                    target_kind: LocalAdaptationTargetKind::SelfState,
                    key: "preferred_mode".to_string(),
                    payload: LocalAdaptationPayload {
                        value: json!("conservative"),
                        trigger_kind: "request_override".to_string(),
                        evidence_refs: vec![
                            "memo://project/request-local-adaptation-source-none".to_string(),
                        ],
                    },
                    source_queue_item_id: None,
                    created_at: "2026-04-16T10:14:31Z".to_string(),
                    updated_at: "2026-04-16T10:14:31Z".to_string(),
                }]),
        )
        .expect("assembly should inject request local adaptation facts");

    let fact = working_memory
        .present
        .self_state
        .facts
        .iter()
        .find(|fact| fact.key == "self_state:preferred_mode")
        .expect("request local adaptation fact should exist");
    assert_eq!(fact.source_record_id, None);
}

#[test]
fn assembler_displays_non_string_request_local_adaptation_payloads_in_self_state() {
    let path = fresh_db_path("request-local-adaptation-non-string");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/request-local-adaptation-non-string".to_string(),
            source_label: Some("request-local-adaptation-non-string".to_string()),
            source_kind: None,
            content: "request non-string local adaptation payloads should display predictably"
                .to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:07Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("request adaptation non-string ingest should succeed");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("request-specific non-string adaptation")
                .with_local_adaptation_entries(vec![LocalAdaptationEntry {
                    entry_id: "request-local-adaptation:risk-boundary:bool".to_string(),
                    subject_ref: "subject://agent/demo".to_string(),
                    target_kind: LocalAdaptationTargetKind::RiskBoundary,
                    key: "deployment_allowed".to_string(),
                    payload: LocalAdaptationPayload {
                        value: json!(false),
                        trigger_kind: "request_override".to_string(),
                        evidence_refs: vec![
                            "memo://project/request-local-adaptation-non-string".to_string(),
                        ],
                    },
                    source_queue_item_id: None,
                    created_at: "2026-04-16T10:14:42Z".to_string(),
                    updated_at: "2026-04-16T10:14:42Z".to_string(),
                }]),
        )
        .expect("assembly should inject non-string request local adaptation facts");

    assert!(
        working_memory
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.key == "risk_boundary:deployment_allowed" && fact.value == "false"),
        "adaptive self-state provider should stringify non-string request-local adaptation payloads"
    );
}

#[test]
fn assembler_overwrites_request_local_adaptations_for_same_key() {
    let path = fresh_db_path("request-local-adaptation-order");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/request-local-adaptation-order".to_string(),
            source_label: Some("request-local-adaptation-order".to_string()),
            source_kind: None,
            content: "request local adaptation ordering should preserve caller order".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:06Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("request adaptation order ingest should succeed");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("request-specific adaptation order")
                .with_local_adaptation_entries(vec![
                    LocalAdaptationEntry {
                        entry_id: "request-local-adaptation:self-state:preferred_mode:first"
                            .to_string(),
                        subject_ref: "subject://agent/demo".to_string(),
                        target_kind: LocalAdaptationTargetKind::SelfState,
                        key: "preferred_mode".to_string(),
                        payload: LocalAdaptationPayload {
                            value: json!("aggressive"),
                            trigger_kind: "request_override".to_string(),
                            evidence_refs: vec![
                                "memo://project/request-local-adaptation-order".to_string(),
                            ],
                        },
                        source_queue_item_id: None,
                        created_at: "2026-04-16T10:14:31Z".to_string(),
                        updated_at: "2026-04-16T10:14:31Z".to_string(),
                    },
                    LocalAdaptationEntry {
                        entry_id: "request-local-adaptation:self-state:preferred_mode:second"
                            .to_string(),
                        subject_ref: "subject://agent/demo".to_string(),
                        target_kind: LocalAdaptationTargetKind::SelfState,
                        key: "preferred_mode".to_string(),
                        payload: LocalAdaptationPayload {
                            value: json!("conservative"),
                            trigger_kind: "request_override".to_string(),
                            evidence_refs: vec![
                                "memo://project/request-local-adaptation-order".to_string(),
                            ],
                        },
                        source_queue_item_id: None,
                        created_at: "2026-04-16T10:14:32Z".to_string(),
                        updated_at: "2026-04-16T10:14:32Z".to_string(),
                    },
                ]),
        )
        .expect("assembly should resolve request local adaptation lifecycle");

    let values = working_memory
        .present
        .self_state
        .facts
        .iter()
        .filter(|fact| fact.key == "self_state:preferred_mode")
        .map(|fact| fact.value.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        values,
        vec!["conservative"],
        "later request-local adaptations should overwrite earlier values for the same logical key"
    );
}

#[test]
fn assembler_prefers_request_local_adaptation_over_subject_state_for_same_key() {
    let path = fresh_db_path("subject-request-local-adaptation-order");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let repository = MemoryRepository::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/subject-request-local-adaptation-order".to_string(),
            source_label: Some("subject-request-local-adaptation-order".to_string()),
            source_kind: None,
            content: "subject and request adaptations should keep override order".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:18Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("subject/request adaptation order ingest should succeed");

    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode".to_string(),
            subject_ref: "subject://agent/demo".to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("conservative"),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec![
                    "memo://project/subject-request-local-adaptation-order".to_string(),
                ],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:30Z".to_string(),
            updated_at: "2026-04-16T10:14:30Z".to_string(),
        })
        .expect("subject local adaptation entry should insert");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("subject and request adaptation precedence")
                .with_subject_ref("subject://agent/demo")
                .with_local_adaptation_entries(vec![LocalAdaptationEntry {
                    entry_id: "request-local-adaptation:self-state:preferred_mode".to_string(),
                    subject_ref: "subject://agent/demo".to_string(),
                    target_kind: LocalAdaptationTargetKind::SelfState,
                    key: "preferred_mode".to_string(),
                    payload: LocalAdaptationPayload {
                        value: json!("aggressive"),
                        trigger_kind: "request_override".to_string(),
                        evidence_refs: vec![
                            "memo://project/subject-request-local-adaptation-order".to_string(),
                        ],
                    },
                    source_queue_item_id: None,
                    created_at: "2026-04-16T10:14:31Z".to_string(),
                    updated_at: "2026-04-16T10:14:31Z".to_string(),
                }]),
        )
        .expect("assembly should resolve subject/request adaptation precedence");

    let values = working_memory
        .present
        .self_state
        .facts
        .iter()
        .filter(|fact| fact.key == "self_state:preferred_mode")
        .map(|fact| fact.value.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        values,
        vec!["aggressive"],
        "request-local adaptation should override repository-backed subject adaptation for the same logical key"
    );
}

#[test]
fn assembler_preserves_request_local_adaptations_when_subject_lookup_is_empty() {
    let path = fresh_db_path("subject-request-local-adaptation-empty-lookup");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/subject-request-local-adaptation-empty-lookup".to_string(),
            source_label: Some("subject-request-local-adaptation-empty-lookup".to_string()),
            source_kind: None,
            content: "request local adaptations should survive an empty subject lookup".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:19Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("subject/request empty-lookup ingest should succeed");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("subject and request adaptation empty lookup")
                .with_subject_ref("subject://agent/unknown")
                .with_local_adaptation_entries(vec![LocalAdaptationEntry {
                    entry_id: "request-local-adaptation:self-state:preferred_mode".to_string(),
                    subject_ref: "subject://agent/unknown".to_string(),
                    target_kind: LocalAdaptationTargetKind::SelfState,
                    key: "preferred_mode".to_string(),
                    payload: LocalAdaptationPayload {
                        value: json!("conservative"),
                        trigger_kind: "request_override".to_string(),
                        evidence_refs: vec![
                            "memo://project/subject-request-local-adaptation-empty-lookup"
                                .to_string(),
                        ],
                    },
                    source_queue_item_id: None,
                    created_at: "2026-04-16T10:14:31Z".to_string(),
                    updated_at: "2026-04-16T10:14:31Z".to_string(),
                }]),
        )
        .expect(
            "assembly should preserve request local adaptations even when subject lookup is empty",
        );

    assert!(
        working_memory
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.key == "self_state:preferred_mode" && fact.value == "conservative"),
        "request-local adaptation should survive even when repository subject lookup returns no rows"
    );
}

#[test]
fn assembler_injects_risk_boundary_and_private_t3_local_adaptations() {
    let path = fresh_db_path("subject-local-adaptation-risk-boundary");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let repository = MemoryRepository::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/subject-local-adaptation-boundaries".to_string(),
            source_label: Some("subject-local-adaptation-boundaries".to_string()),
            source_kind: None,
            content: "risk boundaries and private t3 adaptations should survive assembly"
                .to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:10Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("subject adaptation boundary ingest should succeed");

    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "local-adaptation:risk-boundary:deploy".to_string(),
            subject_ref: "subject://agent/demo".to_string(),
            target_kind: LocalAdaptationTargetKind::RiskBoundary,
            key: "deploy".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("requires_human_review"),
                trigger_kind: "safety_rule".to_string(),
                evidence_refs: vec![
                    "memo://project/subject-local-adaptation-boundaries".to_string(),
                ],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:40Z".to_string(),
            updated_at: "2026-04-16T10:14:40Z".to_string(),
        })
        .expect("risk-boundary local adaptation entry should insert");
    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "local-adaptation:private-t3:hypothesis".to_string(),
            subject_ref: "subject://agent/demo".to_string(),
            target_kind: LocalAdaptationTargetKind::PrivateT3,
            key: "hypothesis".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("prefer_regulative_first"),
                trigger_kind: "private_mapping".to_string(),
                evidence_refs: vec![
                    "memo://project/subject-local-adaptation-boundaries".to_string(),
                ],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:41Z".to_string(),
            updated_at: "2026-04-16T10:14:41Z".to_string(),
        })
        .expect("private-t3 local adaptation entry should insert");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("subject-specific boundaries")
                .with_subject_ref("subject://agent/demo"),
        )
        .expect("assembly should inject risk-boundary and private-t3 local adaptation facts");

    assert!(
        working_memory
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.key == "risk_boundary:deploy" && fact.value == "requires_human_review"),
        "adaptive self-state provider should inject risk-boundary local adaptation facts"
    );
    assert!(
        working_memory
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.key == "private_t3:hypothesis"
                && fact.value == "prefer_regulative_first"),
        "adaptive self-state provider should inject private-t3 local adaptation facts"
    );
}

#[test]
fn assembler_displays_non_string_local_adaptation_payloads_in_self_state() {
    let path = fresh_db_path("subject-local-adaptation-non-string");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let repository = MemoryRepository::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/subject-local-adaptation-non-string".to_string(),
            source_label: Some("subject-local-adaptation-non-string".to_string()),
            source_kind: None,
            content: "non-string local adaptation payloads should display predictably".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:15Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("subject adaptation non-string ingest should succeed");

    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "local-adaptation:risk-boundary:bool".to_string(),
            subject_ref: "subject://agent/demo".to_string(),
            target_kind: LocalAdaptationTargetKind::RiskBoundary,
            key: "deployment_allowed".to_string(),
            payload: LocalAdaptationPayload {
                value: json!(false),
                trigger_kind: "safety_rule".to_string(),
                evidence_refs: vec![
                    "memo://project/subject-local-adaptation-non-string".to_string(),
                ],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:42Z".to_string(),
            updated_at: "2026-04-16T10:14:42Z".to_string(),
        })
        .expect("non-string local adaptation entry should insert");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("subject-specific non-string adaptation")
                .with_subject_ref("subject://agent/demo"),
        )
        .expect("assembly should inject non-string local adaptation facts");

    assert!(
        working_memory
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.key == "risk_boundary:deployment_allowed" && fact.value == "false"),
        "adaptive self-state provider should stringify non-string local adaptation payloads"
    );
}

#[test]
fn assembler_merges_subject_and_request_local_adaptations() {
    let path = fresh_db_path("subject-request-local-adaptation-merge");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let repository = MemoryRepository::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/subject-request-local-adaptation-merge".to_string(),
            source_label: Some("subject-request-local-adaptation-merge".to_string()),
            source_kind: None,
            content: "subject and request local adaptations should merge".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:17Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("subject/request adaptation ingest should succeed");

    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode".to_string(),
            subject_ref: "subject://agent/demo".to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("conservative"),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec![
                    "memo://project/subject-request-local-adaptation-merge".to_string(),
                ],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:30Z".to_string(),
            updated_at: "2026-04-16T10:14:30Z".to_string(),
        })
        .expect("subject local adaptation entry should insert");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("subject and request adaptation")
                .with_subject_ref("subject://agent/demo")
                .with_local_adaptation_entries(vec![LocalAdaptationEntry {
                    entry_id: "request-local-adaptation:risk-boundary:deploy".to_string(),
                    subject_ref: "subject://agent/demo".to_string(),
                    target_kind: LocalAdaptationTargetKind::RiskBoundary,
                    key: "deploy".to_string(),
                    payload: LocalAdaptationPayload {
                        value: json!("requires_human_review"),
                        trigger_kind: "request_override".to_string(),
                        evidence_refs: vec![
                            "memo://project/subject-request-local-adaptation-merge".to_string(),
                        ],
                    },
                    source_queue_item_id: None,
                    created_at: "2026-04-16T10:14:31Z".to_string(),
                    updated_at: "2026-04-16T10:14:31Z".to_string(),
                }]),
        )
        .expect("assembly should merge subject and request local adaptations");

    assert!(
        working_memory
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.key == "self_state:preferred_mode" && fact.value == "conservative"),
        "repository-backed subject adaptation should still be present after merge"
    );
    assert!(
        working_memory
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.key == "risk_boundary:deploy" && fact.value == "requires_human_review"),
        "request-local adaptation should survive alongside repository-backed subject adaptation"
    );
}

#[test]
fn assembler_overwrites_subject_local_adaptations_by_updated_at() {
    let path = fresh_db_path("subject-local-adaptation-order");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let repository = MemoryRepository::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/subject-local-adaptation-order".to_string(),
            source_label: Some("subject-local-adaptation-order".to_string()),
            source_kind: None,
            content: "local adaptation ordering should preserve most recent first".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:20Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("subject adaptation order ingest should succeed");

    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode:old".to_string(),
            subject_ref: "subject://agent/demo".to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("aggressive"),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec!["memo://project/subject-local-adaptation-order".to_string()],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:30Z".to_string(),
            updated_at: "2026-04-16T10:14:30Z".to_string(),
        })
        .expect("older local adaptation entry should insert");
    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode:new".to_string(),
            subject_ref: "subject://agent/demo".to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("conservative"),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec!["memo://project/subject-local-adaptation-order".to_string()],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:31Z".to_string(),
            updated_at: "2026-04-16T10:14:31Z".to_string(),
        })
        .expect("newer local adaptation entry should insert");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("subject-specific adaptation order")
                .with_subject_ref("subject://agent/demo"),
        )
        .expect("assembly should resolve repository local adaptation lifecycle");

    let values = working_memory
        .present
        .self_state
        .facts
        .iter()
        .filter(|fact| fact.key == "self_state:preferred_mode")
        .map(|fact| fact.value.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        values,
        vec!["conservative"],
        "newer repository-backed local adaptations should overwrite older values for the same logical key"
    );
}

#[test]
fn assembler_breaks_subject_local_adaptation_ties_by_entry_id() {
    let path = fresh_db_path("subject-local-adaptation-entry-id-order");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let repository = MemoryRepository::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/subject-local-adaptation-entry-id-order".to_string(),
            source_label: Some("subject-local-adaptation-entry-id-order".to_string()),
            source_kind: None,
            content: "local adaptation tie-break ordering should be deterministic".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:21Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("subject adaptation tie-break ingest should succeed");

    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode:a".to_string(),
            subject_ref: "subject://agent/demo".to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("aggressive"),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec![
                    "memo://project/subject-local-adaptation-entry-id-order".to_string(),
                ],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:30Z".to_string(),
            updated_at: "2026-04-16T10:14:30Z".to_string(),
        })
        .expect("first tie-break local adaptation entry should insert");
    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode:b".to_string(),
            subject_ref: "subject://agent/demo".to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("conservative"),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec![
                    "memo://project/subject-local-adaptation-entry-id-order".to_string(),
                ],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:30Z".to_string(),
            updated_at: "2026-04-16T10:14:30Z".to_string(),
        })
        .expect("second tie-break local adaptation entry should insert");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("subject-specific adaptation tie break")
                .with_subject_ref("subject://agent/demo"),
        )
        .expect("assembly should resolve local adaptation entry-id tie break");

    let values = working_memory
        .present
        .self_state
        .facts
        .iter()
        .filter(|fact| fact.key == "self_state:preferred_mode")
        .map(|fact| fact.value.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        values,
        vec!["conservative"],
        "repository-backed local adaptations should deterministically prefer the higher entry_id when updated_at ties"
    );
}

#[test]
fn assembler_hides_inactive_local_adaptations_after_latest_tombstone() {
    let path = fresh_db_path("subject-local-adaptation-inactive");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let repository = MemoryRepository::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/subject-local-adaptation-inactive".to_string(),
            source_label: Some("subject-local-adaptation-inactive".to_string()),
            source_kind: None,
            content: "inactive local adaptations should suppress older active values".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:22Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("subject adaptation inactive ingest should succeed");

    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode:active".to_string(),
            subject_ref: "subject://agent/demo".to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("conservative"),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec!["memo://project/subject-local-adaptation-inactive".to_string()],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:30Z".to_string(),
            updated_at: "2026-04-16T10:14:30Z".to_string(),
        })
        .expect("active local adaptation entry should insert");
    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode:inactive".to_string(),
            subject_ref: "subject://agent/demo".to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!({
                    "active": false,
                    "reason": "paused for this session"
                }),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec!["memo://project/subject-local-adaptation-inactive".to_string()],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:31Z".to_string(),
            updated_at: "2026-04-16T10:14:31Z".to_string(),
        })
        .expect("inactive local adaptation entry should insert");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("subject-specific inactive adaptation")
                .with_subject_ref("subject://agent/demo"),
        )
        .expect("assembly should suppress inactive local adaptation values");

    assert!(
        !working_memory
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.key == "self_state:preferred_mode"),
        "inactive winner should remove the logical key from surfaced self-state facts"
    );
}

#[test]
fn assembler_preserves_explicit_persisted_self_model_seam() {
    let path = fresh_db_path("explicit-persisted-self-model");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let repository = MemoryRepository::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/explicit-persisted-self-model".to_string(),
            source_label: Some("explicit-persisted-self-model".to_string()),
            source_kind: None,
            content: "explicit self-model read model should remain the projection seam".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:23Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("explicit persisted self-model ingest should succeed");

    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode".to_string(),
            subject_ref: "subject://agent/demo".to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("conservative"),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec!["memo://project/explicit-persisted-self-model".to_string()],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:30Z".to_string(),
            updated_at: "2026-04-16T10:14:30Z".to_string(),
        })
        .expect("subject local adaptation entry should insert");

    let explicit_read_model = SelfModelReadModel::from_overlay_entries(&[LocalAdaptationEntry {
        entry_id: "request-local-adaptation:self-state:preferred_mode".to_string(),
        subject_ref: "subject://agent/demo".to_string(),
        target_kind: LocalAdaptationTargetKind::SelfState,
        key: "preferred_mode".to_string(),
        payload: LocalAdaptationPayload {
            value: json!("aggressive"),
            trigger_kind: "request_override".to_string(),
            evidence_refs: vec!["memo://project/explicit-persisted-self-model".to_string()],
        },
        source_queue_item_id: None,
        created_at: "2026-04-16T10:14:31Z".to_string(),
        updated_at: "2026-04-16T10:14:31Z".to_string(),
    }]);

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("explicit self-model read model")
                .with_subject_ref("subject://agent/demo")
                .with_persisted_self_model(explicit_read_model),
        )
        .expect("assembly should preserve the explicit self-model read-model seam");

    let values = working_memory
        .present
        .self_state
        .facts
        .iter()
        .filter(|fact| fact.key == "self_state:preferred_mode")
        .map(|fact| fact.value.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        values,
        vec!["aggressive"],
        "assembly should respect an explicit persisted self-model instead of rebuilding from subject ledger rows"
    );
}

#[test]
fn assembler_loads_active_consumed_skill_template_winners_additively() {
    let path = fresh_db_path("persisted-skill-template-runtime-read-model");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let repository = MemoryRepository::new(db.conn());
    let subject_ref = "subject://agent/demo";

    let explicit_support = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/runtime-skill-template-explicit".to_string(),
            source_label: Some("runtime-skill-template-explicit".to_string()),
            source_kind: None,
            content: "runtime skill template explicit support stays explainable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-29T09:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("explicit support ingest should succeed")
        .record_ids[0]
        .clone();
    let persisted_support = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/runtime-skill-template-persisted".to_string(),
            source_label: Some("runtime-skill-template-persisted".to_string()),
            source_kind: None,
            content: "runtime skill template persisted support stays explainable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-29T09:01:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("persisted support ingest should succeed")
        .record_ids[0]
        .clone();

    let mut explicit_template = SkillMemoryTemplate::new(
        "explicit-template",
        ActionTemplate::new(ActionKind::Regulative, "use the caller supplied template")
            .with_intent("prefer the explicit request seam"),
    );
    explicit_template.preconditions = Preconditions {
        required_goal_terms: vec!["clarify".to_string()],
        required_capability_flags: vec!["skill_projection_ready".to_string()],
        required_readiness_flags: vec!["citation_trace_ready".to_string()],
        ..Preconditions::default()
    };
    explicit_template.expected_outcome = ExpectedOutcome {
        effects: vec!["explicit action stays active".to_string()],
    };
    explicit_template.boundaries = Boundaries {
        risk_markers: vec!["explicit_skill".to_string()],
        supporting_record_ids: vec![explicit_support.clone()],
        blocked_active_risks: Vec::new(),
    };

    let build_persisted_template = |template_id: &str, summary: &str, support_id: &str| {
        let mut template = SkillMemoryTemplate::new(
            template_id,
            ActionTemplate::new(ActionKind::Instrumental, summary)
                .with_intent("reuse a consumed persisted template"),
        );
        template.preconditions = Preconditions {
            required_goal_terms: vec!["clarify".to_string()],
            required_task_context_terms: vec!["runtime".to_string()],
            required_capability_flags: vec!["skill_projection_ready".to_string()],
            required_readiness_flags: vec!["citation_trace_ready".to_string()],
            ..Preconditions::default()
        };
        template.expected_outcome = ExpectedOutcome {
            effects: vec!["persisted action becomes branch-ready".to_string()],
        };
        template.boundaries = Boundaries {
            risk_markers: vec!["persisted_skill".to_string()],
            supporting_record_ids: vec![support_id.to_string()],
            blocked_active_risks: Vec::new(),
        };
        template
    };

    let consumed_template = build_persisted_template(
        "persisted-consumed-template",
        "load the consumed persisted template",
        &persisted_support,
    );
    let superseded_template = build_persisted_template(
        "persisted-consumed-template",
        "older duplicate consumed template should stay inactive",
        &persisted_support,
    );

    let mut older_duplicate = persisted_skill_template_candidate(
        "lpq:subject-a:consumed:older",
        subject_ref,
        RuminationCandidateStatus::Consumed,
        "2026-04-29T09:09:00Z",
        &superseded_template,
    );
    older_duplicate.updated_at = "2026-04-29T09:09:00Z".to_string();

    let mut consumed_winner = persisted_skill_template_candidate(
        "lpq:subject-a:consumed:winner",
        subject_ref,
        RuminationCandidateStatus::Consumed,
        "2026-04-29T09:10:00Z",
        &consumed_template,
    );
    consumed_winner.updated_at = "2026-04-29T09:15:00Z".to_string();

    for candidate in [
        older_duplicate,
        consumed_winner,
        persisted_skill_template_candidate(
            "lpq:subject-a:pending",
            subject_ref,
            RuminationCandidateStatus::Pending,
            "2026-04-29T09:11:00Z",
            &build_persisted_template(
                "persisted-pending-template",
                "do not load the pending template",
                &persisted_support,
            ),
        ),
        persisted_skill_template_candidate(
            "lpq:subject-a:rejected",
            subject_ref,
            RuminationCandidateStatus::Rejected,
            "2026-04-29T09:12:00Z",
            &build_persisted_template(
                "persisted-rejected-template",
                "do not load the rejected template",
                &persisted_support,
            ),
        ),
        persisted_skill_template_candidate(
            "lpq:subject-a:archived",
            subject_ref,
            RuminationCandidateStatus::Archived,
            "2026-04-29T09:13:00Z",
            &build_persisted_template(
                "persisted-archived-template",
                "do not load the archived template",
                &persisted_support,
            ),
        ),
        persisted_skill_template_candidate(
            "lpq:subject-b:consumed",
            "subject://agent/other",
            RuminationCandidateStatus::Consumed,
            "2026-04-29T09:14:00Z",
            &build_persisted_template(
                "persisted-other-subject-template",
                "do not load the other subject template",
                &persisted_support,
            ),
        ),
    ] {
        repository
            .insert_rumination_candidate(&candidate)
            .expect("skill template candidate should insert");
    }

    let assembler = WorkingMemoryAssembler::new(db.conn(), MinimalSelfStateProvider);
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("runtime skill template")
                .with_subject_ref(subject_ref)
                .with_task_context("runtime assembly")
                .with_active_goal("clarify the next step safely")
                .with_capability_flag("skill_projection_ready")
                .with_readiness_flag("citation_trace_ready")
                .with_skill_template(explicit_template),
        )
        .expect("assembly should load consumed persisted skill templates");

    assert_eq!(working_memory.branches.len(), 2);
    assert_eq!(
        working_memory
            .branches
            .iter()
            .map(|branch| branch.candidate.summary.as_str())
            .collect::<Vec<_>>(),
        vec![
            "use the caller supplied template",
            "load the consumed persisted template",
        ]
    );
    assert_eq!(
        working_memory.branches[0]
            .supporting_evidence
            .iter()
            .map(|fragment| fragment.record_id.as_str())
            .collect::<Vec<_>>(),
        vec![explicit_support.as_str()]
    );
    assert_eq!(
        working_memory.branches[1]
            .supporting_evidence
            .iter()
            .map(|fragment| fragment.record_id.as_str())
            .collect::<Vec<_>>(),
        vec![persisted_support.as_str()]
    );
    assert_eq!(
        working_memory.branches[1].source,
        ActionSource::SkillTemplate {
            template_id: "persisted-consumed-template".to_string(),
        }
    );
    assert!(
        working_memory
            .branches
            .iter()
            .all(|branch| !branch.candidate.summary.contains("do not load")),
        "pending, rejected, archived, and non-subject candidates must stay inactive"
    );
    assert!(
        working_memory.branches.iter().all(|branch| {
            branch.candidate.summary != "older duplicate consumed template should stay inactive"
        }),
        "duplicate consumed candidates should resolve through the active read model before branch projection"
    );
}

#[test]
fn assembler_suppresses_tombstoned_persisted_skill_templates_but_keeps_explicit_templates() {
    let path = fresh_db_path("persisted-skill-template-runtime-tombstones");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let repository = MemoryRepository::new(db.conn());
    let subject_ref = "subject://agent/tombstones";

    let explicit_support = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/runtime-tombstone-explicit".to_string(),
            source_label: Some("runtime-tombstone-explicit".to_string()),
            source_kind: None,
            content: "explicit runtime template support should remain active".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-30T08:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("explicit support ingest should succeed")
        .record_ids[0]
        .clone();
    let persisted_support = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/runtime-tombstone-persisted".to_string(),
            source_label: Some("runtime-tombstone-persisted".to_string()),
            source_kind: None,
            content: "persisted runtime template support should remain explainable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-30T08:01:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("persisted support ingest should succeed")
        .record_ids[0]
        .clone();

    let mut explicit_template = SkillMemoryTemplate::new(
        "explicit-template",
        ActionTemplate::new(ActionKind::Regulative, "use the caller supplied template"),
    );
    explicit_template.preconditions = Preconditions {
        required_goal_terms: vec!["clarify".to_string()],
        required_capability_flags: vec!["skill_projection_ready".to_string()],
        required_readiness_flags: vec!["citation_trace_ready".to_string()],
        ..Preconditions::default()
    };
    explicit_template.boundaries.supporting_record_ids = vec![explicit_support.clone()];

    let build_persisted_template = |template_id: &str, summary: &str| {
        let mut template = SkillMemoryTemplate::new(
            template_id,
            ActionTemplate::new(ActionKind::Instrumental, summary),
        );
        template.preconditions = Preconditions {
            required_goal_terms: vec!["clarify".to_string()],
            required_task_context_terms: vec!["runtime".to_string()],
            required_capability_flags: vec!["skill_projection_ready".to_string()],
            required_readiness_flags: vec!["citation_trace_ready".to_string()],
            ..Preconditions::default()
        };
        template.boundaries = Boundaries {
            risk_markers: vec!["persisted_skill".to_string()],
            supporting_record_ids: vec![persisted_support.clone()],
            blocked_active_risks: Vec::new(),
        };
        template
    };

    let mut rejected_consumed = persisted_skill_template_candidate(
        "lpq:tombstone:rejected:consumed",
        subject_ref,
        RuminationCandidateStatus::Consumed,
        "2026-04-30T08:05:00Z",
        &build_persisted_template(
            "rejected-template",
            "earlier consumed template should be suppressed by rejection",
        ),
    );
    rejected_consumed.updated_at = "2026-04-30T08:05:00Z".to_string();

    let mut rejected_tombstone = persisted_skill_template_candidate(
        "lpq:tombstone:rejected:tombstone",
        subject_ref,
        RuminationCandidateStatus::Rejected,
        "2026-04-30T08:06:00Z",
        &build_persisted_template(
            "rejected-template",
            "later rejected tombstone should stay inactive",
        ),
    );
    rejected_tombstone.updated_at = "2026-04-30T08:10:00Z".to_string();

    let mut archived_consumed = persisted_skill_template_candidate(
        "lpq:tombstone:archived:consumed",
        subject_ref,
        RuminationCandidateStatus::Consumed,
        "2026-04-30T08:07:00Z",
        &build_persisted_template(
            "archived-template",
            "earlier consumed template should be suppressed by archive",
        ),
    );
    archived_consumed.updated_at = "2026-04-30T08:07:00Z".to_string();

    let mut archived_tombstone = persisted_skill_template_candidate(
        "lpq:tombstone:archived:tombstone",
        subject_ref,
        RuminationCandidateStatus::Archived,
        "2026-04-30T08:08:00Z",
        &build_persisted_template(
            "archived-template",
            "later archived tombstone should stay inactive",
        ),
    );
    archived_tombstone.updated_at = "2026-04-30T08:11:00Z".to_string();

    let mut active_consumed = persisted_skill_template_candidate(
        "lpq:tombstone:active",
        subject_ref,
        RuminationCandidateStatus::Consumed,
        "2026-04-30T08:09:00Z",
        &build_persisted_template(
            "active-template",
            "keep the active consumed persisted template",
        ),
    );
    active_consumed.updated_at = "2026-04-30T08:09:00Z".to_string();

    for candidate in [
        rejected_consumed,
        rejected_tombstone,
        archived_consumed,
        archived_tombstone,
        active_consumed,
    ] {
        repository
            .insert_rumination_candidate(&candidate)
            .expect("skill template candidate should insert");
    }

    let assembler = WorkingMemoryAssembler::new(db.conn(), MinimalSelfStateProvider);
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("runtime skill template tombstones")
                .with_subject_ref(subject_ref)
                .with_task_context("runtime assembly")
                .with_active_goal("clarify the next step safely")
                .with_capability_flag("skill_projection_ready")
                .with_readiness_flag("citation_trace_ready")
                .with_skill_template(explicit_template),
        )
        .expect("assembly should suppress tombstoned persisted templates");

    assert_eq!(working_memory.branches.len(), 2);
    assert_eq!(
        working_memory
            .branches
            .iter()
            .map(|branch| branch.candidate.summary.as_str())
            .collect::<Vec<_>>(),
        vec![
            "use the caller supplied template",
            "keep the active consumed persisted template",
        ]
    );
    assert_eq!(
        working_memory.branches[0]
            .supporting_evidence
            .iter()
            .map(|fragment| fragment.record_id.as_str())
            .collect::<Vec<_>>(),
        vec![explicit_support.as_str()]
    );
    assert_eq!(
        working_memory.branches[1]
            .supporting_evidence
            .iter()
            .map(|fragment| fragment.record_id.as_str())
            .collect::<Vec<_>>(),
        vec![persisted_support.as_str()]
    );
    assert!(
        working_memory.branches.iter().all(|branch| {
            !branch.candidate.summary.contains("should be suppressed by")
                && !branch.candidate.summary.contains("tombstone")
        }),
        "later rejected or archived rows must suppress earlier consumed persisted templates"
    );
}

#[test]
fn assembler_activates_persisted_skill_templates_only_after_consume_transition() {
    let path = fresh_db_path("persisted-skill-template-lifecycle-activation");
    let db = Database::open(&path).expect("database should open");
    let repository = MemoryRepository::new(db.conn());
    let subject_ref = "subject://agent/lifecycle";

    let mut template = SkillMemoryTemplate::new(
        "persisted-lifecycle-template",
        ActionTemplate::new(
            ActionKind::Instrumental,
            "activate only after the consumed lifecycle transition",
        )
        .with_intent("keep runtime activation gated by explicit review"),
    );
    template.preconditions = Preconditions {
        required_goal_terms: vec!["clarify".to_string()],
        required_task_context_terms: vec!["runtime".to_string()],
        required_capability_flags: vec!["skill_projection_ready".to_string()],
        required_readiness_flags: vec!["citation_trace_ready".to_string()],
        ..Preconditions::default()
    };
    template.expected_outcome = ExpectedOutcome {
        effects: vec!["pending candidates stay inactive until consumed".to_string()],
    };

    repository
        .insert_rumination_candidate(&persisted_skill_template_candidate(
            "lpq:lifecycle:pending",
            subject_ref,
            RuminationCandidateStatus::Pending,
            "2026-04-29T09:30:00Z",
            &template,
        ))
        .expect("pending skill template candidate should insert");

    let request = WorkingMemoryRequest::new("runtime lifecycle bridge")
        .with_subject_ref(subject_ref)
        .with_task_context("runtime assembly")
        .with_active_goal("clarify the next step safely")
        .with_capability_flag("skill_projection_ready")
        .with_readiness_flag("citation_trace_ready");
    let assembler = WorkingMemoryAssembler::new(db.conn(), MinimalSelfStateProvider);

    let pending_working_memory = assembler
        .assemble(&request)
        .expect("assembly should succeed while the candidate is still pending");
    assert!(
        pending_working_memory.branches.is_empty(),
        "pending candidates must remain inactive"
    );

    repository
        .consume_skill_template_candidate("lpq:lifecycle:pending", "2026-04-29T09:31:00Z")
        .expect("consume transition should succeed");

    let consumed_working_memory = assembler
        .assemble(&request)
        .expect("assembly should load the consumed candidate");
    assert_eq!(consumed_working_memory.branches.len(), 1);
    assert_eq!(
        consumed_working_memory.branches[0].candidate.summary,
        "activate only after the consumed lifecycle transition"
    );
    assert_eq!(
        consumed_working_memory.branches[0]
            .candidate
            .intent
            .as_deref(),
        Some("keep runtime activation gated by explicit review")
    );
}

#[test]
fn self_model_read_model_fails_closed_unresolved_governed_conflicts() {
    let read_model = SelfModelReadModel::from_entries(
        &[
            LocalAdaptationEntry {
                entry_id: "local-adaptation:self-state:preferred_mode:v1".to_string(),
                subject_ref: "subject://agent/demo".to_string(),
                target_kind: LocalAdaptationTargetKind::SelfState,
                key: "preferred_mode".to_string(),
                payload: LocalAdaptationPayload {
                    value: json!("conservative"),
                    trigger_kind: "user_preference".to_string(),
                    evidence_refs: vec!["memo://project/self-model-governance".to_string()],
                },
                source_queue_item_id: None,
                created_at: "2026-04-16T10:14:30Z".to_string(),
                updated_at: "2026-04-16T10:14:30Z".to_string(),
            },
            LocalAdaptationEntry {
                entry_id: "local-adaptation:self-state:preferred_mode:v2".to_string(),
                subject_ref: "subject://agent/demo".to_string(),
                target_kind: LocalAdaptationTargetKind::SelfState,
                key: "preferred_mode".to_string(),
                payload: LocalAdaptationPayload {
                    value: json!({
                        "value": "aggressive",
                        "governance": {
                            "resolution": "unresolved",
                            "review_reason": "material preference conflict"
                        }
                    }),
                    trigger_kind: "self_model_review".to_string(),
                    evidence_refs: vec!["memo://project/self-model-governance".to_string()],
                },
                source_queue_item_id: Some("rumination-queue-1".to_string()),
                created_at: "2026-04-16T10:14:31Z".to_string(),
                updated_at: "2026-04-16T10:14:31Z".to_string(),
            },
        ],
        &[],
    );

    assert_eq!(read_model.entries().len(), 1);
    assert_eq!(
        read_model.entries()[0].governance_resolution(),
        Some(SelfModelResolutionState::Unresolved)
    );
    assert_eq!(
        read_model.entries()[0]
            .governance
            .as_ref()
            .expect("governance metadata should be preserved")
            .conflicting_entry_ids,
        vec!["local-adaptation:self-state:preferred_mode:v1".to_string()]
    );
    assert!(
        read_model.active_facts().is_empty(),
        "unresolved governed conflicts must fail closed instead of surfacing a mechanical winner"
    );
}

#[test]
fn self_model_read_model_fails_closed_when_governed_conflict_is_overwritten_without_review() {
    let read_model = SelfModelReadModel::from_entries(
        &[
            LocalAdaptationEntry {
                entry_id: "local-adaptation:self-state:preferred_mode:v1".to_string(),
                subject_ref: "subject://agent/demo".to_string(),
                target_kind: LocalAdaptationTargetKind::SelfState,
                key: "preferred_mode".to_string(),
                payload: LocalAdaptationPayload {
                    value: json!({
                        "value": "conservative",
                        "governance": {
                            "resolution": "unresolved",
                            "review_reason": "material preference conflict"
                        }
                    }),
                    trigger_kind: "self_model_review".to_string(),
                    evidence_refs: vec!["memo://project/self-model-governance".to_string()],
                },
                source_queue_item_id: Some("rumination-queue-plain".to_string()),
                created_at: "2026-04-16T10:14:30Z".to_string(),
                updated_at: "2026-04-16T10:14:30Z".to_string(),
            },
            LocalAdaptationEntry {
                entry_id: "local-adaptation:self-state:preferred_mode:v2".to_string(),
                subject_ref: "subject://agent/demo".to_string(),
                target_kind: LocalAdaptationTargetKind::SelfState,
                key: "preferred_mode".to_string(),
                payload: LocalAdaptationPayload {
                    value: json!("aggressive"),
                    trigger_kind: "recent_override".to_string(),
                    evidence_refs: vec!["memo://project/self-model-governance".to_string()],
                },
                source_queue_item_id: None,
                created_at: "2026-04-16T10:14:31Z".to_string(),
                updated_at: "2026-04-16T10:14:31Z".to_string(),
            },
        ],
        &[],
    );

    assert_eq!(read_model.entries().len(), 1);
    assert_eq!(
        read_model.entries()[0].governance_resolution(),
        Some(SelfModelResolutionState::Unresolved)
    );
    assert_eq!(
        read_model.entries()[0]
            .governance
            .as_ref()
            .expect("governed conflict should synthesize governance metadata on the winner")
            .conflicting_entry_ids,
        vec!["local-adaptation:self-state:preferred_mode:v1".to_string()]
    );
    assert!(
        read_model.active_facts().is_empty(),
        "plain overwrites must not bypass a same-key governed conflict that is still unresolved"
    );
}

#[test]
fn self_model_read_model_surfaces_accepted_governed_conflicts_deterministically() {
    let read_model = SelfModelReadModel::from_entries(
        &[
            LocalAdaptationEntry {
                entry_id: "local-adaptation:self-state:preferred_mode:v1".to_string(),
                subject_ref: "subject://agent/demo".to_string(),
                target_kind: LocalAdaptationTargetKind::SelfState,
                key: "preferred_mode".to_string(),
                payload: LocalAdaptationPayload {
                    value: json!("conservative"),
                    trigger_kind: "user_preference".to_string(),
                    evidence_refs: vec!["memo://project/self-model-governance".to_string()],
                },
                source_queue_item_id: None,
                created_at: "2026-04-16T10:14:30Z".to_string(),
                updated_at: "2026-04-16T10:14:30Z".to_string(),
            },
            LocalAdaptationEntry {
                entry_id: "local-adaptation:self-state:preferred_mode:v2".to_string(),
                subject_ref: "subject://agent/demo".to_string(),
                target_kind: LocalAdaptationTargetKind::SelfState,
                key: "preferred_mode".to_string(),
                payload: LocalAdaptationPayload {
                    value: json!({
                        "value": "aggressive",
                        "governance": {
                            "resolution": "accepted",
                            "review_reason": "approved override"
                        }
                    }),
                    trigger_kind: "self_model_review".to_string(),
                    evidence_refs: vec!["memo://project/self-model-governance".to_string()],
                },
                source_queue_item_id: Some("rumination-queue-2".to_string()),
                created_at: "2026-04-16T10:14:31Z".to_string(),
                updated_at: "2026-04-16T10:14:31Z".to_string(),
            },
        ],
        &[],
    );

    assert_eq!(
        read_model.entries()[0].governance_resolution(),
        Some(SelfModelResolutionState::Accepted)
    );
    assert_eq!(
        read_model.active_facts(),
        vec![SelfStateFact {
            key: "self_state:preferred_mode".to_string(),
            value: "aggressive".to_string(),
            source_record_id: None,
        }]
    );
}

#[test]
fn self_model_read_model_fails_closed_invalid_governance_metadata() {
    let read_model = SelfModelReadModel::from_entries(
        &[LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode:v1".to_string(),
            subject_ref: "subject://agent/demo".to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!({
                    "value": "aggressive",
                    "governance": {
                        "resolution": "acceptd",
                        "review_reason": "typo should not bypass governance"
                    }
                }),
                trigger_kind: "self_model_review".to_string(),
                evidence_refs: vec!["memo://project/self-model-governance".to_string()],
            },
            source_queue_item_id: Some("rumination-queue-invalid".to_string()),
            created_at: "2026-04-16T10:14:31Z".to_string(),
            updated_at: "2026-04-16T10:14:31Z".to_string(),
        }],
        &[],
    );

    assert_eq!(
        read_model.entries()[0].governance_resolution(),
        Some(SelfModelResolutionState::Unresolved)
    );
    assert_eq!(
        read_model.entries()[0]
            .governance
            .as_ref()
            .expect("invalid governance metadata should fail closed")
            .review_reason
            .as_deref(),
        Some("invalid governance metadata")
    );
    assert!(
        read_model.active_facts().is_empty(),
        "invalid governance metadata must not silently downgrade into an ungoverned mechanical winner"
    );
}

#[test]
fn self_model_read_model_hides_rejected_snapshot_governance_metadata() {
    let snapshot = PersistedSelfModelSnapshot {
        subject_ref: "subject://agent/demo".to_string(),
        snapshot_id: "self-model-snapshot-governance-001".to_string(),
        entries: vec![PersistedSelfModelSnapshotEntry {
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            value: "aggressive".to_string(),
            active: true,
            governance: Some(SelfModelGovernanceMetadata {
                resolution: SelfModelResolutionState::Rejected,
                conflicting_entry_ids: vec![
                    "local-adaptation:self-state:preferred_mode:v1".to_string(),
                ],
                review_reason: Some("review rejected the override".to_string()),
            }),
            source_queue_item_id: Some("rumination-queue-3".to_string()),
            updated_at: "2026-04-16T10:15:00Z".to_string(),
            entry_id: "self-model-snapshot-entry:preferred_mode:v2".to_string(),
        }],
        compacted_through_updated_at: "2026-04-16T10:15:00Z".to_string(),
        compacted_through_entry_id: "self-model-snapshot-entry:preferred_mode:v2".to_string(),
        created_at: "2026-04-16T10:15:05Z".to_string(),
        updated_at: "2026-04-16T10:15:05Z".to_string(),
    };
    let read_model = SelfModelReadModel::from_persisted_state(Some(&snapshot), &[], &[]);

    assert_eq!(
        read_model.entries()[0].governance_resolution(),
        Some(SelfModelResolutionState::Rejected)
    );
    assert!(
        read_model.active_facts().is_empty(),
        "rejected governance metadata should suppress the compacted winner deterministically"
    );
}

#[test]
fn assembler_rebuilds_persisted_self_model_from_snapshot_and_ledger_tail() {
    let path = fresh_db_path("self-model-snapshot-tail-rebuild");
    let db = Database::open(&path).expect("database should open");
    let repository = MemoryRepository::new(db.conn());
    let subject_ref = "subject://agent/demo";

    for entry in [
        LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode:v1".to_string(),
            subject_ref: subject_ref.to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("conservative"),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec!["memo://project/self-model-snapshot-tail".to_string()],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:30Z".to_string(),
            updated_at: "2026-04-16T10:14:30Z".to_string(),
        },
        LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode:v2".to_string(),
            subject_ref: subject_ref.to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("aggressive"),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec!["memo://project/self-model-snapshot-tail".to_string()],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:31Z".to_string(),
            updated_at: "2026-04-16T10:14:31Z".to_string(),
        },
        LocalAdaptationEntry {
            entry_id: "local-adaptation:risk-boundary:deploy:v1".to_string(),
            subject_ref: subject_ref.to_string(),
            target_kind: LocalAdaptationTargetKind::RiskBoundary,
            key: "deploy".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("requires_human_review"),
                trigger_kind: "risk_rule".to_string(),
                evidence_refs: vec!["memo://project/self-model-snapshot-tail".to_string()],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:32Z".to_string(),
            updated_at: "2026-04-16T10:14:32Z".to_string(),
        },
    ] {
        repository
            .insert_local_adaptation_entry(&entry)
            .expect("local adaptation entry should insert");
    }

    let snapshot = compact_self_model_subject(
        &repository,
        subject_ref,
        "self-model-snapshot-001",
        "2026-04-16T10:15:00Z",
    )
    .expect("compaction should succeed")
    .expect("compaction should produce a snapshot");
    assert_eq!(snapshot.entries.len(), 2);
    assert_eq!(
        snapshot.compacted_through_updated_at,
        "2026-04-16T10:14:32Z"
    );
    assert!(
        repository
            .list_local_adaptation_entries(subject_ref)
            .expect("ledger rows should load after compaction")
            .is_empty(),
        "compaction should prune ledger rows already represented by the snapshot"
    );

    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode:v3".to_string(),
            subject_ref: subject_ref.to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("exploratory"),
                trigger_kind: "recent_override".to_string(),
                evidence_refs: vec!["memo://project/self-model-snapshot-tail".to_string()],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:15:10Z".to_string(),
            updated_at: "2026-04-16T10:15:10Z".to_string(),
        })
        .expect("tail ledger row should insert after compaction");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("self model snapshot tail rebuild")
                .with_subject_ref(subject_ref),
        )
        .expect("assembly should rebuild persisted self-model from snapshot and ledger tail");

    let resolved = working_memory
        .present
        .self_state
        .facts
        .iter()
        .filter(|fact| {
            fact.key == "self_state:preferred_mode" || fact.key == "risk_boundary:deploy"
        })
        .map(|fact| (fact.key.clone(), fact.value.clone()))
        .collect::<Vec<_>>();
    assert_eq!(
        resolved,
        vec![
            (
                "self_state:preferred_mode".to_string(),
                "exploratory".to_string(),
            ),
            (
                "risk_boundary:deploy".to_string(),
                "requires_human_review".to_string(),
            ),
        ],
        "assembler should merge compacted snapshot state with newer ledger tail rows through the existing self-model seam"
    );
}

#[test]
fn assembler_keeps_inactive_snapshot_winners_suppressive_after_compaction() {
    let path = fresh_db_path("self-model-snapshot-inactive");
    let db = Database::open(&path).expect("database should open");
    let repository = MemoryRepository::new(db.conn());
    let subject_ref = "subject://agent/demo";

    for entry in [
        LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode:v1".to_string(),
            subject_ref: subject_ref.to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("conservative"),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec!["memo://project/self-model-snapshot-inactive".to_string()],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:30Z".to_string(),
            updated_at: "2026-04-16T10:14:30Z".to_string(),
        },
        LocalAdaptationEntry {
            entry_id: "local-adaptation:self-state:preferred_mode:tombstone".to_string(),
            subject_ref: subject_ref.to_string(),
            target_kind: LocalAdaptationTargetKind::SelfState,
            key: "preferred_mode".to_string(),
            payload: LocalAdaptationPayload {
                value: json!({"active": false, "status": "inactive"}),
                trigger_kind: "user_preference".to_string(),
                evidence_refs: vec!["memo://project/self-model-snapshot-inactive".to_string()],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:14:31Z".to_string(),
            updated_at: "2026-04-16T10:14:31Z".to_string(),
        },
    ] {
        repository
            .insert_local_adaptation_entry(&entry)
            .expect("local adaptation entry should insert");
    }

    compact_self_model_subject(
        &repository,
        subject_ref,
        "self-model-snapshot-002",
        "2026-04-16T10:15:00Z",
    )
    .expect("compaction should succeed")
    .expect("compaction should produce a snapshot");

    repository
        .insert_local_adaptation_entry(&LocalAdaptationEntry {
            entry_id: "local-adaptation:risk-boundary:deploy:v1".to_string(),
            subject_ref: subject_ref.to_string(),
            target_kind: LocalAdaptationTargetKind::RiskBoundary,
            key: "deploy".to_string(),
            payload: LocalAdaptationPayload {
                value: json!("requires_human_review"),
                trigger_kind: "risk_rule".to_string(),
                evidence_refs: vec!["memo://project/self-model-snapshot-inactive".to_string()],
            },
            source_queue_item_id: None,
            created_at: "2026-04-16T10:15:10Z".to_string(),
            updated_at: "2026-04-16T10:15:10Z".to_string(),
        })
        .expect("tail ledger row should insert after compaction");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("self model snapshot inactive winner")
                .with_subject_ref(subject_ref),
        )
        .expect("assembly should preserve inactive snapshot winners");

    assert!(
        !working_memory
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.key == "self_state:preferred_mode"),
        "inactive snapshot winners should remain suppressive after compaction"
    );
    assert!(
        working_memory
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.key == "risk_boundary:deploy" && fact.value == "requires_human_review"),
        "newer ledger tail rows should still surface alongside compacted inactive winners"
    );
}

#[test]
fn assembler_orders_resolved_local_adaptations_by_target_kind_then_key() {
    let path = fresh_db_path("local-adaptation-target-kind-order");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/local-adaptation-target-kind-order".to_string(),
            source_label: Some("local-adaptation-target-kind-order".to_string()),
            source_kind: None,
            content: "resolved local adaptation order should be deterministic".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:23Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("target-kind order ingest should succeed");

    let assembler = WorkingMemoryAssembler::new(
        db.conn(),
        AdaptiveSelfStateProvider::new(TestSelfStateProvider),
    );
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("deterministic local adaptation order")
                .with_local_adaptation_entries(vec![
                    LocalAdaptationEntry {
                        entry_id: "request-local-adaptation:private-t3:zeta".to_string(),
                        subject_ref: "subject://agent/demo".to_string(),
                        target_kind: LocalAdaptationTargetKind::PrivateT3,
                        key: "zeta".to_string(),
                        payload: LocalAdaptationPayload {
                            value: json!("tentative"),
                            trigger_kind: "request_override".to_string(),
                            evidence_refs: vec![
                                "memo://project/local-adaptation-target-kind-order".to_string(),
                            ],
                        },
                        source_queue_item_id: None,
                        created_at: "2026-04-16T10:14:33Z".to_string(),
                        updated_at: "2026-04-16T10:14:33Z".to_string(),
                    },
                    LocalAdaptationEntry {
                        entry_id: "request-local-adaptation:self-state:beta".to_string(),
                        subject_ref: "subject://agent/demo".to_string(),
                        target_kind: LocalAdaptationTargetKind::SelfState,
                        key: "beta".to_string(),
                        payload: LocalAdaptationPayload {
                            value: json!("two"),
                            trigger_kind: "request_override".to_string(),
                            evidence_refs: vec![
                                "memo://project/local-adaptation-target-kind-order".to_string(),
                            ],
                        },
                        source_queue_item_id: None,
                        created_at: "2026-04-16T10:14:31Z".to_string(),
                        updated_at: "2026-04-16T10:14:31Z".to_string(),
                    },
                    LocalAdaptationEntry {
                        entry_id: "request-local-adaptation:risk-boundary:alpha".to_string(),
                        subject_ref: "subject://agent/demo".to_string(),
                        target_kind: LocalAdaptationTargetKind::RiskBoundary,
                        key: "alpha".to_string(),
                        payload: LocalAdaptationPayload {
                            value: json!("review"),
                            trigger_kind: "request_override".to_string(),
                            evidence_refs: vec![
                                "memo://project/local-adaptation-target-kind-order".to_string(),
                            ],
                        },
                        source_queue_item_id: None,
                        created_at: "2026-04-16T10:14:32Z".to_string(),
                        updated_at: "2026-04-16T10:14:32Z".to_string(),
                    },
                    LocalAdaptationEntry {
                        entry_id: "request-local-adaptation:self-state:alpha".to_string(),
                        subject_ref: "subject://agent/demo".to_string(),
                        target_kind: LocalAdaptationTargetKind::SelfState,
                        key: "alpha".to_string(),
                        payload: LocalAdaptationPayload {
                            value: json!("one"),
                            trigger_kind: "request_override".to_string(),
                            evidence_refs: vec![
                                "memo://project/local-adaptation-target-kind-order".to_string(),
                            ],
                        },
                        source_queue_item_id: None,
                        created_at: "2026-04-16T10:14:30Z".to_string(),
                        updated_at: "2026-04-16T10:14:30Z".to_string(),
                    },
                ]),
        )
        .expect("assembly should produce deterministic local adaptation order");

    let keys = working_memory
        .present
        .self_state
        .facts
        .iter()
        .filter(|fact| {
            fact.key.starts_with("self_state:")
                || fact.key.starts_with("risk_boundary:")
                || fact.key.starts_with("private_t3:")
        })
        .map(|fact| fact.key.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        keys,
        vec![
            "self_state:alpha",
            "self_state:beta",
            "risk_boundary:alpha",
            "private_t3:zeta",
        ],
        "resolved local adaptations should merge in self_state, risk_boundary, private_t3 order with deterministic key ordering inside each lane"
    );
}

#[test]
fn assembler_preserves_action_candidate_summary_and_intent() {
    let path = fresh_db_path("action-candidate-summary-intent");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/action-candidate-summary-intent".to_string(),
            source_label: Some("action-candidate-summary-intent".to_string()),
            source_kind: None,
            content: "working memory candidate seed should survive assembly".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:15:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("candidate-summary ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("candidate seed").with_action_seed(ActionSeed::new(
                ActionCandidate::new(
                    ActionKind::Epistemic,
                    "inspect the retrieved evidence before acting",
                )
                .with_intent("reduce uncertainty before selecting an action"),
            )),
        )
        .expect("assembly should preserve action candidate summary and intent");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(
        working_memory.branches[0].candidate.summary,
        "inspect the retrieved evidence before acting"
    );
    assert_eq!(
        working_memory.branches[0].candidate.intent.as_deref(),
        Some("reduce uncertainty before selecting an action")
    );
}

#[test]
fn assembler_preserves_action_candidate_parameters_and_expected_effects() {
    let path = fresh_db_path("action-candidate-parameters-effects");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/action-candidate-parameters-effects".to_string(),
            source_label: Some("action-candidate-parameters-effects".to_string()),
            source_kind: None,
            content: "working memory candidate parameters should survive assembly".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:16:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("candidate-parameters ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("candidate parameters").with_action_seed(ActionSeed::new(
                ActionCandidate {
                    kind: ActionKind::Instrumental,
                    summary: "apply the selected action".to_string(),
                    intent: Some("execute once evidence is sufficient".to_string()),
                    parameters: vec![
                        "target=file:src/main.rs".to_string(),
                        "mode=safe".to_string(),
                    ],
                    expected_effects: vec![
                        "state advances".to_string(),
                        "citations remain intact".to_string(),
                    ],
                },
            )),
        )
        .expect("assembly should preserve action candidate parameters and effects");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(
        working_memory.branches[0].candidate.parameters,
        vec![
            "target=file:src/main.rs".to_string(),
            "mode=safe".to_string()
        ]
    );
    assert_eq!(
        working_memory.branches[0].candidate.expected_effects,
        vec![
            "state advances".to_string(),
            "citations remain intact".to_string()
        ]
    );
}

#[test]
fn assembler_preserves_default_branch_supporting_evidence_provenance() {
    let path = fresh_db_path("default-branch-support-provenance");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/default-branch-support-provenance".to_string(),
            source_label: Some("default-branch-support-provenance".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:45:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("default-branch-support provenance ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "inspect all supporting evidence",
                ))),
        )
        .expect("assembly should preserve default branch-support provenance");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(working_memory.branches[0].supporting_evidence.len(), 1);

    let fragment = &working_memory.branches[0].supporting_evidence[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.provenance.origin, "ingest");
    assert_eq!(
        fragment.provenance.imported_via.as_deref(),
        Some("ingest_service")
    );
    assert!(
        fragment
            .provenance
            .derived_from
            .iter()
            .any(|value| value.starts_with("memo://project/default-branch-support-provenance#")),
        "default branch-support path should preserve the source-derived provenance anchor"
    );
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && fragment
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "default branch-support provenance coverage should preserve both provenance branches"
    );
}

#[test]
fn assembler_clamps_zero_limit_to_one_world_fragment() {
    let path = fresh_db_path("assembly-zero-limit-clamp");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/assembly-zero-limit-first".to_string(),
            source_label: Some("assembly-zero-limit-first".to_string()),
            source_kind: None,
            content: "zero limit should still recall the strongest cited fragment".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("first ingest should succeed");
    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/assembly-zero-limit-second".to_string(),
            source_label: Some("assembly-zero-limit-second".to_string()),
            source_kind: None,
            content: "zero limit should not collapse assembly to an empty frame".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T09:59:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("second ingest should succeed");

    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let working_memory = assembler
        .assemble(&WorkingMemoryRequest::new("zero limit").with_limit(0))
        .expect("assembly should clamp zero limit to one fragment");

    assert_eq!(
        working_memory.present.world_fragments.len(),
        1,
        "working-memory assembly should clamp limit=0 to one recalled world fragment"
    );
}

#[test]
fn working_memory_request_bounded_limit_matches_search_recall_ceiling() {
    let request = WorkingMemoryRequest::new("bounded limit").with_limit(999);
    assert_eq!(
        request.bounded_limit(),
        agent_memos::search::lexical::MAX_RECALL_LIMIT,
        "working-memory request limit helper should clamp to the same recall ceiling as search"
    );
}

#[test]
fn working_memory_request_new_starts_with_default_empty_runtime_state() {
    let request = WorkingMemoryRequest::new("default request");
    assert_eq!(request.limit, WorkingMemoryRequest::DEFAULT_LIMIT);
    assert_eq!(
        request.filters,
        agent_memos::search::SearchFilters::default()
    );
    assert!(request.subject_ref.is_none());
    assert!(request.task_context.is_none());
    assert!(request.active_goal.is_none());
    assert!(request.active_risks.is_empty());
    assert!(request.metacog_flags.is_empty());
    assert!(request.capability_flags.is_empty());
    assert!(request.readiness_flags.is_empty());
    assert!(request.action_seeds.is_empty());
    assert!(request.local_adaptation_entries.is_empty());
    assert!(request.integrated_results.is_empty());
}

#[test]
fn working_memory_request_bounded_limit_clamps_zero_to_one() {
    let request = WorkingMemoryRequest::new("bounded limit").with_limit(0);
    assert_eq!(
        request.bounded_limit(),
        1,
        "working-memory request limit helper should clamp zero to one"
    );
}

#[test]
fn working_memory_request_selected_truth_facts_project_record_ids_and_layers() {
    let path = fresh_db_path("selected-truth-facts");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let repo = MemoryRepository::new(db.conn());

    let first = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/selected-truth-facts-first".to_string(),
            source_label: Some("selected-truth-facts-first".to_string()),
            source_kind: None,
            content: "selected truth facts should project id and truth layer".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:14:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("first ingest should succeed");
    let second = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/selected-truth-facts-second".to_string(),
            source_label: Some("selected-truth-facts-second".to_string()),
            source_kind: None,
            content: "selected truth facts should preserve caller truth ordering".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T1,
            recorded_at: "2026-04-16T10:14:30Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("second ingest should succeed");

    let first_truth = repo
        .get_truth_record(&first.record_ids[0])
        .expect("truth should load")
        .expect("first truth should exist");
    let second_truth = repo
        .get_truth_record(&second.record_ids[0])
        .expect("truth should load")
        .expect("second truth should exist");

    let facts = WorkingMemoryRequest::new("selected truth facts")
        .selected_truth_facts(&[first_truth, second_truth]);

    assert_eq!(
        facts,
        vec![
            SelfStateFact {
                key: format!("truth_record:{}", first.record_ids[0]),
                value: "t2".to_string(),
                source_record_id: Some(first.record_ids[0].clone()),
            },
            SelfStateFact {
                key: format!("truth_record:{}", second.record_ids[0]),
                value: "t1".to_string(),
                source_record_id: Some(second.record_ids[0].clone()),
            }
        ]
    );
}

#[test]
fn minimal_self_state_provider_preserves_request_control_fields_and_truth_facts() {
    let path = fresh_db_path("minimal-self-state-provider");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let repo = MemoryRepository::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/minimal-self-state-provider".to_string(),
            source_label: Some("minimal-self-state-provider".to_string()),
            source_kind: None,
            content: "minimal self-state provider should preserve request fields".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:15:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");

    let truth = repo
        .get_truth_record(&record.record_ids[0])
        .expect("truth should load")
        .expect("truth should exist");
    let request = WorkingMemoryRequest::new("minimal self state")
        .with_task_context("keep current control state visible")
        .with_capability_flag("lexical_search_ready")
        .with_readiness_flag("truth_governance_ready");

    let snapshot = MinimalSelfStateProvider.snapshot(&request, &[truth]);

    assert_eq!(
        snapshot.task_context.as_deref(),
        Some("keep current control state visible")
    );
    assert_eq!(
        snapshot.capability_flags,
        vec!["lexical_search_ready".to_string()]
    );
    assert_eq!(
        snapshot.readiness_flags,
        vec!["truth_governance_ready".to_string()]
    );
    assert_eq!(
        snapshot.facts,
        vec![SelfStateFact {
            key: format!("truth_record:{}", record.record_ids[0]),
            value: "t2".to_string(),
            source_record_id: Some(record.record_ids[0].clone()),
        }]
    );
}

#[test]
fn minimal_self_state_provider_projects_explicit_stable_and_runtime_self_model() {
    let path = fresh_db_path("minimal-self-model-projection");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let repo = MemoryRepository::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/minimal-self-model-projection".to_string(),
            source_label: Some("minimal-self-model-projection".to_string()),
            source_kind: None,
            content: "self model should separate stable knowledge from runtime state".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:15:30Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");

    let truth = repo
        .get_truth_record(&record.record_ids[0])
        .expect("truth should load")
        .expect("truth should exist");
    let request = WorkingMemoryRequest::new("minimal self model")
        .with_task_context("separate runtime state")
        .with_capability_flag("lexical_search_ready")
        .with_readiness_flag("truth_governance_ready");

    let model = MinimalSelfStateProvider.project(&request, &[truth]);

    assert_eq!(
        model.stable.capability_flags,
        vec!["lexical_search_ready".to_string()]
    );
    assert_eq!(
        model.runtime.task_context.as_deref(),
        Some("separate runtime state")
    );
    assert_eq!(
        model.runtime.readiness_flags,
        vec!["truth_governance_ready".to_string()]
    );
    assert_eq!(model.runtime.facts, Vec::<SelfStateFact>::new());
    assert_eq!(
        model.stable.facts,
        vec![SelfStateFact {
            key: format!("truth_record:{}", record.record_ids[0]),
            value: "t2".to_string(),
            source_record_id: Some(record.record_ids[0].clone()),
        }]
    );
    assert_eq!(
        model.project_snapshot(),
        SelfStateSnapshot {
            task_context: Some("separate runtime state".to_string()),
            capability_flags: vec!["lexical_search_ready".to_string()],
            readiness_flags: vec!["truth_governance_ready".to_string()],
            facts: vec![SelfStateFact {
                key: format!("truth_record:{}", record.record_ids[0]),
                value: "t2".to_string(),
                source_record_id: Some(record.record_ids[0].clone()),
            }],
        }
    );
}

#[test]
fn adaptive_self_state_provider_routes_local_overlays_through_runtime_self_model_facts() {
    let request =
        WorkingMemoryRequest::new("runtime overlays").with_local_adaptation_entries(vec![
            LocalAdaptationEntry {
                entry_id: "request-local-adaptation:self-state:preferred_mode".to_string(),
                subject_ref: "subject://agent/demo".to_string(),
                target_kind: LocalAdaptationTargetKind::SelfState,
                key: "preferred_mode".to_string(),
                payload: LocalAdaptationPayload {
                    value: json!("conservative"),
                    trigger_kind: "request_override".to_string(),
                    evidence_refs: vec!["memo://project/request-local-adaptation".to_string()],
                },
                source_queue_item_id: None,
                created_at: "2026-04-16T10:14:31Z".to_string(),
                updated_at: "2026-04-16T10:14:31Z".to_string(),
            },
            LocalAdaptationEntry {
                entry_id: "request-local-adaptation:risk-boundary:deploy".to_string(),
                subject_ref: "subject://agent/demo".to_string(),
                target_kind: LocalAdaptationTargetKind::RiskBoundary,
                key: "deploy".to_string(),
                payload: LocalAdaptationPayload {
                    value: json!("requires_human_review"),
                    trigger_kind: "request_override".to_string(),
                    evidence_refs: vec!["memo://project/request-local-adaptation".to_string()],
                },
                source_queue_item_id: None,
                created_at: "2026-04-16T10:14:32Z".to_string(),
                updated_at: "2026-04-16T10:14:32Z".to_string(),
            },
        ]);

    let model = AdaptiveSelfStateProvider::new(MinimalSelfStateProvider).project(&request, &[]);

    assert_eq!(
        model.runtime.facts,
        vec![
            SelfStateFact {
                key: "self_state:preferred_mode".to_string(),
                value: "conservative".to_string(),
                source_record_id: None,
            },
            SelfStateFact {
                key: "risk_boundary:deploy".to_string(),
                value: "requires_human_review".to_string(),
                source_record_id: None,
            },
        ]
    );
    assert_eq!(
        model.project_snapshot().facts,
        vec![
            SelfStateFact {
                key: "self_state:preferred_mode".to_string(),
                value: "conservative".to_string(),
                source_record_id: None,
            },
            SelfStateFact {
                key: "risk_boundary:deploy".to_string(),
                value: "requires_human_review".to_string(),
                source_record_id: None,
            },
        ]
    );
}

#[test]
fn assembler_clamps_excessive_limit_to_search_recall_ceiling() {
    let path = fresh_db_path("assembly-excessive-limit-clamp");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    for index in 0..30 {
        ingest
            .ingest(IngestRequest {
                source_uri: format!("memo://project/assembly-limit-{index}"),
                source_label: Some(format!("assembly-limit-{index}")),
                source_kind: None,
                content: "excessive assembly limit should still respect bounded lexical recall"
                    .to_string(),
                scope: Scope::Project,
                record_type: RecordType::Observation,
                truth_layer: TruthLayer::T2,
                recorded_at: format!("2026-04-16T10:{index:02}:00Z"),
                valid_from: None,
                valid_to: None,
            })
            .expect("bulk ingest should succeed");
    }

    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let working_memory = assembler
        .assemble(&WorkingMemoryRequest::new("bounded lexical recall").with_limit(999))
        .expect("assembly should clamp excessive limit to search recall ceiling");

    assert_eq!(
        working_memory.present.world_fragments.len(),
        agent_memos::search::lexical::MAX_RECALL_LIMIT,
        "working-memory assembly should not exceed the search recall ceiling when limit is excessively large"
    );
}

#[test]
fn assembler_returns_empty_world_fragments_for_whitespace_query() {
    let path = fresh_db_path("assembly-whitespace-query");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/assembly-whitespace-query".to_string(),
            source_label: Some("assembly-whitespace-query".to_string()),
            source_kind: None,
            content: "blank assembly queries should not produce misleading recall".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:31:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");

    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("   ")
                .with_active_goal("preserve control state on blank recall")
                .with_active_risk("blank query")
                .with_metacog_flag(MetacognitiveFlag {
                    code: "blank_query".to_string(),
                    detail: Some("no recall should occur".to_string()),
                }),
        )
        .expect("assembly should succeed for whitespace-only query");

    assert!(
        working_memory.present.world_fragments.is_empty(),
        "whitespace-only assembly query should produce an empty recalled world set"
    );
    assert_eq!(
        working_memory
            .present
            .active_goal
            .as_ref()
            .map(|goal| goal.summary.as_str()),
        Some("preserve control state on blank recall")
    );
    assert_eq!(working_memory.present.active_risks, vec!["blank query"]);
    assert_eq!(working_memory.present.metacog_flags.len(), 1);
}

#[test]
fn assembler_materializes_action_seed_with_empty_support_on_whitespace_query() {
    let path = fresh_db_path("whitespace-query-empty-branch-support");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/whitespace-query-empty-branch-support".to_string(),
            source_label: Some("whitespace-query-empty-branch-support".to_string()),
            source_kind: None,
            content: "blank queries should not prevent branch materialization".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:32:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");

    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("   ").with_action_seed(ActionSeed::new(
                ActionCandidate::new(ActionKind::Epistemic, "ask for more evidence"),
            )),
        )
        .expect("assembly should still materialize action seeds on blank query");

    assert_eq!(working_memory.branches.len(), 1);
    assert!(
        working_memory.branches[0].supporting_evidence.is_empty(),
        "blank recall should materialize the branch with empty supporting evidence rather than failing"
    );
}

#[test]
fn assembler_preserves_branch_identity_on_whitespace_query() {
    let path = fresh_db_path("whitespace-query-branch-identity");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/whitespace-query-branch-identity".to_string(),
            source_label: Some("whitespace-query-branch-identity".to_string()),
            source_kind: None,
            content: "blank queries should preserve branch identity even without evidence"
                .to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:33:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");

    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("   ").with_action_seed(
                ActionSeed::new(ActionCandidate::new(
                    ActionKind::Regulative,
                    "pause and request clarification",
                ))
                .with_risk_marker("blank_query"),
            ),
        )
        .expect("assembly should preserve branch identity on blank query");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(
        working_memory.branches[0].candidate.kind,
        ActionKind::Regulative
    );
    assert_eq!(
        working_memory.branches[0].candidate.summary,
        "pause and request clarification"
    );
    assert_eq!(
        working_memory.branches[0].risk_markers,
        vec!["blank_query".to_string()]
    );
}

#[test]
fn assembler_preserves_action_candidate_fields_on_whitespace_query() {
    let path = fresh_db_path("whitespace-query-candidate-fields");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/whitespace-query-candidate-fields".to_string(),
            source_label: Some("whitespace-query-candidate-fields".to_string()),
            source_kind: None,
            content: "blank queries should preserve candidate fields even without evidence"
                .to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:34:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");

    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("   ").with_action_seed(ActionSeed::new(ActionCandidate {
                kind: ActionKind::Instrumental,
                summary: "apply the selected action".to_string(),
                intent: Some("execute once evidence is sufficient".to_string()),
                parameters: vec![
                    "target=file:src/main.rs".to_string(),
                    "mode=safe".to_string(),
                ],
                expected_effects: vec![
                    "state advances".to_string(),
                    "citations remain intact".to_string(),
                ],
            })),
        )
        .expect("assembly should preserve candidate fields on blank query");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(
        working_memory.branches[0].candidate.kind,
        ActionKind::Instrumental
    );
    assert_eq!(
        working_memory.branches[0].candidate.intent.as_deref(),
        Some("execute once evidence is sufficient")
    );
    assert_eq!(
        working_memory.branches[0].candidate.parameters,
        vec![
            "target=file:src/main.rs".to_string(),
            "mode=safe".to_string()
        ]
    );
    assert_eq!(
        working_memory.branches[0].candidate.expected_effects,
        vec![
            "state advances".to_string(),
            "citations remain intact".to_string()
        ]
    );
}

#[test]
fn assembler_preserves_default_branch_supporting_evidence_score() {
    let path = fresh_db_path("default-branch-support-score");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/default-branch-support-score".to_string(),
            source_label: Some("default-branch-support-score".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:46:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("default-branch-support score ingest should succeed");

    let expected_score = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("search should succeed")
        .results[0]
        .score
        .clone();

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "inspect all supporting evidence",
                ))),
        )
        .expect("assembly should preserve default branch-support score");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(working_memory.branches[0].supporting_evidence.len(), 1);
    assert_eq!(
        working_memory.branches[0].supporting_evidence[0].score,
        expected_score
    );
}

#[test]
fn assembler_errors_when_branch_support_references_missing_record() {
    let path = fresh_db_path("missing-branch-support-record");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/missing-branch-support-record".to_string(),
            source_label: Some("missing-branch-support-record".to_string()),
            source_kind: None,
            content: "branch support ids should fail closed when evidence is missing".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:47:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");

    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let error = assembler
        .assemble(
            &WorkingMemoryRequest::new("branch support missing")
                .with_limit(1)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Instrumental,
                        "attempt unsupported action",
                    ))
                    .with_supporting_record_ids(vec!["mem-missing-support".to_string()]),
                ),
        )
        .expect_err("assembly should fail when branch support references a missing record");

    assert!(
        matches!(
            error,
            agent_memos::cognition::assembly::WorkingMemoryAssemblyError::MissingSupportingRecord { ref record_id }
            if record_id == "mem-missing-support"
        ),
        "assembly should surface the missing supporting record id, got: {error:?}"
    );
}

#[test]
fn assembler_preserves_explicit_supporting_record_id_order() {
    let path = fresh_db_path("explicit-support-order");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let first = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/explicit-support-order-first".to_string(),
            source_label: Some("explicit-support-order-first".to_string()),
            source_kind: None,
            content: "first explicit support record".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:47:10Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("first ingest should succeed");
    let second = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/explicit-support-order-second".to_string(),
            source_label: Some("explicit-support-order-second".to_string()),
            source_kind: None,
            content: "second explicit support record".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:47:20Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("second ingest should succeed");

    let first_id = first.record_ids[0].clone();
    let second_id = second.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("explicit support record")
                .with_limit(2)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "review supporting evidence in declared order",
                    ))
                    .with_supporting_record_ids(vec![second_id.clone(), first_id.clone()]),
                ),
        )
        .expect("assembly should preserve explicit support ordering");

    assert_eq!(
        working_memory.branches[0]
            .supporting_evidence
            .iter()
            .map(|fragment| fragment.record_id.as_str())
            .collect::<Vec<_>>(),
        vec![second_id.as_str(), first_id.as_str()],
        "explicit supporting_record_ids should preserve caller-declared order in branch evidence"
    );
}

#[test]
fn assembler_errors_when_integrated_result_has_no_truth_projection() {
    let path = fresh_db_path("missing-truth-projection");
    let db = Database::open(&path).expect("database should open");
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);

    let error = assembler
        .assemble(
            &WorkingMemoryRequest::new("missing truth projection").with_integrated_results(vec![
                synthetic_result(
                    "mem-missing-truth",
                    "memo://project/missing-truth",
                    "missing truth projection",
                    "integrated results should fail closed when truth projection is absent",
                ),
            ]),
        )
        .expect_err("assembly should fail when integrated results lack a truth projection");

    assert!(
        matches!(
            error,
            agent_memos::cognition::assembly::WorkingMemoryAssemblyError::MissingTruthProjection { ref record_id }
            if record_id == "mem-missing-truth"
        ),
        "assembly should surface the missing truth projection record id, got: {error:?}"
    );
}

#[test]
fn assembler_preserves_default_branch_supporting_evidence_truth_context() {
    let path = fresh_db_path("default-branch-support-truth-context");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let governance = TruthGovernanceService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/default-branch-support-truth-context".to_string(),
            source_label: Some("default-branch-support-truth-context".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T3,
            recorded_at: "2026-04-16T10:47:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("default-branch-support truth-context ingest should succeed");

    let review = governance
        .create_promotion_review(CreatePromotionReviewRequest {
            review_id: "review-default-branch-support".to_string(),
            source_record_id: record.record_ids[0].clone(),
            created_at: "2026-04-16T10:47:30Z".to_string(),
            review_notes: Some(json!({
                "summary": "default branch-support review"
            })),
        })
        .expect("default branch-support review should create")
        .review;

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "inspect all supporting evidence",
                ))),
        )
        .expect("assembly should preserve default branch-support truth context");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(working_memory.branches[0].supporting_evidence.len(), 1);

    let fragment = &working_memory.branches[0].supporting_evidence[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.truth_context.truth_layer, TruthLayer::T3);
    assert_eq!(
        fragment.truth_context.open_review_ids,
        vec![review.review_id]
    );
    let t3_state = fragment
        .truth_context
        .t3_state
        .as_ref()
        .expect("default branch-support path should preserve t3 state");
    assert_eq!(
        t3_state.last_reviewed_at.as_deref(),
        Some("2026-04-16T10:47:30Z")
    );
}

#[test]
fn assembler_preserves_default_branch_supporting_evidence_dsl_and_snippet() {
    let path = fresh_db_path("default-branch-support-dsl");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/default-branch-support-dsl".to_string(),
            source_label: Some("default-branch-support-dsl".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:48:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("default-branch-support dsl ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "inspect all supporting evidence",
                ))),
        )
        .expect("assembly should preserve default branch-support dsl");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(working_memory.branches[0].supporting_evidence.len(), 1);

    let fragment = &working_memory.branches[0].supporting_evidence[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    let dsl = fragment
        .dsl
        .as_ref()
        .expect("default branch-support path should preserve dsl");
    assert_eq!(dsl.domain, "project");
    assert_eq!(dsl.kind, "decision");
    assert_eq!(dsl.source_ref, "memo://project/default-branch-support-dsl");
    assert!(
        fragment.snippet.contains("WHY:"),
        "default branch-support path should preserve the structured snippet surface: {:?}",
        fragment.snippet
    );
}

#[test]
fn assembler_preserves_default_branch_supporting_evidence_citation_source_uri() {
    let path = fresh_db_path("default-branch-support-citation-source-uri");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/default-branch-support-citation-source-uri".to_string(),
            source_label: Some("default-branch-support-citation-source-uri".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:49:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("default-branch-support citation-source-uri ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "inspect all supporting evidence",
                ))),
        )
        .expect("assembly should preserve default branch-support citation source uri");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(working_memory.branches[0].supporting_evidence.len(), 1);

    let fragment = &working_memory.branches[0].supporting_evidence[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(
        fragment.citation.source_uri,
        "memo://project/default-branch-support-citation-source-uri"
    );
    assert_eq!(fragment.citation.record_id, record.record_ids[0]);
}

#[test]
fn assembler_preserves_default_branch_supporting_evidence_citation_source_kind() {
    let path = fresh_db_path("default-branch-support-citation-source-kind");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/default-branch-support-citation-source-kind".to_string(),
            source_label: Some("default-branch-support-citation-source-kind".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:49:30Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("default-branch-support citation-source-kind ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "inspect all supporting evidence",
                ))),
        )
        .expect("assembly should preserve default branch-support citation source kind");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(working_memory.branches[0].supporting_evidence.len(), 1);
    assert_eq!(
        working_memory.branches[0].supporting_evidence[0]
            .citation
            .source_kind,
        SourceKind::Note
    );
}

#[test]
fn assembler_preserves_default_branch_supporting_evidence_citation_source_label() {
    let path = fresh_db_path("default-branch-support-citation-source-label");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/default-branch-support-citation-source-label".to_string(),
            source_label: Some("default-branch-support-citation-source-label".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:49:45Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("default-branch-support citation-source-label ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "inspect all supporting evidence",
                ))),
        )
        .expect("assembly should preserve default branch-support citation source label");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(working_memory.branches[0].supporting_evidence.len(), 1);
    assert_eq!(
        working_memory.branches[0].supporting_evidence[0]
            .citation
            .source_label
            .as_deref(),
        Some("default-branch-support-citation-source-label")
    );
}

#[test]
fn assembler_preserves_default_branch_supporting_evidence_citation_validity() {
    let path = fresh_db_path("default-branch-support-citation-validity");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/default-branch-support-citation-validity".to_string(),
            source_label: Some("default-branch-support-citation-validity".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:49:50Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("default-branch-support citation-validity ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "inspect all supporting evidence",
                ))),
        )
        .expect("assembly should preserve default branch-support citation validity");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(working_memory.branches[0].supporting_evidence.len(), 1);
    assert_eq!(
        working_memory.branches[0].supporting_evidence[0]
            .citation
            .validity
            .valid_from
            .as_deref(),
        Some("2026-04-10T00:00:00Z")
    );
    assert_eq!(
        working_memory.branches[0].supporting_evidence[0]
            .citation
            .validity
            .valid_to
            .as_deref(),
        Some("2026-04-20T00:00:00Z")
    );
}

#[test]
fn assembler_preserves_default_branch_supporting_evidence_citation_recorded_at() {
    let path = fresh_db_path("default-branch-support-citation-recorded-at");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/default-branch-support-citation-recorded-at".to_string(),
            source_label: Some("default-branch-support-citation-recorded-at".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:49:55Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("default-branch-support citation-recorded-at ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "inspect all supporting evidence",
                ))),
        )
        .expect("assembly should preserve default branch-support citation recorded_at");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(working_memory.branches[0].supporting_evidence.len(), 1);
    assert_eq!(
        working_memory.branches[0].supporting_evidence[0]
            .citation
            .recorded_at,
        "2026-04-16T10:49:55Z"
    );
}

#[test]
fn assembler_preserves_default_branch_supporting_evidence_citation_anchor() {
    let path = fresh_db_path("default-branch-support-citation-anchor");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/default-branch-support-citation-anchor".to_string(),
            source_label: Some("default-branch-support-citation-anchor".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:49:58Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("default-branch-support citation-anchor ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "inspect all supporting evidence",
                ))),
        )
        .expect("assembly should preserve default branch-support citation anchor");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(working_memory.branches[0].supporting_evidence.len(), 1);
    assert_eq!(
        working_memory.branches[0].supporting_evidence[0]
            .citation
            .anchor
            .chunk_index,
        0
    );
    assert_eq!(
        working_memory.branches[0].supporting_evidence[0]
            .citation
            .anchor
            .chunk_count,
        1
    );
    assert!(matches!(
        working_memory.branches[0].supporting_evidence[0]
            .citation
            .anchor
            .anchor,
        ChunkAnchor::LineRange {
            start_line: 1,
            end_line: 1
        }
    ));
}

#[test]
fn assembler_preserves_default_branch_supporting_evidence_filter_trace() {
    let path = fresh_db_path("default-branch-support-filter-trace");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/default-branch-support-filter-trace".to_string(),
            source_label: Some("default-branch-support-filter-trace".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:50:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("default-branch-support filter-trace ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_filters(agent_memos::search::SearchFilters {
                    scope: Some(Scope::Project),
                    record_type: Some(RecordType::Decision),
                    truth_layer: Some(TruthLayer::T2),
                    ..Default::default()
                })
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "inspect all supporting evidence",
                ))),
        )
        .expect("assembly should preserve default branch-support filter trace");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(working_memory.branches[0].supporting_evidence.len(), 1);
    let fragment = &working_memory.branches[0].supporting_evidence[0];
    assert_eq!(fragment.trace.applied_filters.scope, Some(Scope::Project));
    assert_eq!(
        fragment.trace.applied_filters.record_type,
        Some(RecordType::Decision)
    );
    assert_eq!(
        fragment.trace.applied_filters.truth_layer,
        Some(TruthLayer::T2)
    );
    assert_eq!(fragment.trace.matched_query, "decision");
}

#[test]
fn assembler_preserves_default_branch_supporting_evidence_temporal_filter_trace() {
    let path = fresh_db_path("default-branch-support-temporal-filter-trace");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/default-branch-support-temporal-filter-trace".to_string(),
            source_label: Some("default-branch-support-temporal-filter-trace".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:50:30Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("default-branch-support temporal-filter-trace ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_filters(agent_memos::search::SearchFilters {
                    valid_at: Some("2026-04-16T12:00:00Z".to_string()),
                    recorded_from: Some("2026-04-16T00:00:00Z".to_string()),
                    recorded_to: Some("2026-04-17T00:00:00Z".to_string()),
                    ..Default::default()
                })
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "inspect all supporting evidence",
                ))),
        )
        .expect("assembly should preserve default branch-support temporal filter trace");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(working_memory.branches[0].supporting_evidence.len(), 1);
    let fragment = &working_memory.branches[0].supporting_evidence[0];
    assert_eq!(
        fragment.trace.applied_filters.valid_at.as_deref(),
        Some("2026-04-16T12:00:00Z")
    );
    assert_eq!(
        fragment.trace.applied_filters.recorded_from.as_deref(),
        Some("2026-04-16T00:00:00Z")
    );
    assert_eq!(
        fragment.trace.applied_filters.recorded_to.as_deref(),
        Some("2026-04-17T00:00:00Z")
    );
    assert_eq!(fragment.trace.matched_query, "decision");
}

#[test]
fn assembler_preserves_default_branch_supporting_evidence_trace_summary() {
    let path = fresh_db_path("default-branch-support-trace-summary");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/default-branch-support-trace-summary".to_string(),
            source_label: Some("default-branch-support-trace-summary".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T10:50:30Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("default-branch-support trace-summary ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "inspect all supporting evidence",
                ))),
        )
        .expect("assembly should preserve default branch-support trace summary");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(working_memory.branches[0].supporting_evidence.len(), 1);
    let fragment = &working_memory.branches[0].supporting_evidence[0];
    assert_eq!(
        fragment.trace.channel_contribution,
        ChannelContribution::LexicalOnly
    );
    assert_eq!(fragment.trace.matched_query, "lexical-first baseline");
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && fragment
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "default branch-support trace summary should preserve both lexical and structured provenance"
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
fn assembler_preserves_integrated_follow_up_provenance_on_fragments() {
    let path = fresh_db_path("follow-up-provenance-world-fragments");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary-provenance".to_string(),
            source_label: Some("primary-provenance".to_string()),
            source_kind: None,
            content: "primary query result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:10:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up-provenance".to_string(),
            source_label: Some("follow-up-provenance".to_string()),
            source_kind: None,
            content: "follow-up provenance result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:11:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("follow-up ingest should succeed");

    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary query result").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let follow_up_result = search
        .search(&SearchRequest::new("follow-up provenance result").with_limit(1))
        .expect("follow-up search should succeed")
        .results;
    let mut integrated_results = primary_result;
    integrated_results.extend(follow_up_result);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary query result")
                .with_limit(1)
                .with_integrated_results(integrated_results),
        )
        .expect("assembly should preserve integrated follow-up provenance");

    assert_eq!(working_memory.present.world_fragments.len(), 2);
    let primary_fragment = working_memory
        .present
        .world_fragments
        .iter()
        .find(|fragment| fragment.record_id == primary.record_ids[0])
        .expect("primary fragment should exist");
    assert_eq!(primary_fragment.provenance.origin, "ingest");

    let follow_up_fragment = working_memory
        .present
        .world_fragments
        .iter()
        .find(|fragment| fragment.record_id == follow_up.record_ids[0])
        .expect("follow-up fragment should exist");
    assert_eq!(follow_up_fragment.provenance.origin, "ingest");
    assert_eq!(
        follow_up_fragment.provenance.imported_via.as_deref(),
        Some("ingest_service")
    );
    assert!(
        follow_up_fragment
            .provenance
            .derived_from
            .iter()
            .any(|value| value.starts_with("memo://project/follow-up-provenance#"))
    );
    assert_eq!(
        follow_up_fragment.trace.matched_query,
        "follow-up provenance result"
    );
}

#[test]
fn assembler_hydrates_missing_dsl_for_integrated_follow_up_world_fragments() {
    let path = fresh_db_path("follow-up-world-fragments-hydrate-dsl");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary-follow-up-hydrate-dsl".to_string(),
            source_label: Some("primary-follow-up-hydrate-dsl".to_string()),
            source_kind: None,
            content: "primary hydrate dsl query".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:15:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up-world-hydrate-dsl".to_string(),
            source_label: Some("follow-up-world-hydrate-dsl".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:16:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("follow-up ingest should succeed");

    let follow_up_id = follow_up.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary hydrate dsl").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let mut follow_up_result = search
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("follow-up search should succeed")
        .results;
    follow_up_result[0].snippet = "caller-provided follow-up snippet".to_string();
    follow_up_result[0].dsl = None;
    let mut integrated_results = primary_result;
    integrated_results.extend(follow_up_result);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary hydrate dsl")
                .with_limit(1)
                .with_integrated_results(integrated_results),
        )
        .expect("assembly should hydrate missing dsl for integrated follow-up world fragments");

    let follow_up_fragment = working_memory
        .present
        .world_fragments
        .iter()
        .find(|fragment| fragment.record_id == follow_up_id)
        .expect("follow-up fragment should exist");

    assert_eq!(
        follow_up_fragment.snippet, "caller-provided follow-up snippet",
        "dsl hydration should preserve the caller-provided follow-up snippet"
    );
    let dsl = follow_up_fragment
        .dsl
        .as_ref()
        .expect("follow-up world fragment should recover its repository-backed dsl");
    assert_eq!(dsl.domain, "project");
    assert_eq!(dsl.kind, "decision");
    assert_eq!(dsl.source_ref, "memo://project/follow-up-world-hydrate-dsl");
}

#[test]
fn assembler_filters_integrated_results_by_taxonomy_filters() {
    let path = fresh_db_path("integrated-results-taxonomy-filter");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let retrieval = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/integrated-taxonomy-retrieval".to_string(),
            source_label: Some("integrated-taxonomy-retrieval".to_string()),
            source_kind: None,
            content: "retrieval baseline keeps lexical search explainable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:12:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("retrieval ingest should succeed");
    let config = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/integrated-taxonomy-config".to_string(),
            source_label: Some("integrated-taxonomy-config".to_string()),
            source_kind: None,
            content: "config baseline keeps toml setting review stable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:13:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("config ingest should succeed");

    let retrieval_id = retrieval.record_ids[0].clone();
    let config_id = config.record_ids[0].clone();
    let search = SearchService::new(db.conn());
    let integrated_results = search
        .search(&SearchRequest::new("baseline").with_limit(2))
        .expect("baseline search should succeed")
        .results;

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("baseline")
                .with_limit(1)
                .with_filters(agent_memos::search::SearchFilters {
                    topic: Some("retrieval".to_string()),
                    ..Default::default()
                })
                .with_integrated_results(integrated_results),
        )
        .expect("assembly should filter caller-provided integrated results by taxonomy");

    let record_ids = working_memory
        .present
        .world_fragments
        .iter()
        .map(|fragment| fragment.record_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(record_ids, vec![retrieval_id.as_str()]);
    assert!(
        !record_ids.contains(&config_id.as_str()),
        "taxonomy filters should exclude non-matching caller-provided integrated results"
    );
    assert_eq!(
        working_memory.present.world_fragments[0]
            .dsl
            .as_ref()
            .expect("filtered fragment should retain dsl payload")
            .topic,
        "retrieval"
    );
}

#[test]
fn assembler_rejects_filtered_integrated_supporting_record_ids() {
    let path = fresh_db_path("integrated-results-taxonomy-filter-supporting-record");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _retrieval = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/integrated-supporting-retrieval".to_string(),
            source_label: Some("integrated-supporting-retrieval".to_string()),
            source_kind: None,
            content: "retrieval baseline keeps lexical search explainable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:14:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("retrieval ingest should succeed");
    let config = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/integrated-supporting-config".to_string(),
            source_label: Some("integrated-supporting-config".to_string()),
            source_kind: None,
            content: "config baseline keeps toml setting review stable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:15:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("config ingest should succeed");

    let config_id = config.record_ids[0].clone();
    let search = SearchService::new(db.conn());
    let integrated_results = search
        .search(&SearchRequest::new("baseline").with_limit(2))
        .expect("baseline search should succeed")
        .results;

    let err = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("baseline")
                .with_limit(1)
                .with_filters(agent_memos::search::SearchFilters {
                    topic: Some("retrieval".to_string()),
                    ..Default::default()
                })
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "inspect filtered support",
                    ))
                    .with_supporting_record_ids(vec![config_id.clone()]),
                ),
        )
        .expect_err("assembly should reject explicit support ids filtered out by taxonomy");

    assert!(matches!(
        err,
        agent_memos::cognition::assembly::WorkingMemoryAssemblyError::MissingSupportingRecord {
            ref record_id
        } if record_id == &config_id
    ));
}

#[test]
fn assembler_filters_integrated_results_by_core_and_temporal_filters() {
    let path = fresh_db_path("integrated-results-core-temporal-filter");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let current = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/integrated-current-decision".to_string(),
            source_label: Some("integrated-current-decision".to_string()),
            source_kind: None,
            content: "decision baseline remains valid and recent".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:16:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("current ingest should succeed");
    let stale = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/integrated-stale-decision".to_string(),
            source_label: Some("integrated-stale-decision".to_string()),
            source_kind: None,
            content: "decision baseline is stale and should be filtered out".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T11:16:00Z".to_string(),
            valid_from: Some("2026-03-01T00:00:00Z".to_string()),
            valid_to: Some("2026-04-05T00:00:00Z".to_string()),
        })
        .expect("stale ingest should succeed");

    let current_id = current.record_ids[0].clone();
    let stale_id = stale.record_ids[0].clone();
    let search = SearchService::new(db.conn());
    let integrated_results = search
        .search(&SearchRequest::new("decision baseline").with_limit(2))
        .expect("baseline search should succeed")
        .results;

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision baseline")
                .with_limit(1)
                .with_filters(agent_memos::search::SearchFilters {
                    scope: Some(Scope::Project),
                    record_type: Some(RecordType::Decision),
                    truth_layer: Some(TruthLayer::T2),
                    valid_at: Some("2026-04-16T12:00:00Z".to_string()),
                    recorded_from: Some("2026-04-10T00:00:00Z".to_string()),
                    recorded_to: Some("2026-04-17T00:00:00Z".to_string()),
                    ..Default::default()
                })
                .with_integrated_results(integrated_results),
        )
        .expect("assembly should filter caller-provided integrated results by core and temporal filters");

    let record_ids = working_memory
        .present
        .world_fragments
        .iter()
        .map(|fragment| fragment.record_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(record_ids, vec![current_id.as_str()]);
    assert!(
        !record_ids.contains(&stale_id.as_str()),
        "core and temporal filters should exclude stale caller-provided integrated results"
    );
}

#[test]
fn assembler_rejects_filtered_integrated_supporting_record_ids_for_core_and_temporal_filters() {
    let path = fresh_db_path("integrated-results-core-temporal-filter-supporting-record");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _current = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/integrated-support-current".to_string(),
            source_label: Some("integrated-support-current".to_string()),
            source_kind: None,
            content: "decision baseline remains valid and recent".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:17:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("current ingest should succeed");
    let stale = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/integrated-support-stale".to_string(),
            source_label: Some("integrated-support-stale".to_string()),
            source_kind: None,
            content: "decision baseline is stale and should be filtered out".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-01T11:17:00Z".to_string(),
            valid_from: Some("2026-03-01T00:00:00Z".to_string()),
            valid_to: Some("2026-04-05T00:00:00Z".to_string()),
        })
        .expect("stale ingest should succeed");

    let stale_id = stale.record_ids[0].clone();
    let search = SearchService::new(db.conn());
    let integrated_results = search
        .search(&SearchRequest::new("decision baseline").with_limit(2))
        .expect("baseline search should succeed")
        .results;

    let err = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision baseline")
                .with_limit(1)
                .with_filters(agent_memos::search::SearchFilters {
                    scope: Some(Scope::Project),
                    record_type: Some(RecordType::Decision),
                    truth_layer: Some(TruthLayer::T2),
                    valid_at: Some("2026-04-16T12:00:00Z".to_string()),
                    recorded_from: Some("2026-04-10T00:00:00Z".to_string()),
                    recorded_to: Some("2026-04-17T00:00:00Z".to_string()),
                    ..Default::default()
                })
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "inspect filtered stale support",
                    ))
                    .with_supporting_record_ids(vec![stale_id.clone()]),
                ),
        )
        .expect_err(
            "assembly should reject explicit support ids filtered out by core/temporal filters",
        );

    assert!(matches!(
        err,
        agent_memos::cognition::assembly::WorkingMemoryAssemblyError::MissingSupportingRecord {
            ref record_id
        } if record_id == &stale_id
    ));
}

#[test]
fn assembler_respects_taxonomy_filters_from_retrieval_request() {
    let path = fresh_db_path("taxonomy-filtered-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/retrieval".to_string(),
            source_label: Some("retrieval".to_string()),
            source_kind: None,
            content: "retrieval baseline keeps lexical search explainable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("retrieval ingest should succeed");
    ingest
        .ingest(IngestRequest {
            source_uri: "memo://system/runtime".to_string(),
            source_label: Some("runtime".to_string()),
            source_kind: None,
            content: "runtime architecture keeps storage integration inspectable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:05:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("config ingest should succeed");

    let assembled = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(&WorkingMemoryRequest::new("baseline").with_filters(
            agent_memos::search::SearchFilters {
                topic: Some("retrieval".to_string()),
                ..Default::default()
            },
        ))
        .expect("taxonomy-filtered assembly should succeed");

    assert_eq!(assembled.present.world_fragments.len(), 1);
    let fragment = &assembled.present.world_fragments[0];
    assert_eq!(fragment.citation.source_uri, "memo://project/retrieval");
    assert_eq!(
        fragment.dsl.as_ref().expect("dsl should be present").topic,
        "retrieval"
    );
}

fn seed_taxonomy_filtered_records(ingest: &IngestService<'_>) {
    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/retrieval".to_string(),
            source_label: Some("retrieval".to_string()),
            source_kind: None,
            content: "retrieval baseline keeps lexical search explainable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("retrieval ingest should succeed");
    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/config".to_string(),
            source_label: Some("config".to_string()),
            source_kind: None,
            content: "config baseline keeps toml setting review stable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:05:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("config ingest should succeed");
    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/observation".to_string(),
            source_label: Some("observation".to_string()),
            source_kind: None,
            content: "runtime status ready".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:10:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("observation ingest should succeed");
}

#[test]
fn assembler_respects_domain_filters_from_retrieval_request() {
    let path = fresh_db_path("domain-filtered-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    seed_taxonomy_filtered_records(&ingest);

    let assembled = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(&WorkingMemoryRequest::new("baseline").with_filters(
            agent_memos::search::SearchFilters {
                domain: Some("project".to_string()),
                ..Default::default()
            },
        ))
        .expect("domain-filtered assembly should succeed");

    assert_eq!(assembled.present.world_fragments.len(), 2);
    assert!(
        assembled
            .present
            .world_fragments
            .iter()
            .all(
                |fragment| fragment.dsl.as_ref().expect("dsl should be present").domain
                    == "project"
            ),
        "domain filter should keep only project-domain fragments"
    );
}

#[test]
fn assembler_respects_aspect_filters_from_retrieval_request() {
    let path = fresh_db_path("aspect-filtered-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    seed_taxonomy_filtered_records(&ingest);

    let assembled = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(&WorkingMemoryRequest::new("status ready").with_filters(
            agent_memos::search::SearchFilters {
                aspect: Some("state".to_string()),
                ..Default::default()
            },
        ))
        .expect("aspect-filtered assembly should succeed");

    assert_eq!(assembled.present.world_fragments.len(), 1);
    let fragment = &assembled.present.world_fragments[0];
    assert_eq!(fragment.citation.source_uri, "memo://project/observation");
    assert_eq!(
        fragment.dsl.as_ref().expect("dsl should be present").aspect,
        "state"
    );
}

#[test]
fn assembler_respects_kind_filters_from_retrieval_request() {
    let path = fresh_db_path("kind-filtered-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    seed_taxonomy_filtered_records(&ingest);

    let assembled = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(&WorkingMemoryRequest::new("status ready").with_filters(
            agent_memos::search::SearchFilters {
                kind: Some("observation".to_string()),
                ..Default::default()
            },
        ))
        .expect("kind-filtered assembly should succeed");

    assert_eq!(assembled.present.world_fragments.len(), 1);
    let fragment = &assembled.present.world_fragments[0];
    assert_eq!(fragment.citation.source_uri, "memo://project/observation");
    assert_eq!(
        fragment.dsl.as_ref().expect("dsl should be present").kind,
        "observation"
    );
}

#[test]
fn assembler_preserves_topic_and_kind_filters_in_fragment_trace() {
    let path = fresh_db_path("taxonomy-trace-topic-kind");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    seed_taxonomy_filtered_records(&ingest);

    let assembled = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(&WorkingMemoryRequest::new("baseline").with_filters(
            agent_memos::search::SearchFilters {
                topic: Some("retrieval".to_string()),
                kind: Some("decision".to_string()),
                ..Default::default()
            },
        ))
        .expect("taxonomy-trace assembly should succeed");

    assert_eq!(assembled.present.world_fragments.len(), 1);
    let fragment = &assembled.present.world_fragments[0];
    assert_eq!(
        fragment.trace.applied_filters.topic.as_deref(),
        Some("retrieval")
    );
    assert_eq!(
        fragment.trace.applied_filters.kind.as_deref(),
        Some("decision")
    );
}

#[test]
fn assembler_preserves_domain_and_aspect_filters_in_fragment_trace() {
    let path = fresh_db_path("taxonomy-trace-domain-aspect");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    seed_taxonomy_filtered_records(&ingest);

    let assembled = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(&WorkingMemoryRequest::new("status ready").with_filters(
            agent_memos::search::SearchFilters {
                domain: Some("project".to_string()),
                aspect: Some("state".to_string()),
                ..Default::default()
            },
        ))
        .expect("taxonomy-trace assembly should succeed");

    assert_eq!(assembled.present.world_fragments.len(), 1);
    let fragment = &assembled.present.world_fragments[0];
    assert_eq!(
        fragment.trace.applied_filters.domain.as_deref(),
        Some("project")
    );
    assert_eq!(
        fragment.trace.applied_filters.aspect.as_deref(),
        Some("state")
    );
}

#[test]
fn assembler_preserves_temporal_filters_in_fragment_trace() {
    let path = fresh_db_path("temporal-filter-trace-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/temporal".to_string(),
            source_label: Some("temporal".to_string()),
            source_kind: None,
            content: "retrieval baseline keeps lexical search explainable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:15:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("temporal ingest should succeed");

    let assembled = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(&WorkingMemoryRequest::new("baseline").with_filters(
            agent_memos::search::SearchFilters {
                valid_at: Some("2026-04-16T12:30:00Z".to_string()),
                recorded_from: Some("2026-04-16T00:00:00Z".to_string()),
                recorded_to: Some("2026-04-17T00:00:00Z".to_string()),
                ..Default::default()
            },
        ))
        .expect("temporal-filtered assembly should succeed");

    assert_eq!(assembled.present.world_fragments.len(), 1);
    let fragment = &assembled.present.world_fragments[0];
    assert_eq!(
        fragment.trace.applied_filters.valid_at.as_deref(),
        Some("2026-04-16T12:30:00Z")
    );
    assert_eq!(
        fragment.trace.applied_filters.recorded_from.as_deref(),
        Some("2026-04-16T00:00:00Z")
    );
    assert_eq!(
        fragment.trace.applied_filters.recorded_to.as_deref(),
        Some("2026-04-17T00:00:00Z")
    );
}

#[test]
fn assembler_preserves_scope_record_type_and_truth_layer_filters_in_fragment_trace() {
    let path = fresh_db_path("scope-record-truth-filter-trace");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/filter-trace".to_string(),
            source_label: Some("filter-trace".to_string()),
            source_kind: None,
            content: "retrieval baseline keeps lexical search explainable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:20:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("filter-trace ingest should succeed");

    let assembled = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(&WorkingMemoryRequest::new("baseline").with_filters(
            agent_memos::search::SearchFilters {
                scope: Some(Scope::Project),
                record_type: Some(RecordType::Decision),
                truth_layer: Some(TruthLayer::T2),
                ..Default::default()
            },
        ))
        .expect("scope/record/truth filtered assembly should succeed");

    assert_eq!(assembled.present.world_fragments.len(), 1);
    let fragment = &assembled.present.world_fragments[0];
    assert_eq!(fragment.trace.applied_filters.scope, Some(Scope::Project));
    assert_eq!(
        fragment.trace.applied_filters.record_type,
        Some(RecordType::Decision)
    );
    assert_eq!(
        fragment.trace.applied_filters.truth_layer,
        Some(TruthLayer::T2)
    );
}

#[test]
fn assembler_preserves_structured_only_temporal_filters_in_fragment_trace() {
    let path = fresh_db_path("structured-only-temporal-filter-trace");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-temporal".to_string(),
            source_label: Some("structured-only-temporal".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T14:30:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("structured-only-temporal ingest should succeed");

    let assembled = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(&WorkingMemoryRequest::new("decision").with_filters(
            agent_memos::search::SearchFilters {
                valid_at: Some("2026-04-16T14:35:00Z".to_string()),
                recorded_from: Some("2026-04-16T00:00:00Z".to_string()),
                recorded_to: Some("2026-04-17T00:00:00Z".to_string()),
                ..Default::default()
            },
        ))
        .expect("structured-only temporal-filtered assembly should succeed");

    assert_eq!(assembled.present.world_fragments.len(), 1);
    let fragment = &assembled.present.world_fragments[0];
    assert_eq!(
        fragment.trace.applied_filters.valid_at.as_deref(),
        Some("2026-04-16T14:35:00Z")
    );
    assert_eq!(
        fragment.trace.applied_filters.recorded_from.as_deref(),
        Some("2026-04-16T00:00:00Z")
    );
    assert_eq!(
        fragment.trace.applied_filters.recorded_to.as_deref(),
        Some("2026-04-17T00:00:00Z")
    );
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "structured-only temporal-filtered assembly should preserve structured provenance"
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

#[test]
fn assembler_preserves_integrated_results_when_query_is_whitespace() {
    let path = fresh_db_path("integrated-results-whitespace-query");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/whitespace-integrated".to_string(),
            source_label: Some("whitespace-integrated".to_string()),
            source_kind: None,
            content: "explicit integrated evidence should survive blank assembly queries"
                .to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:11:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");

    let record_id = record.record_ids[0].clone();
    let search = SearchService::new(db.conn());
    let integrated_results = search
        .search(&SearchRequest::new("explicit integrated evidence").with_limit(1))
        .expect("search should succeed")
        .results;

    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("   ")
                .with_integrated_results(integrated_results)
                .with_active_goal("preserve explicit evidence across blank queries"),
        )
        .expect("assembly should preserve explicit integrated results on blank query");

    let record_ids = working_memory
        .present
        .world_fragments
        .iter()
        .map(|fragment| fragment.record_id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        record_ids,
        vec![record_id.as_str()],
        "blank assembly query should not discard caller-provided integrated results"
    );
}

#[test]
fn assembler_uses_snapshot_backed_current_world_model_when_available() {
    let path = fresh_db_path("snapshot-backed-current-world-model");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let snapshot_record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/snapshot-backed-current-world-model".to_string(),
            source_label: Some("snapshot-backed-current-world-model".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "persisted current world model should back runtime assembly".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:11:01Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("snapshot-backed ingest should succeed");

    let snapshot_id = snapshot_record.record_ids[0].clone();
    let mut snapshot_result = SearchService::new(db.conn())
        .search(&SearchRequest::new("persisted current world model").with_limit(1))
        .expect("search should succeed")
        .results
        .into_iter()
        .next()
        .expect("snapshot result should exist");
    snapshot_result.snippet = "snapshot-backed world fragment".to_string();
    let expected_world_model = persist_current_world_model_snapshot(
        db.conn(),
        "subject://agent/snapshot-backed",
        snapshot_result,
        "world-model-snapshot-runtime-001",
    );

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("this live query should not be used")
                .with_subject_ref("subject://agent/snapshot-backed"),
        )
        .expect("assembly should load the persisted current world model");

    assert_eq!(
        working_memory.present.world_fragments,
        expected_world_model.project_fragments(),
        "runtime assembly should project the persisted current world-model snapshot back through the explicit seam"
    );
    assert!(
        working_memory
            .present
            .self_state
            .facts
            .iter()
            .any(|fact| fact.source_record_id.as_deref() == Some(snapshot_id.as_str())),
        "snapshot-backed assembly should still rebuild truth-derived self-state facts for downstream compatibility"
    );
}

#[test]
fn assembler_falls_back_to_live_retrieval_when_snapshot_is_missing() {
    let path = fresh_db_path("snapshot-missing-live-fallback");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let live_record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/snapshot-missing-live-fallback".to_string(),
            source_label: Some("snapshot-missing-live-fallback".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "live retrieval fallback must stay unchanged when no snapshot exists"
                .to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:11:02Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("live fallback ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("live retrieval fallback")
                .with_limit(1)
                .with_subject_ref("subject://agent/no-snapshot"),
        )
        .expect("assembly should fall back to live retrieval");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, live_record.record_ids[0]);
    assert_eq!(fragment.trace.matched_query, "live retrieval fallback");
}

#[test]
fn assembler_prefers_explicit_integrated_results_over_snapshot_backed_world_model() {
    let path = fresh_db_path("integrated-results-precede-snapshot");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/integrated-results-precede-snapshot".to_string(),
            source_label: Some("integrated-results-precede-snapshot".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "persisted snapshot should stay lower-precedence than explicit evidence"
                .to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:11:03Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("snapshot precedence ingest should succeed");
    let integrated_record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/integrated-results-precede-snapshot-live".to_string(),
            source_label: Some("integrated-results-precede-snapshot-live".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "explicit integrated evidence must outrank the persisted snapshot".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:11:04Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("integrated precedence ingest should succeed");

    let snapshot_result = SearchService::new(db.conn())
        .search(&SearchRequest::new("persisted snapshot").with_limit(1))
        .expect("snapshot search should succeed")
        .results
        .into_iter()
        .next()
        .expect("snapshot result should exist");
    persist_current_world_model_snapshot(
        db.conn(),
        "subject://agent/integrated-precedence",
        snapshot_result,
        "world-model-snapshot-runtime-002",
    );

    let mut integrated_result = SearchService::new(db.conn())
        .search(&SearchRequest::new("explicit integrated evidence").with_limit(1))
        .expect("integrated search should succeed")
        .results
        .into_iter()
        .next()
        .expect("integrated result should exist");
    integrated_result.snippet = "explicit integrated snippet outranks snapshot".to_string();

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("   ")
                .with_subject_ref("subject://agent/integrated-precedence")
                .with_integrated_results(vec![integrated_result]),
        )
        .expect("assembly should prefer explicit integrated results");

    assert_eq!(
        working_memory
            .present
            .world_fragments
            .iter()
            .map(|fragment| fragment.record_id.as_str())
            .collect::<Vec<_>>(),
        vec![integrated_record.record_ids[0].as_str()],
        "explicit integrated results should remain higher-precedence than the persisted current world model"
    );
    assert_eq!(
        working_memory.present.world_fragments[0].snippet,
        "explicit integrated snippet outranks snapshot"
    );
}

#[test]
fn snapshot_backed_world_fragments_remain_compatible_with_branch_materialization() {
    let path = fresh_db_path("snapshot-backed-branch-compatibility");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let snapshot_record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/snapshot-backed-branch-compatibility".to_string(),
            source_label: Some("snapshot-backed-branch-compatibility".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "snapshot-backed fragments must still support downstream branches".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:11:05Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("snapshot-backed branch ingest should succeed");

    let mut snapshot_result = SearchService::new(db.conn())
        .search(&SearchRequest::new("snapshot-backed fragments").with_limit(1))
        .expect("search should succeed")
        .results
        .into_iter()
        .next()
        .expect("snapshot result should exist");
    snapshot_result.snippet = "snapshot-backed branch fragment".to_string();
    let expected_world_model = persist_current_world_model_snapshot(
        db.conn(),
        "subject://agent/snapshot-branch",
        snapshot_result,
        "world-model-snapshot-runtime-003",
    );

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("ignore live retrieval in favor of the snapshot")
                .with_subject_ref("subject://agent/snapshot-branch")
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "review the snapshot-backed evidence",
                ))),
        )
        .expect("snapshot-backed fragments should remain compatible with branch materialization");

    assert_eq!(
        working_memory.present.world_fragments,
        expected_world_model.project_fragments()
    );
    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(
        working_memory.branches[0].supporting_evidence, working_memory.present.world_fragments,
        "default branch support should continue to consume outward world_fragments even when they come from the snapshot-backed runtime read model"
    );
    assert_eq!(
        working_memory.branches[0].supporting_evidence[0].record_id,
        snapshot_record.record_ids[0]
    );
}

#[test]
fn assembler_preserves_integrated_result_order_when_query_is_whitespace() {
    let path = fresh_db_path("integrated-results-whitespace-order");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let first = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/whitespace-integrated-order-first".to_string(),
            source_label: Some("whitespace-integrated-order-first".to_string()),
            source_kind: None,
            content: "first explicit integrated evidence".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:11:05Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("first ingest should succeed");
    let second = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/whitespace-integrated-order-second".to_string(),
            source_label: Some("whitespace-integrated-order-second".to_string()),
            source_kind: None,
            content: "second explicit integrated evidence".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:11:10Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("second ingest should succeed");

    let first_id = first.record_ids[0].clone();
    let second_id = second.record_ids[0].clone();
    let search = SearchService::new(db.conn());
    let first_result = search
        .search(&SearchRequest::new("first explicit integrated evidence").with_limit(1))
        .expect("first search should succeed")
        .results
        .into_iter()
        .next()
        .expect("first result should exist");
    let second_result = search
        .search(&SearchRequest::new("second explicit integrated evidence").with_limit(1))
        .expect("second search should succeed")
        .results
        .into_iter()
        .next()
        .expect("second result should exist");

    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("   ")
                .with_integrated_results(vec![second_result, first_result]),
        )
        .expect("assembly should preserve caller-provided integrated result ordering");

    assert_eq!(
        working_memory
            .present
            .world_fragments
            .iter()
            .map(|fragment| fragment.record_id.as_str())
            .collect::<Vec<_>>(),
        vec![second_id.as_str(), first_id.as_str()],
        "blank assembly query should preserve caller-provided integrated result ordering"
    );
}

#[test]
fn assembler_hydrates_dsl_from_repository_for_integrated_results_on_whitespace_query() {
    let path = fresh_db_path("integrated-results-whitespace-hydrates-dsl");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/whitespace-integrated-hydrates-dsl".to_string(),
            source_label: Some("whitespace-integrated-hydrates-dsl".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "lexical-first baseline should still carry a structured dsl payload"
                .to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:11:20Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("ingest should succeed");

    let record_id = record.record_ids[0].clone();
    let search = SearchService::new(db.conn());
    let mut integrated_results = search
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("search should succeed")
        .results;
    integrated_results[0].snippet = "caller provided integrated snippet".to_string();
    integrated_results[0].dsl = None;

    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let working_memory = assembler
        .assemble(&WorkingMemoryRequest::new("   ").with_integrated_results(integrated_results))
        .expect("assembly should hydrate missing dsl payloads from the repository");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record_id);
    assert_eq!(
        fragment.snippet, "caller provided integrated snippet",
        "dsl hydration should not overwrite the caller-provided integrated snippet"
    );
    let dsl = fragment
        .dsl
        .as_ref()
        .expect("repository-backed dsl payload should be restored on blank-query assembly");
    assert_eq!(dsl.domain, "project");
    assert_eq!(dsl.kind, "decision");
    assert!(!dsl.claim.is_empty());
}

#[test]
fn assembler_hydrates_dsl_for_explicit_branch_support_on_whitespace_query() {
    let path = fresh_db_path("integrated-results-whitespace-branch-hydrates-dsl");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/whitespace-integrated-branch-hydrates-dsl".to_string(),
            source_label: Some("whitespace-integrated-branch-hydrates-dsl".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "explicit branch support should inherit structured dsl hydration".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:11:25Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("ingest should succeed");

    let record_id = record.record_ids[0].clone();
    let search = SearchService::new(db.conn());
    let mut integrated_results = search
        .search(&SearchRequest::new("explicit branch support").with_limit(1))
        .expect("search should succeed")
        .results;
    integrated_results[0].dsl = None;

    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("   ")
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "review the explicit branch evidence",
                    ))
                    .with_supporting_record_ids(vec![record_id.clone()]),
                ),
        )
        .expect("assembly should hydrate missing dsl payloads for explicit branch support");

    assert_eq!(working_memory.branches.len(), 1);
    let fragment = &working_memory.branches[0].supporting_evidence[0];
    assert_eq!(fragment.record_id, record_id);
    let dsl = fragment
        .dsl
        .as_ref()
        .expect("branch support should receive repository-backed dsl hydration");
    assert_eq!(dsl.domain, "project");
    assert_eq!(dsl.kind, "decision");
    assert!(!dsl.claim.is_empty());
}

#[test]
fn assembler_uses_integrated_results_for_default_branch_support_on_whitespace_query() {
    let path = fresh_db_path("whitespace-integrated-branch-support");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/whitespace-integrated-branch-support".to_string(),
            source_label: Some("whitespace-integrated-branch-support".to_string()),
            source_kind: None,
            content:
                "explicit integrated evidence should feed default branch support on blank queries"
                    .to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:11:30Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");

    let record_id = record.record_ids[0].clone();
    let search = SearchService::new(db.conn());
    let integrated_results = search
        .search(&SearchRequest::new("explicit integrated evidence").with_limit(1))
        .expect("search should succeed")
        .results;

    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("   ")
                .with_integrated_results(integrated_results)
                .with_action_seed(ActionSeed::new(ActionCandidate::new(
                    ActionKind::Epistemic,
                    "review the explicit evidence",
                ))),
        )
        .expect("assembly should preserve integrated results as default branch support");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(
        working_memory.branches[0]
            .supporting_evidence
            .iter()
            .map(|fragment| fragment.record_id.as_str())
            .collect::<Vec<_>>(),
        vec![record_id.as_str()],
        "default branch support should inherit caller-provided integrated evidence even on blank query"
    );
}

#[test]
fn assembler_uses_integrated_results_for_explicit_branch_support_on_whitespace_query() {
    let path = fresh_db_path("whitespace-integrated-explicit-branch-support");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/whitespace-integrated-explicit-branch-support"
                .to_string(),
            source_label: Some("whitespace-integrated-explicit-branch-support".to_string()),
            source_kind: None,
            content:
                "explicit supporting ids should resolve against integrated evidence on blank queries"
                    .to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:11:45Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");

    let record_id = record.record_ids[0].clone();
    let search = SearchService::new(db.conn());
    let integrated_results = search
        .search(&SearchRequest::new("explicit supporting ids").with_limit(1))
        .expect("search should succeed")
        .results;

    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("   ")
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "review the explicit evidence",
                    ))
                    .with_supporting_record_ids(vec![record_id.clone()]),
                ),
        )
        .expect("assembly should resolve explicit branch support from integrated results");

    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(
        working_memory.branches[0]
            .supporting_evidence
            .iter()
            .map(|fragment| fragment.record_id.as_str())
            .collect::<Vec<_>>(),
        vec![record_id.as_str()],
        "explicit supporting ids should resolve against caller-provided integrated results on blank query"
    );
}

#[test]
fn assembler_prefers_explicit_support_order_over_integrated_result_order_on_whitespace_query() {
    let path = fresh_db_path("whitespace-integrated-explicit-support-order");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let first = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/whitespace-explicit-order-first".to_string(),
            source_label: Some("whitespace-explicit-order-first".to_string()),
            source_kind: None,
            content: "first explicit support record on blank query".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:12:10Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("first ingest should succeed");
    let second = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/whitespace-explicit-order-second".to_string(),
            source_label: Some("whitespace-explicit-order-second".to_string()),
            source_kind: None,
            content: "second explicit support record on blank query".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:12:20Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("second ingest should succeed");

    let first_id = first.record_ids[0].clone();
    let second_id = second.record_ids[0].clone();
    let search = SearchService::new(db.conn());
    let first_result = search
        .search(&SearchRequest::new("first explicit support record").with_limit(1))
        .expect("first search should succeed")
        .results
        .into_iter()
        .next()
        .expect("first result should exist");
    let second_result = search
        .search(&SearchRequest::new("second explicit support record").with_limit(1))
        .expect("second search should succeed")
        .results
        .into_iter()
        .next()
        .expect("second result should exist");

    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("   ")
                .with_integrated_results(vec![first_result, second_result])
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "review explicit support in declared order",
                    ))
                    .with_supporting_record_ids(vec![second_id.clone(), first_id.clone()]),
                ),
        )
        .expect("assembly should prefer explicit support order on blank query");

    assert_eq!(
        working_memory.branches[0]
            .supporting_evidence
            .iter()
            .map(|fragment| fragment.record_id.as_str())
            .collect::<Vec<_>>(),
        vec![second_id.as_str(), first_id.as_str()],
        "explicit supporting_record_ids should take precedence over integrated result ordering on blank query"
    );
}

#[test]
fn assembler_dedupes_duplicate_integrated_results_by_record_id() {
    let path = fresh_db_path("dedupe-duplicate-integrated-results");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/dedupe-primary".to_string(),
            source_label: Some("dedupe-primary".to_string()),
            source_kind: None,
            content: "duplicate integrated result should only appear once".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:12:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");

    let primary_id = primary.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let duplicated = search
        .search(&SearchRequest::new("duplicate integrated result").with_limit(1))
        .expect("search should succeed")
        .results
        .into_iter()
        .next()
        .expect("search should return the primary result");

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("duplicate integrated result")
                .with_limit(1)
                .with_integrated_results(vec![duplicated.clone(), duplicated]),
        )
        .expect("assembly should dedupe duplicate integrated results");

    let matching = working_memory
        .present
        .world_fragments
        .iter()
        .filter(|fragment| fragment.record_id == primary_id)
        .count();

    assert_eq!(
        matching, 1,
        "duplicate integrated results should collapse to one world fragment per record_id"
    );
}

#[test]
fn assembler_prefers_first_duplicate_integrated_result_payload() {
    let path = fresh_db_path("dedupe-duplicate-integrated-results-first-wins");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/dedupe-first-wins".to_string(),
            source_label: Some("dedupe-first-wins".to_string()),
            source_kind: None,
            content: "duplicate integrated result ordering should preserve first payload"
                .to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:12:30Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");

    let primary_id = primary.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let mut first = search
        .search(&SearchRequest::new("first payload").with_limit(1))
        .expect("search should succeed")
        .results
        .into_iter()
        .next()
        .expect("search should return the primary result");
    first.snippet = "FIRST PAYLOAD".to_string();
    first.trace.matched_query = "first payload".to_string();

    let mut second = first.clone();
    second.snippet = "SECOND PAYLOAD".to_string();
    second.trace.matched_query = "second payload".to_string();

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("dedupe first wins")
                .with_integrated_results(vec![first, second]),
        )
        .expect("assembly should keep the first duplicate integrated payload");

    let fragment = working_memory
        .present
        .world_fragments
        .iter()
        .find(|fragment| fragment.record_id == primary_id)
        .expect("deduped fragment should exist");

    assert_eq!(fragment.snippet, "FIRST PAYLOAD");
    assert_eq!(fragment.trace.matched_query, "first payload");
}

#[test]
fn assembler_branch_support_prefers_first_duplicate_integrated_result_payload() {
    let path = fresh_db_path("dedupe-duplicate-integrated-results-branch-first-wins");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/dedupe-branch-first-wins".to_string(),
            source_label: Some("dedupe-branch-first-wins".to_string()),
            source_kind: None,
            content: "duplicate integrated branch payload should preserve first occurrence"
                .to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:12:40Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");

    let primary_id = primary.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let mut first = search
        .search(&SearchRequest::new("first branch payload").with_limit(1))
        .expect("search should succeed")
        .results
        .into_iter()
        .next()
        .expect("search should return the primary result");
    first.snippet = "FIRST BRANCH PAYLOAD".to_string();
    first.trace.matched_query = "first branch payload".to_string();

    let mut second = first.clone();
    second.snippet = "SECOND BRANCH PAYLOAD".to_string();
    second.trace.matched_query = "second branch payload".to_string();

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("dedupe branch first wins")
                .with_integrated_results(vec![first, second])
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "review the deduped evidence",
                    ))
                    .with_supporting_record_ids(vec![primary_id.clone()]),
                ),
        )
        .expect("assembly should keep the first duplicate payload in branch support");

    let fragment = &working_memory.branches[0].supporting_evidence[0];
    assert_eq!(fragment.record_id, primary_id);
    assert_eq!(fragment.snippet, "FIRST BRANCH PAYLOAD");
    assert_eq!(fragment.trace.matched_query, "first branch payload");
}

#[test]
fn assembler_dedupes_duplicate_branch_supporting_record_ids() {
    let path = fresh_db_path("dedupe-duplicate-branch-supporting-record-ids");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/support-dedupe".to_string(),
            source_label: Some("support-dedupe".to_string()),
            source_kind: None,
            content: "duplicate branch support ids should only resolve once".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T11:13:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");

    let primary_id = primary.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("duplicate branch support ids")
                .with_limit(1)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate {
                        kind: ActionKind::Instrumental,
                        summary: "use deduped support".to_string(),
                        intent: None,
                        parameters: Vec::new(),
                        expected_effects: Vec::new(),
                    })
                    .with_supporting_record_ids(vec![primary_id.clone(), primary_id.clone()]),
                ),
        )
        .expect("assembly should dedupe duplicate branch supporting ids");

    let branch = working_memory
        .branches
        .first()
        .expect("branch should materialize");
    let matching = branch
        .supporting_evidence
        .iter()
        .filter(|fragment| fragment.record_id == primary_id)
        .count();

    assert_eq!(
        matching, 1,
        "duplicate branch supporting ids should collapse to one supporting evidence fragment"
    );
}

#[test]
fn assembler_preserves_branch_supporting_evidence_provenance_for_integrated_follow_up() {
    let path = fresh_db_path("follow-up-branch-support-provenance");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary-branch-provenance".to_string(),
            source_label: Some("primary-branch-provenance".to_string()),
            source_kind: None,
            content: "primary branch result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:10:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up-branch-provenance".to_string(),
            source_label: Some("follow-up-branch-provenance".to_string()),
            source_kind: None,
            content: "follow-up branch support provenance".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:15:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("follow-up ingest should succeed");

    let follow_up_id = follow_up.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary branch result").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let follow_up_result = search
        .search(&SearchRequest::new("follow-up branch support provenance").with_limit(1))
        .expect("follow-up search should succeed")
        .results;
    let mut integrated_results = primary_result;
    integrated_results.extend(follow_up_result);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary branch result")
                .with_limit(1)
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "inspect follow-up provenance",
                    ))
                    .with_supporting_record_ids(vec![follow_up_id.clone()]),
                ),
        )
        .expect("assembly should preserve branch supporting-evidence provenance");

    let follow_up_fragment = working_memory.branches[0]
        .supporting_evidence
        .iter()
        .find(|fragment| fragment.record_id == follow_up_id)
        .expect("branch supporting evidence should include follow-up fragment");

    assert_eq!(follow_up_fragment.provenance.origin, "ingest");
    assert_eq!(
        follow_up_fragment.provenance.imported_via.as_deref(),
        Some("ingest_service")
    );
    assert!(
        follow_up_fragment
            .provenance
            .derived_from
            .iter()
            .any(|value| value.starts_with("memo://project/follow-up-branch-provenance#"))
    );
    assert_eq!(
        follow_up_fragment.trace.matched_query,
        "follow-up branch support provenance"
    );
}

#[test]
fn assembler_preserves_branch_supporting_evidence_score_for_integrated_follow_up() {
    let path = fresh_db_path("follow-up-branch-support-score");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary-branch-score".to_string(),
            source_label: Some("primary-branch-score".to_string()),
            source_kind: None,
            content: "primary branch result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:20:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up-branch-score".to_string(),
            source_label: Some("follow-up-branch-score".to_string()),
            source_kind: None,
            content: "follow-up branch support score".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:25:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("follow-up ingest should succeed");

    let follow_up_id = follow_up.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary branch result").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let follow_up_result = search
        .search(&SearchRequest::new("follow-up branch support score").with_limit(1))
        .expect("follow-up search should succeed")
        .results;
    let expected_score = follow_up_result[0].score.clone();
    let mut integrated_results = primary_result;
    integrated_results.extend(follow_up_result);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary branch result")
                .with_limit(1)
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "inspect follow-up score",
                    ))
                    .with_supporting_record_ids(vec![follow_up_id.clone()]),
                ),
        )
        .expect("assembly should preserve branch supporting-evidence score");

    let follow_up_fragment = working_memory.branches[0]
        .supporting_evidence
        .iter()
        .find(|fragment| fragment.record_id == follow_up_id)
        .expect("branch supporting evidence should include follow-up fragment");

    assert_eq!(follow_up_fragment.score, expected_score);
    assert_eq!(
        follow_up_fragment.trace.matched_query,
        "follow-up branch support score"
    );
}

#[test]
fn assembler_preserves_branch_supporting_evidence_truth_context_for_integrated_follow_up() {
    let path = fresh_db_path("follow-up-branch-support-truth-context");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let governance = TruthGovernanceService::new(db.conn());

    let _primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary-branch-truth-context".to_string(),
            source_label: Some("primary-branch-truth-context".to_string()),
            source_kind: None,
            content: "primary branch result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:30:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up-branch-truth-context".to_string(),
            source_label: Some("follow-up-branch-truth-context".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "follow-up branch support truth context".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T3,
            recorded_at: "2026-04-16T12:35:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("follow-up ingest should succeed");

    let review = governance
        .create_promotion_review(CreatePromotionReviewRequest {
            review_id: "review-branch-support-truth-context".to_string(),
            source_record_id: follow_up.record_ids[0].clone(),
            created_at: "2026-04-16T12:35:30Z".to_string(),
            review_notes: Some(json!({
                "summary": "branch support follow-up review"
            })),
        })
        .expect("branch support truth-context review should create")
        .review;

    let follow_up_id = follow_up.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary branch result").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let follow_up_result = search
        .search(&SearchRequest::new("follow-up branch support truth context").with_limit(1))
        .expect("follow-up search should succeed")
        .results;
    let mut integrated_results = primary_result;
    integrated_results.extend(follow_up_result);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary branch result")
                .with_limit(1)
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "inspect follow-up truth context",
                    ))
                    .with_supporting_record_ids(vec![follow_up_id.clone()]),
                ),
        )
        .expect("assembly should preserve branch supporting-evidence truth context");

    let follow_up_fragment = working_memory.branches[0]
        .supporting_evidence
        .iter()
        .find(|fragment| fragment.record_id == follow_up_id)
        .expect("branch supporting evidence should include follow-up fragment");

    assert_eq!(follow_up_fragment.truth_context.truth_layer, TruthLayer::T3);
    assert_eq!(
        follow_up_fragment.truth_context.open_review_ids,
        vec![review.review_id]
    );
    let t3_state = follow_up_fragment
        .truth_context
        .t3_state
        .as_ref()
        .expect("branch supporting evidence should preserve t3 state");
    assert_eq!(
        t3_state.last_reviewed_at.as_deref(),
        Some("2026-04-16T12:35:30Z")
    );
}

#[test]
fn assembler_preserves_branch_supporting_evidence_dsl_for_integrated_follow_up() {
    let path = fresh_db_path("follow-up-branch-support-dsl");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary-branch-dsl".to_string(),
            source_label: Some("primary-branch-dsl".to_string()),
            source_kind: None,
            content: "primary branch result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:40:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up-branch-dsl".to_string(),
            source_label: Some("follow-up-branch-dsl".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:45:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("follow-up ingest should succeed");

    let follow_up_id = follow_up.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary branch result").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let follow_up_result = search
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("follow-up search should succeed")
        .results;
    let mut integrated_results = primary_result;
    integrated_results.extend(follow_up_result);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary branch result")
                .with_limit(1)
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "inspect follow-up dsl",
                    ))
                    .with_supporting_record_ids(vec![follow_up_id.clone()]),
                ),
        )
        .expect("assembly should preserve branch supporting-evidence dsl");

    let follow_up_fragment = working_memory.branches[0]
        .supporting_evidence
        .iter()
        .find(|fragment| fragment.record_id == follow_up_id)
        .expect("branch supporting evidence should include follow-up fragment");

    let dsl = follow_up_fragment
        .dsl
        .as_ref()
        .expect("branch supporting evidence should preserve dsl");
    assert_eq!(dsl.domain, "project");
    assert_eq!(dsl.kind, "decision");
    assert_eq!(dsl.source_ref, "memo://project/follow-up-branch-dsl");
    assert!(
        follow_up_fragment.snippet.contains("WHY:"),
        "branch supporting evidence should preserve the structured snippet surface: {:?}",
        follow_up_fragment.snippet
    );
}

#[test]
fn assembler_hydrates_missing_dsl_for_branch_supporting_evidence_on_integrated_follow_up() {
    let path = fresh_db_path("follow-up-branch-support-hydrate-dsl");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary-branch-hydrate-dsl".to_string(),
            source_label: Some("primary-branch-hydrate-dsl".to_string()),
            source_kind: None,
            content: "primary branch hydrate dsl".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:46:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up-branch-hydrate-dsl".to_string(),
            source_label: Some("follow-up-branch-hydrate-dsl".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:47:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("follow-up ingest should succeed");

    let follow_up_id = follow_up.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary branch hydrate dsl").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let mut follow_up_result = search
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("follow-up search should succeed")
        .results;
    follow_up_result[0].snippet = "caller-provided branch snippet".to_string();
    follow_up_result[0].dsl = None;
    let mut integrated_results = primary_result;
    integrated_results.extend(follow_up_result);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary branch hydrate dsl")
                .with_limit(1)
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "inspect follow-up hydrated dsl",
                    ))
                    .with_supporting_record_ids(vec![follow_up_id.clone()]),
                ),
        )
        .expect("assembly should hydrate missing dsl for branch supporting evidence");

    let follow_up_fragment = working_memory.branches[0]
        .supporting_evidence
        .iter()
        .find(|fragment| fragment.record_id == follow_up_id)
        .expect("branch supporting evidence should include follow-up fragment");

    assert_eq!(
        follow_up_fragment.snippet, "caller-provided branch snippet",
        "branch-support dsl hydration should preserve the caller-provided snippet"
    );
    let dsl = follow_up_fragment
        .dsl
        .as_ref()
        .expect("branch supporting evidence should recover its repository-backed dsl");
    assert_eq!(dsl.domain, "project");
    assert_eq!(dsl.kind, "decision");
    assert_eq!(
        dsl.source_ref,
        "memo://project/follow-up-branch-hydrate-dsl"
    );
}

#[test]
fn assembler_preserves_branch_supporting_evidence_trace_for_integrated_follow_up() {
    let path = fresh_db_path("follow-up-branch-support-trace");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary-branch-trace".to_string(),
            source_label: Some("primary-branch-trace".to_string()),
            source_kind: None,
            content: "primary branch result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:50:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up-branch-trace".to_string(),
            source_label: Some("follow-up-branch-trace".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:55:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("follow-up ingest should succeed");

    let follow_up_id = follow_up.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary branch result").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let follow_up_result = search
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("follow-up search should succeed")
        .results;
    let mut integrated_results = primary_result;
    integrated_results.extend(follow_up_result);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary branch result")
                .with_limit(1)
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "inspect follow-up trace",
                    ))
                    .with_supporting_record_ids(vec![follow_up_id.clone()]),
                ),
        )
        .expect("assembly should preserve branch supporting-evidence trace");

    let follow_up_fragment = working_memory.branches[0]
        .supporting_evidence
        .iter()
        .find(|fragment| fragment.record_id == follow_up_id)
        .expect("branch supporting evidence should include follow-up fragment");

    assert_eq!(
        follow_up_fragment.trace.channel_contribution,
        ChannelContribution::LexicalOnly
    );
    assert!(
        follow_up_fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && follow_up_fragment
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "branch supporting evidence should preserve both lexical and structured trace provenance"
    );
    assert_eq!(
        follow_up_fragment.trace.matched_query,
        "lexical-first baseline"
    );
}

#[test]
fn assembler_preserves_branch_supporting_evidence_filter_trace_for_integrated_follow_up() {
    let path = fresh_db_path("follow-up-branch-support-filter-trace");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary-branch-filter-trace".to_string(),
            source_label: Some("primary-branch-filter-trace".to_string()),
            source_kind: None,
            content: "primary branch result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up-branch-filter-trace".to_string(),
            source_label: Some("follow-up-branch-filter-trace".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:05:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("follow-up ingest should succeed");

    let follow_up_id = follow_up.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary branch result").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let follow_up_result = search
        .search(&SearchRequest::new("decision").with_limit(1).with_filters(
            agent_memos::search::SearchFilters {
                scope: Some(Scope::Project),
                record_type: Some(RecordType::Decision),
                truth_layer: Some(TruthLayer::T2),
                ..Default::default()
            },
        ))
        .expect("follow-up search should succeed")
        .results;
    let mut integrated_results = primary_result;
    integrated_results.extend(follow_up_result);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary branch result")
                .with_limit(1)
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "inspect follow-up filter trace",
                    ))
                    .with_supporting_record_ids(vec![follow_up_id.clone()]),
                ),
        )
        .expect("assembly should preserve branch supporting-evidence filter trace");

    let follow_up_fragment = working_memory.branches[0]
        .supporting_evidence
        .iter()
        .find(|fragment| fragment.record_id == follow_up_id)
        .expect("branch supporting evidence should include follow-up fragment");

    assert_eq!(
        follow_up_fragment.trace.applied_filters.scope,
        Some(Scope::Project)
    );
    assert_eq!(
        follow_up_fragment.trace.applied_filters.record_type,
        Some(RecordType::Decision)
    );
    assert_eq!(
        follow_up_fragment.trace.applied_filters.truth_layer,
        Some(TruthLayer::T2)
    );
    assert_eq!(follow_up_fragment.trace.matched_query, "decision");
}

#[test]
fn assembler_preserves_branch_supporting_evidence_citation_source_uri_for_integrated_follow_up() {
    let path = fresh_db_path("follow-up-branch-support-citation-source-uri");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary-branch-citation-source-uri".to_string(),
            source_label: Some("primary-branch-citation-source-uri".to_string()),
            source_kind: None,
            content: "primary branch result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:10:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up-branch-citation-source-uri".to_string(),
            source_label: Some("follow-up-branch-citation-source-uri".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "follow-up branch citation source uri".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:15:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("follow-up ingest should succeed");

    let follow_up_id = follow_up.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary branch result").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let follow_up_result = search
        .search(&SearchRequest::new("follow-up branch citation source uri").with_limit(1))
        .expect("follow-up search should succeed")
        .results;
    let mut integrated_results = primary_result;
    integrated_results.extend(follow_up_result);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary branch result")
                .with_limit(1)
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "inspect follow-up citation source uri",
                    ))
                    .with_supporting_record_ids(vec![follow_up_id.clone()]),
                ),
        )
        .expect("assembly should preserve branch supporting-evidence citation source uri");

    let follow_up_fragment = working_memory.branches[0]
        .supporting_evidence
        .iter()
        .find(|fragment| fragment.record_id == follow_up_id)
        .expect("branch supporting evidence should include follow-up fragment");

    assert_eq!(
        follow_up_fragment.citation.source_uri,
        "memo://project/follow-up-branch-citation-source-uri"
    );
    assert_eq!(follow_up_fragment.citation.record_id, follow_up_id);
}

#[test]
fn assembler_preserves_branch_supporting_evidence_citation_validity_for_integrated_follow_up() {
    let path = fresh_db_path("follow-up-branch-support-citation-validity");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary-branch-citation-validity".to_string(),
            source_label: Some("primary-branch-citation-validity".to_string()),
            source_kind: None,
            content: "primary branch result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:20:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up-branch-citation-validity".to_string(),
            source_label: Some("follow-up-branch-citation-validity".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "follow-up branch citation validity".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:25:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("follow-up ingest should succeed");

    let follow_up_id = follow_up.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary branch result").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let follow_up_result = search
        .search(&SearchRequest::new("follow-up branch citation validity").with_limit(1))
        .expect("follow-up search should succeed")
        .results;
    let mut integrated_results = primary_result;
    integrated_results.extend(follow_up_result);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary branch result")
                .with_limit(1)
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "inspect follow-up citation validity",
                    ))
                    .with_supporting_record_ids(vec![follow_up_id.clone()]),
                ),
        )
        .expect("assembly should preserve branch supporting-evidence citation validity");

    let follow_up_fragment = working_memory.branches[0]
        .supporting_evidence
        .iter()
        .find(|fragment| fragment.record_id == follow_up_id)
        .expect("branch supporting evidence should include follow-up fragment");

    assert_eq!(
        follow_up_fragment.citation.validity.valid_from.as_deref(),
        Some("2026-04-10T00:00:00Z")
    );
    assert_eq!(
        follow_up_fragment.citation.validity.valid_to.as_deref(),
        Some("2026-04-20T00:00:00Z")
    );
}

#[test]
fn assembler_preserves_branch_supporting_evidence_citation_recorded_at_for_integrated_follow_up() {
    let path = fresh_db_path("follow-up-branch-support-citation-recorded-at");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary-branch-citation-recorded-at".to_string(),
            source_label: Some("primary-branch-citation-recorded-at".to_string()),
            source_kind: None,
            content: "primary branch result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:30:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up-branch-citation-recorded-at".to_string(),
            source_label: Some("follow-up-branch-citation-recorded-at".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "follow-up branch citation recorded at".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:35:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("follow-up ingest should succeed");

    let follow_up_id = follow_up.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary branch result").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let follow_up_result = search
        .search(&SearchRequest::new("follow-up branch citation recorded at").with_limit(1))
        .expect("follow-up search should succeed")
        .results;
    let mut integrated_results = primary_result;
    integrated_results.extend(follow_up_result);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary branch result")
                .with_limit(1)
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "inspect follow-up citation recorded at",
                    ))
                    .with_supporting_record_ids(vec![follow_up_id.clone()]),
                ),
        )
        .expect("assembly should preserve branch supporting-evidence citation recorded_at");

    let follow_up_fragment = working_memory.branches[0]
        .supporting_evidence
        .iter()
        .find(|fragment| fragment.record_id == follow_up_id)
        .expect("branch supporting evidence should include follow-up fragment");

    assert_eq!(
        follow_up_fragment.citation.recorded_at,
        "2026-04-16T13:35:00Z"
    );
}

#[test]
fn assembler_preserves_branch_supporting_evidence_citation_source_kind_for_integrated_follow_up() {
    let path = fresh_db_path("follow-up-branch-support-citation-source-kind");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary-branch-citation-source-kind".to_string(),
            source_label: Some("primary-branch-citation-source-kind".to_string()),
            source_kind: None,
            content: "primary branch result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:40:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up-branch-citation-source-kind".to_string(),
            source_label: Some("follow-up-branch-citation-source-kind".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "follow-up branch citation source kind".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:45:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("follow-up ingest should succeed");

    let follow_up_id = follow_up.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary branch result").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let follow_up_result = search
        .search(&SearchRequest::new("follow-up branch citation source kind").with_limit(1))
        .expect("follow-up search should succeed")
        .results;
    let mut integrated_results = primary_result;
    integrated_results.extend(follow_up_result);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary branch result")
                .with_limit(1)
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "inspect follow-up citation source kind",
                    ))
                    .with_supporting_record_ids(vec![follow_up_id.clone()]),
                ),
        )
        .expect("assembly should preserve branch supporting-evidence citation source_kind");

    let follow_up_fragment = working_memory.branches[0]
        .supporting_evidence
        .iter()
        .find(|fragment| fragment.record_id == follow_up_id)
        .expect("branch supporting evidence should include follow-up fragment");

    assert_eq!(follow_up_fragment.citation.source_kind, SourceKind::Note);
}

#[test]
fn assembler_preserves_branch_supporting_evidence_citation_source_label_for_integrated_follow_up() {
    let path = fresh_db_path("follow-up-branch-support-citation-source-label");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary-branch-citation-source-label".to_string(),
            source_label: Some("primary-branch-citation-source-label".to_string()),
            source_kind: None,
            content: "primary branch result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:50:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up-branch-citation-source-label".to_string(),
            source_label: Some("follow-up-branch-citation-source-label".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "follow-up branch citation source label".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:55:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("follow-up ingest should succeed");

    let follow_up_id = follow_up.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary branch result").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let follow_up_result = search
        .search(&SearchRequest::new("follow-up branch citation source label").with_limit(1))
        .expect("follow-up search should succeed")
        .results;
    let mut integrated_results = primary_result;
    integrated_results.extend(follow_up_result);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary branch result")
                .with_limit(1)
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "inspect follow-up citation source label",
                    ))
                    .with_supporting_record_ids(vec![follow_up_id.clone()]),
                ),
        )
        .expect("assembly should preserve branch supporting-evidence citation source_label");

    let follow_up_fragment = working_memory.branches[0]
        .supporting_evidence
        .iter()
        .find(|fragment| fragment.record_id == follow_up_id)
        .expect("branch supporting evidence should include follow-up fragment");

    assert_eq!(
        follow_up_fragment.citation.source_label.as_deref(),
        Some("follow-up-branch-citation-source-label")
    );
}

#[test]
fn assembler_preserves_branch_supporting_evidence_citation_anchor_for_integrated_follow_up() {
    let path = fresh_db_path("follow-up-branch-support-citation-anchor");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let _primary = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/primary-branch-citation-anchor".to_string(),
            source_label: Some("primary-branch-citation-anchor".to_string()),
            source_kind: None,
            content: "primary branch result".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T14:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("primary ingest should succeed");
    let follow_up = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/follow-up-branch-citation-anchor".to_string(),
            source_label: Some("follow-up-branch-citation-anchor".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "follow-up branch citation anchor".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T14:05:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("follow-up ingest should succeed");

    let follow_up_id = follow_up.record_ids[0].clone();
    let assembler = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider);
    let search = SearchService::new(db.conn());
    let primary_result = search
        .search(&SearchRequest::new("primary branch result").with_limit(1))
        .expect("primary search should succeed")
        .results;
    let follow_up_result = search
        .search(&SearchRequest::new("follow-up branch citation anchor").with_limit(1))
        .expect("follow-up search should succeed")
        .results;
    let mut integrated_results = primary_result;
    integrated_results.extend(follow_up_result);

    let working_memory = assembler
        .assemble(
            &WorkingMemoryRequest::new("primary branch result")
                .with_limit(1)
                .with_integrated_results(integrated_results)
                .with_action_seed(
                    ActionSeed::new(ActionCandidate::new(
                        ActionKind::Epistemic,
                        "inspect follow-up citation anchor",
                    ))
                    .with_supporting_record_ids(vec![follow_up_id.clone()]),
                ),
        )
        .expect("assembly should preserve branch supporting-evidence citation anchor");

    let follow_up_fragment = working_memory.branches[0]
        .supporting_evidence
        .iter()
        .find(|fragment| fragment.record_id == follow_up_id)
        .expect("branch supporting evidence should include follow-up fragment");

    assert_eq!(follow_up_fragment.citation.anchor.chunk_index, 0);
    assert_eq!(follow_up_fragment.citation.anchor.chunk_count, 1);
    assert!(matches!(
        follow_up_fragment.citation.anchor.anchor,
        ChunkAnchor::LineRange {
            start_line: 1,
            end_line: 1
        }
    ));
}

#[test]
fn assembler_preserves_structured_recall_trace_into_world_fragments() {
    let path = fresh_db_path("structured-trace-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-trace".to_string(),
            source_label: Some("structured-trace".to_string()),
            source_kind: None,
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("structured-trace ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_limit(1))
        .expect("structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve structured trace");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "working-memory fragments should preserve structured recall provenance"
    );
    assert_eq!(
        fragment.trace.channel_contribution,
        ChannelContribution::LexicalOnly
    );
}

#[test]
fn assembler_preserves_structured_snippet_priority_in_world_fragments() {
    let path = fresh_db_path("structured-snippet-priority-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-snippet-priority".to_string(),
            source_label: Some("structured-snippet-priority".to_string()),
            source_kind: None,
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:10:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("structured-snippet-priority ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve structured snippet priority");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert!(
        fragment.snippet.contains("WHY:"),
        "working-memory fragments should preserve the structured snippet chosen by retrieval: {:?}",
        fragment.snippet
    );
}

#[test]
fn assembler_preserves_structured_only_snippet_in_world_fragments() {
    let path = fresh_db_path("structured-only-snippet-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-snippet".to_string(),
            source_label: Some("structured-only-snippet".to_string()),
            source_kind: None,
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:15:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("structured-only-snippet ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_limit(1))
        .expect("structured-only recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve structured-only snippet");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert!(
        fragment.snippet.contains("WHY:"),
        "structured-only fragments should preserve the structured snippet chosen by retrieval: {:?}",
        fragment.snippet
    );
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "structured-only snippet coverage should still preserve structured provenance"
    );
}

#[test]
fn assembler_preserves_structured_only_score_on_fragments() {
    let path = fresh_db_path("structured-only-score-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-score".to_string(),
            source_label: Some("structured-only-score".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:28:40Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("structured-only score ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_limit(1))
        .expect("structured-only search should succeed")
        .results;
    let expected_score = results[0].score.clone();

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve structured-only score");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.score, expected_score);
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "structured-only fragment score coverage should preserve structured provenance"
    );
}

#[test]
fn assembler_preserves_source_metadata_in_fragment_citations() {
    let path = fresh_db_path("source-metadata-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/source-metadata".to_string(),
            source_label: Some("source-metadata".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "retrieval baseline keeps lexical search explainable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:30:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("source-metadata ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_limit(1))
        .expect("search should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve source metadata");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.citation.source_kind, SourceKind::Note);
    assert_eq!(
        fragment.citation.source_label.as_deref(),
        Some("source-metadata")
    );
    assert_eq!(
        fragment.citation.source_uri,
        "memo://project/source-metadata"
    );
}

#[test]
fn assembler_preserves_structured_only_record_provenance_on_fragments() {
    let path = fresh_db_path("structured-only-record-provenance-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-record-provenance-assembly".to_string(),
            source_label: Some("structured-only-record-provenance-assembly".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:33:40Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("structured-only record-provenance ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(&WorkingMemoryRequest::new("decision"))
        .expect("assembly should preserve structured-only record provenance");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.provenance.origin, "ingest");
    assert_eq!(
        fragment.provenance.imported_via.as_deref(),
        Some("ingest_service")
    );
    assert!(
        fragment
            .provenance
            .derived_from
            .iter()
            .any(|value| value
                .starts_with("memo://project/structured-only-record-provenance-assembly#")),
        "structured-only fragments should preserve the source-derived provenance anchor"
    );
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "structured-only fragment provenance coverage should preserve structured provenance"
    );
}

#[test]
fn assembler_preserves_validity_window_in_fragment_citations() {
    let path = fresh_db_path("validity-window-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/validity-window".to_string(),
            source_label: Some("validity-window".to_string()),
            source_kind: None,
            content: "retrieval baseline keeps lexical search explainable".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:35:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("validity-window ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("baseline").with_limit(1))
        .expect("search should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve validity window");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(
        fragment.citation.validity.valid_from.as_deref(),
        Some("2026-04-10T00:00:00Z")
    );
    assert_eq!(
        fragment.citation.validity.valid_to.as_deref(),
        Some("2026-04-20T00:00:00Z")
    );
}

#[test]
fn assembler_preserves_structured_only_recall_contract_end_to_end() {
    let path = fresh_db_path("structured-only-contract-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-contract".to_string(),
            source_label: Some("structured-only-contract".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:40:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("structured-only-contract ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_limit(1))
        .expect("structured-only recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve the structured-only contract");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.citation.source_kind, SourceKind::Note);
    assert_eq!(
        fragment.citation.source_label.as_deref(),
        Some("structured-only-contract")
    );
    assert_eq!(
        fragment.citation.validity.valid_from.as_deref(),
        Some("2026-04-10T00:00:00Z")
    );
    assert_eq!(
        fragment.citation.validity.valid_to.as_deref(),
        Some("2026-04-20T00:00:00Z")
    );
    assert!(
        fragment.snippet.contains("WHY:"),
        "structured-only assembly should preserve the structured snippet surface: {:?}",
        fragment.snippet
    );
    assert_eq!(
        fragment.trace.channel_contribution,
        ChannelContribution::LexicalOnly
    );
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "structured-only assembly should preserve structured provenance"
    );
}

#[test]
fn assembler_preserves_structured_only_source_uri_in_fragment_citation() {
    let path = fresh_db_path("structured-only-source-uri");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-source-uri".to_string(),
            source_label: Some("structured-only-source-uri".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:42:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("structured-only-source-uri ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_limit(1))
        .expect("structured-only recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve structured-only source uri");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(
        fragment.citation.source_uri,
        "memo://project/structured-only-source-uri"
    );
}

#[test]
fn assembler_preserves_structured_only_source_label_in_fragment_citation() {
    let path = fresh_db_path("structured-only-source-label");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-source-label".to_string(),
            source_label: Some("structured-only-source-label".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:43:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("structured-only-source-label ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_limit(1))
        .expect("structured-only recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve structured-only source label");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(
        fragment.citation.source_label.as_deref(),
        Some("structured-only-source-label")
    );
}

#[test]
fn assembler_preserves_structured_only_source_kind_in_fragment_citation() {
    let path = fresh_db_path("structured-only-source-kind");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-source-kind".to_string(),
            source_label: Some("structured-only-source-kind".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:44:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("structured-only-source-kind ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_limit(1))
        .expect("structured-only recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve structured-only source kind");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.citation.source_kind, SourceKind::Note);
}

#[test]
fn assembler_preserves_structured_only_recorded_at_in_fragment_citation() {
    let path = fresh_db_path("structured-only-recorded-at");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-recorded-at".to_string(),
            source_label: Some("structured-only-recorded-at".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:44:20Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("structured-only-recorded-at ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_limit(1))
        .expect("structured-only recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve structured-only recorded_at");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.citation.recorded_at, "2026-04-16T13:44:20Z");
}

#[test]
fn assembler_preserves_structured_only_line_range_anchor_in_fragment_citation() {
    let path = fresh_db_path("structured-only-line-range-anchor");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-line-range-anchor".to_string(),
            source_label: Some("structured-only-line-range-anchor".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:44:25Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("structured-only-line-range-anchor ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_limit(1))
        .expect("structured-only recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve structured-only line-range anchor");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert!(matches!(
        fragment.citation.anchor.anchor,
        ChunkAnchor::LineRange {
            start_line: 1,
            end_line: 1
        }
    ));
}

#[test]
fn assembler_preserves_structured_only_chunk_anchor_in_fragment_citation() {
    let path = fresh_db_path("structured-only-chunk-anchor");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-chunk-anchor".to_string(),
            source_label: Some("structured-only-chunk-anchor".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:44:27Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("structured-only-chunk-anchor ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_limit(1))
        .expect("structured-only recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve structured-only chunk anchor");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.citation.anchor.chunk_index, 0);
    assert_eq!(fragment.citation.anchor.chunk_count, 1);
}

#[test]
fn assembler_preserves_structured_only_scope_on_fragments() {
    let path = fresh_db_path("structured-only-scope");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-scope".to_string(),
            source_label: Some("structured-only-scope".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:44:30Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("structured-only-scope ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_limit(1))
        .expect("structured-only recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve structured-only scope");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(
        fragment.citation.source_uri,
        "memo://project/structured-only-scope"
    );
    assert_eq!(fragment.trace.applied_filters.scope, None);
}

#[test]
fn assembler_preserves_structured_only_truth_layer_on_fragments() {
    let path = fresh_db_path("structured-only-truth-layer");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-truth-layer".to_string(),
            source_label: Some("structured-only-truth-layer".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:44:40Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("structured-only-truth-layer ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_limit(1))
        .expect("structured-only recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve structured-only truth layer");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.truth_context.truth_layer, TruthLayer::T2);
    assert_eq!(fragment.citation.record_id, record.record_ids[0]);
}

#[test]
fn assembler_preserves_structured_only_open_candidate_ids_in_truth_context() {
    let path = fresh_db_path("structured-only-open-candidates");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let governance = TruthGovernanceService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-open-candidates".to_string(),
            source_label: Some("structured-only-open-candidates".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:44:45Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("structured-only-open-candidates ingest should succeed");
    let basis = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-open-candidates-basis".to_string(),
            source_label: Some("structured-only-open-candidates-basis".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "basis evidence for ontology candidacy".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:44:46Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("structured-only-open-candidates basis ingest should succeed");

    let candidate = governance
        .create_ontology_candidate(CreateOntologyCandidateRequest {
            candidate_id: "candidate-structured-only-open".to_string(),
            source_record_id: record.record_ids[0].clone(),
            basis_record_ids: vec![record.record_ids[0].clone(), basis.record_ids[0].clone()],
            proposed_structure: json!({
                "kind": "ontology_node",
                "label": "structured-only candidate"
            }),
            created_at: "2026-04-16T13:44:47Z".to_string(),
        })
        .expect("structured-only open candidate should create");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_limit(1))
        .expect("structured-only recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve structured-only open candidates");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(
        fragment.truth_context.open_candidate_ids,
        vec![candidate.candidate_id]
    );
    assert!(fragment.truth_context.open_review_ids.is_empty());
}

#[test]
fn assembler_preserves_structured_only_open_review_ids_in_truth_context() {
    let path = fresh_db_path("structured-only-open-reviews");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let governance = TruthGovernanceService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-open-reviews".to_string(),
            source_label: Some("structured-only-open-reviews".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T3,
            recorded_at: "2026-04-16T13:44:48Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("structured-only-open-reviews ingest should succeed");

    let review = governance
        .create_promotion_review(CreatePromotionReviewRequest {
            review_id: "review-structured-only-open".to_string(),
            source_record_id: record.record_ids[0].clone(),
            created_at: "2026-04-16T13:44:49Z".to_string(),
            review_notes: Some(json!({
                "summary": "structured-only open review"
            })),
        })
        .expect("structured-only open review should create")
        .review;

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_limit(1))
        .expect("structured-only recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve structured-only open reviews");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(
        fragment.truth_context.open_review_ids,
        vec![review.review_id]
    );
    assert!(fragment.truth_context.open_candidate_ids.is_empty());
    let t3_state = fragment
        .truth_context
        .t3_state
        .as_ref()
        .expect("structured-only T3 fragments should carry t3 state");
    assert_eq!(
        t3_state.last_reviewed_at.as_deref(),
        Some("2026-04-16T13:44:49Z")
    );
}

#[test]
fn assembler_preserves_structured_only_t3_state_details_in_truth_context() {
    let path = fresh_db_path("structured-only-t3-state-details");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-t3-state-details".to_string(),
            source_label: Some("structured-only-t3-state-details".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T3,
            recorded_at: "2026-04-16T13:44:57Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("structured-only-t3-state-details ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("decision").with_limit(1))
        .expect("structured-only recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("decision")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve structured-only t3 state details");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    let t3_state = fragment
        .truth_context
        .t3_state
        .as_ref()
        .expect("structured-only T3 fragments should carry t3 state");
    assert_eq!(t3_state.confidence, T3Confidence::Medium);
    assert_eq!(t3_state.revocation_state, T3RevocationState::Active);
    assert!(fragment.truth_context.open_review_ids.is_empty());
    assert!(fragment.truth_context.open_candidate_ids.is_empty());
}

#[test]
fn assembler_preserves_mixed_open_candidate_ids_in_truth_context() {
    let path = fresh_db_path("mixed-open-candidates");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let governance = TruthGovernanceService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-open-candidates".to_string(),
            source_label: Some("mixed-open-candidates".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:44:50Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-open-candidates ingest should succeed");
    let basis = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-open-candidates-basis".to_string(),
            source_label: Some("mixed-open-candidates-basis".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "basis evidence for mixed ontology candidacy".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:44:51Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-open-candidates basis ingest should succeed");

    let candidate = governance
        .create_ontology_candidate(CreateOntologyCandidateRequest {
            candidate_id: "candidate-mixed-open".to_string(),
            source_record_id: record.record_ids[0].clone(),
            basis_record_ids: vec![record.record_ids[0].clone(), basis.record_ids[0].clone()],
            proposed_structure: json!({
                "kind": "ontology_node",
                "label": "mixed candidate"
            }),
            created_at: "2026-04-16T13:44:52Z".to_string(),
        })
        .expect("mixed open candidate should create");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed open candidates");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(
        fragment.truth_context.open_candidate_ids,
        vec![candidate.candidate_id]
    );
    assert!(fragment.truth_context.open_review_ids.is_empty());
}

#[test]
fn assembler_preserves_mixed_open_review_ids_in_truth_context() {
    let path = fresh_db_path("mixed-open-reviews");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());
    let governance = TruthGovernanceService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-open-reviews".to_string(),
            source_label: Some("mixed-open-reviews".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T3,
            recorded_at: "2026-04-16T13:44:55Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-open-reviews ingest should succeed");

    let review = governance
        .create_promotion_review(CreatePromotionReviewRequest {
            review_id: "review-mixed-open".to_string(),
            source_record_id: record.record_ids[0].clone(),
            created_at: "2026-04-16T13:44:56Z".to_string(),
            review_notes: Some(json!({
                "summary": "mixed open review"
            })),
        })
        .expect("mixed open review should create")
        .review;

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed open reviews");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(
        fragment.truth_context.open_review_ids,
        vec![review.review_id]
    );
    assert!(fragment.truth_context.open_candidate_ids.is_empty());
    let t3_state = fragment
        .truth_context
        .t3_state
        .as_ref()
        .expect("mixed T3 fragments should carry t3 state");
    assert_eq!(
        t3_state.last_reviewed_at.as_deref(),
        Some("2026-04-16T13:44:56Z")
    );
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && fragment
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "mixed open-review recall should preserve both provenance branches"
    );
}

#[test]
fn assembler_preserves_structured_only_matched_query_in_fragment_trace() {
    let path = fresh_db_path("structured-only-matched-query");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-matched-query".to_string(),
            source_label: Some("structured-only-matched-query".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:44:30Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("structured-only-matched-query ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(&WorkingMemoryRequest::new("decision").with_limit(1))
        .expect("assembly should preserve structured-only matched query");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.trace.matched_query, "decision");
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "structured-only matched-query coverage should still preserve structured provenance"
    );
}

#[test]
fn assembler_preserves_structured_only_core_filters_in_fragment_trace() {
    let path = fresh_db_path("structured-only-core-filter-trace");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/structured-only-core-filter".to_string(),
            source_label: Some("structured-only-core-filter".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:46:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("structured-only-core-filter ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(&WorkingMemoryRequest::new("decision").with_filters(
            agent_memos::search::SearchFilters {
                scope: Some(Scope::Project),
                record_type: Some(RecordType::Decision),
                truth_layer: Some(TruthLayer::T2),
                ..Default::default()
            },
        ))
        .expect("structured-only filtered assembly should succeed");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.trace.applied_filters.scope, Some(Scope::Project));
    assert_eq!(
        fragment.trace.applied_filters.record_type,
        Some(RecordType::Decision)
    );
    assert_eq!(
        fragment.trace.applied_filters.truth_layer,
        Some(TruthLayer::T2)
    );
}

#[test]
fn assembler_preserves_mixed_lexical_and_structured_provenance_in_world_fragments() {
    let path = fresh_db_path("mixed-provenance-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-provenance".to_string(),
            source_label: Some("mixed-provenance".to_string()),
            source_kind: None,
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:20:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("mixed-provenance ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed provenance");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(
        fragment.trace.channel_contribution,
        ChannelContribution::LexicalOnly
    );
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple),
        "working-memory fragments should preserve raw lexical provenance"
    );
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "working-memory fragments should preserve structured provenance"
    );
}

#[test]
fn assembler_preserves_mixed_provenance_and_structured_snippet_together() {
    let path = fresh_db_path("mixed-provenance-snippet-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-provenance-snippet".to_string(),
            source_label: Some("mixed-provenance-snippet".to_string()),
            source_kind: None,
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:25:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("mixed-provenance-snippet ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed provenance and snippet priority");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(
        fragment.trace.channel_contribution,
        ChannelContribution::LexicalOnly
    );
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple),
        "working-memory fragments should preserve raw lexical provenance"
    );
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "working-memory fragments should preserve structured provenance"
    );
    assert!(
        fragment.snippet.contains("WHY:"),
        "working-memory fragments should keep the structured snippet in the mixed-hit case: {:?}",
        fragment.snippet
    );
}

#[test]
fn assembler_preserves_mixed_score_on_fragments() {
    let path = fresh_db_path("mixed-score-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-score".to_string(),
            source_label: Some("mixed-score".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:26:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("mixed score ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed search should succeed")
        .results;
    let expected_score = results[0].score.clone();

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed score");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.score, expected_score);
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && fragment
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "mixed fragment score coverage should preserve both provenance branches"
    );
}

#[test]
fn assembler_preserves_embedding_only_trace_on_fragments() {
    let path = fresh_db_path("embedding-only-trace-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/embedding-only-trace".to_string(),
            source_label: Some("embedding-only-trace".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "retrieval fusion semantic retrieval fusion citations".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:26:30Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("embedding-only trace ingest should succeed");

    let results = SearchService::with_runtime_config(
        db.conn(),
        &ready_embedding_config(RetrievalMode::LexicalOnly),
        Some(RetrievalMode::EmbeddingOnly),
    )
    .search(&SearchRequest::new("retrieval fusion").with_limit(1))
    .expect("embedding-only search should succeed")
    .results;

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("retrieval fusion")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve embedding-only trace");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(
        fragment.trace.channel_contribution,
        ChannelContribution::EmbeddingOnly
    );
    assert_eq!(
        fragment.trace.query_strategies,
        vec![agent_memos::search::QueryStrategy::Embedding]
    );
}

#[test]
fn assembler_preserves_embedding_only_truncated_snippet_on_fragments() {
    let path = fresh_db_path("embedding-only-snippet-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    let content = "embedding snippets should preserve the original content surface even when they are routed through the second channel without lexical fallback and this sentence intentionally exceeds ninety six characters";
    let expected_snippet = content.chars().take(96).collect::<String>();

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/embedding-only-snippet".to_string(),
            source_label: Some("embedding-only-snippet".to_string()),
            source_kind: Some(SourceKind::Note),
            content: content.to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:26:31Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("embedding-only snippet ingest should succeed");

    let results = SearchService::with_runtime_config(
        db.conn(),
        &ready_embedding_config(RetrievalMode::LexicalOnly),
        Some(RetrievalMode::EmbeddingOnly),
    )
    .search(&SearchRequest::new("semantic second channel").with_limit(1))
    .expect("embedding-only search should succeed")
    .results;

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("semantic second channel")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve embedding-only snippet");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.snippet, expected_snippet);
}

#[test]
fn assembler_preserves_hybrid_trace_on_fragments() {
    let path = fresh_db_path("hybrid-trace-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        agent_memos::core::config::EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("builtin-16".to_string()),
            endpoint: None,
        },
    );

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/hybrid-trace".to_string(),
            source_label: Some("hybrid-trace".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "retrieval fusion semantic retrieval fusion citations".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:26:45Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("hybrid trace ingest should succeed");

    let results = SearchService::with_runtime_config(
        db.conn(),
        &ready_embedding_config(RetrievalMode::LexicalOnly),
        Some(RetrievalMode::Hybrid),
    )
    .search(&SearchRequest::new("retrieval fusion").with_limit(1))
    .expect("hybrid search should succeed")
    .results;

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("retrieval fusion")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve hybrid trace");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(
        fragment.trace.channel_contribution,
        ChannelContribution::Hybrid
    );
    assert_eq!(
        fragment.trace.query_strategies,
        vec![
            agent_memos::search::QueryStrategy::Jieba,
            agent_memos::search::QueryStrategy::Simple,
            agent_memos::search::QueryStrategy::Structured,
            agent_memos::search::QueryStrategy::Embedding,
        ],
        "hybrid fragment traces should preserve the full ready-path strategy ordering"
    );
}

#[test]
fn assembler_preserves_mixed_recall_contract_end_to_end() {
    let path = fresh_db_path("mixed-recall-contract-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-recall-contract".to_string(),
            source_label: Some("mixed-recall-contract".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:45:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-recall-contract ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve the mixed-recall contract");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.citation.record_id, record.record_ids[0]);
    assert_eq!(fragment.citation.source_kind, SourceKind::Note);
    assert_eq!(
        fragment.citation.source_label.as_deref(),
        Some("mixed-recall-contract")
    );
    assert_eq!(
        fragment.citation.validity.valid_from.as_deref(),
        Some("2026-04-10T00:00:00Z")
    );
    assert_eq!(
        fragment.trace.channel_contribution,
        ChannelContribution::LexicalOnly
    );
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple),
        "mixed-recall contract should preserve raw lexical provenance"
    );
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Structured),
        "mixed-recall contract should preserve structured provenance"
    );
    assert!(
        fragment.snippet.contains("WHY:"),
        "mixed-recall contract should preserve the structured snippet surface: {:?}",
        fragment.snippet
    );
}

#[test]
fn assembler_preserves_mixed_recall_valid_to_in_fragment_citation() {
    let path = fresh_db_path("mixed-recall-valid-to");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-recall-valid-to".to_string(),
            source_label: Some("mixed-recall-valid-to".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:50:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-recall-valid-to ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed valid_to");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(
        fragment.citation.validity.valid_to.as_deref(),
        Some("2026-04-20T00:00:00Z")
    );
}

#[test]
fn assembler_preserves_mixed_recall_valid_from_in_fragment_citation() {
    let path = fresh_db_path("mixed-recall-valid-from");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-recall-valid-from".to_string(),
            source_label: Some("mixed-recall-valid-from".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:48:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-recall-valid-from ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed valid_from");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(
        fragment.citation.validity.valid_from.as_deref(),
        Some("2026-04-10T00:00:00Z")
    );
}

#[test]
fn assembler_preserves_mixed_recall_recorded_at_in_fragment_citation() {
    let path = fresh_db_path("mixed-recall-recorded-at");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-recall-recorded-at".to_string(),
            source_label: Some("mixed-recall-recorded-at".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:52:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-recall-recorded-at ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed recorded_at");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.citation.recorded_at, "2026-04-16T13:52:00Z");
}

#[test]
fn assembler_preserves_mixed_recall_chunk_anchor_in_fragment_citation() {
    let path = fresh_db_path("mixed-recall-chunk-anchor");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-recall-chunk-anchor".to_string(),
            source_label: Some("mixed-recall-chunk-anchor".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:53:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-recall-chunk-anchor ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed chunk anchor");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.citation.anchor.chunk_index, 0);
    assert_eq!(fragment.citation.anchor.chunk_count, 1);
}

#[test]
fn assembler_preserves_mixed_recall_line_range_anchor_in_fragment_citation() {
    let path = fresh_db_path("mixed-recall-line-range-anchor");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-recall-line-range-anchor".to_string(),
            source_label: Some("mixed-recall-line-range-anchor".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:53:30Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-recall-line-range-anchor ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed line-range anchor");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert!(matches!(
        fragment.citation.anchor.anchor,
        ChunkAnchor::LineRange {
            start_line: 1,
            end_line: 1
        }
    ));
}

#[test]
fn assembler_preserves_mixed_recall_source_uri_in_fragment_citation() {
    let path = fresh_db_path("mixed-recall-source-uri");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-recall-source-uri".to_string(),
            source_label: Some("mixed-recall-source-uri".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T13:55:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-recall-source-uri ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed source uri");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(
        fragment.citation.source_uri,
        "memo://project/mixed-recall-source-uri"
    );
}

#[test]
fn assembler_preserves_mixed_recall_record_id_in_fragment_citation() {
    let path = fresh_db_path("mixed-recall-record-id");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-recall-record-id".to_string(),
            source_label: Some("mixed-recall-record-id".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T14:02:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-recall-record-id ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed record id");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.citation.record_id, record.record_ids[0]);
}

#[test]
fn assembler_preserves_mixed_recall_source_label_in_fragment_citation() {
    let path = fresh_db_path("mixed-recall-source-label");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-recall-source-label".to_string(),
            source_label: Some("mixed-recall-source-label".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T14:00:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-recall-source-label ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed source label");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(
        fragment.citation.source_label.as_deref(),
        Some("mixed-recall-source-label")
    );
}

#[test]
fn assembler_preserves_mixed_recall_source_kind_in_fragment_citation() {
    let path = fresh_db_path("mixed-recall-source-kind");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-recall-source-kind".to_string(),
            source_label: Some("mixed-recall-source-kind".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T14:05:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-recall-source-kind ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed source kind");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.citation.source_kind, SourceKind::Note);
}

#[test]
fn assembler_preserves_mixed_record_provenance_on_fragments() {
    let path = fresh_db_path("mixed-record-provenance-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-record-provenance-assembly".to_string(),
            source_label: Some("mixed-record-provenance-assembly".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T14:05:30Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed record-provenance ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed record provenance");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.provenance.origin, "ingest");
    assert_eq!(
        fragment.provenance.imported_via.as_deref(),
        Some("ingest_service")
    );
    assert!(
        fragment
            .provenance
            .derived_from
            .iter()
            .any(|value| value.starts_with("memo://project/mixed-record-provenance-assembly#")),
        "mixed fragments should preserve the source-derived provenance anchor"
    );
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && fragment
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "mixed fragment provenance coverage should preserve both provenance branches"
    );
}

#[test]
fn assembler_preserves_mixed_recall_truth_layer_in_truth_context() {
    let path = fresh_db_path("mixed-recall-truth-layer");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-recall-truth-layer".to_string(),
            source_label: Some("mixed-recall-truth-layer".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T14:06:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-recall-truth-layer ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed truth layer");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.truth_context.truth_layer, TruthLayer::T2);
}

#[test]
fn assembler_preserves_mixed_t3_state_details_in_truth_context() {
    let path = fresh_db_path("mixed-t3-state-details");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-t3-state-details".to_string(),
            source_label: Some("mixed-t3-state-details".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T3,
            recorded_at: "2026-04-16T14:07:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-t3-state-details ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed t3 state details");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    let t3_state = fragment
        .truth_context
        .t3_state
        .as_ref()
        .expect("mixed T3 fragments should carry t3 state");
    assert_eq!(t3_state.confidence, T3Confidence::Medium);
    assert_eq!(t3_state.revocation_state, T3RevocationState::Active);
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && fragment
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "mixed t3-state detail recall should preserve both provenance branches"
    );
}

#[test]
fn assembler_preserves_mixed_recall_matched_query_in_fragment_trace() {
    let path = fresh_db_path("mixed-recall-matched-query");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-recall-matched-query".to_string(),
            source_label: Some("mixed-recall-matched-query".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T14:10:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-recall-matched-query ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed matched query");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    assert_eq!(fragment.trace.matched_query, "lexical-first baseline");
}

#[test]
fn assembler_preserves_mixed_temporal_filters_in_fragment_trace() {
    let path = fresh_db_path("mixed-temporal-filter-trace");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-temporal".to_string(),
            source_label: Some("mixed-temporal".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T14:20:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-temporal ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline").with_filters(
                agent_memos::search::SearchFilters {
                    valid_at: Some("2026-04-16T14:30:00Z".to_string()),
                    recorded_from: Some("2026-04-16T00:00:00Z".to_string()),
                    recorded_to: Some("2026-04-17T00:00:00Z".to_string()),
                    ..Default::default()
                },
            ),
        )
        .expect("mixed temporal-filtered assembly should succeed");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(
        fragment.trace.applied_filters.valid_at.as_deref(),
        Some("2026-04-16T14:30:00Z")
    );
    assert_eq!(
        fragment.trace.applied_filters.recorded_from.as_deref(),
        Some("2026-04-16T00:00:00Z")
    );
    assert_eq!(
        fragment.trace.applied_filters.recorded_to.as_deref(),
        Some("2026-04-17T00:00:00Z")
    );
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && fragment
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "mixed temporal-filtered assembly should preserve both provenance branches"
    );
}

#[test]
fn assembler_preserves_mixed_recall_core_filters_in_fragment_trace() {
    let path = fresh_db_path("mixed-recall-core-filter-trace");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-recall-core-filter".to_string(),
            source_label: Some("mixed-recall-core-filter".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T14:15:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-recall-core-filter ingest should succeed");

    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline").with_filters(
                agent_memos::search::SearchFilters {
                    scope: Some(Scope::Project),
                    record_type: Some(RecordType::Decision),
                    truth_layer: Some(TruthLayer::T2),
                    ..Default::default()
                },
            ),
        )
        .expect("assembly should preserve mixed core filters");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.trace.applied_filters.scope, Some(Scope::Project));
    assert_eq!(
        fragment.trace.applied_filters.record_type,
        Some(RecordType::Decision)
    );
    assert_eq!(
        fragment.trace.applied_filters.truth_layer,
        Some(TruthLayer::T2)
    );
    assert!(
        fragment
            .trace
            .query_strategies
            .contains(&agent_memos::search::QueryStrategy::Simple)
            && fragment
                .trace
                .query_strategies
                .contains(&agent_memos::search::QueryStrategy::Structured),
        "mixed-recall core-filter coverage should preserve both provenance branches"
    );
}

#[test]
fn assembler_preserves_mixed_recall_dsl_payload_on_fragments() {
    let path = fresh_db_path("mixed-recall-dsl-payload");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let record = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/mixed-recall-dsl-payload".to_string(),
            source_label: Some("mixed-recall-dsl-payload".to_string()),
            source_kind: Some(SourceKind::Note),
            content: "use lexical-first as baseline because explainability matters".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T14:25:00Z".to_string(),
            valid_from: Some("2026-04-10T00:00:00Z".to_string()),
            valid_to: Some("2026-04-20T00:00:00Z".to_string()),
        })
        .expect("mixed-recall-dsl-payload ingest should succeed");

    let results = SearchService::new(db.conn())
        .search(&SearchRequest::new("lexical-first baseline").with_limit(1))
        .expect("mixed lexical + structured recall should succeed")
        .results;
    let working_memory = WorkingMemoryAssembler::new(db.conn(), TestSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("lexical-first baseline")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should preserve mixed dsl payload");

    assert_eq!(working_memory.present.world_fragments.len(), 1);
    let fragment = &working_memory.present.world_fragments[0];
    assert_eq!(fragment.record_id, record.record_ids[0]);
    let dsl = fragment
        .dsl
        .as_ref()
        .expect("mixed-recall fragments should keep the DSL payload");
    assert_eq!(dsl.domain, "project");
    assert_eq!(dsl.kind, "decision");
    assert!(!dsl.claim.is_empty());
}
