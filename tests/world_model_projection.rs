use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    cognition::{
        assembly::{MinimalSelfStateProvider, WorkingMemoryAssembler, WorkingMemoryRequest},
        working_memory::TruthContext,
        world_model::{
            CURRENT_WORLD_KEY, CurrentWorldSlice, ProjectedWorldModel, WorldFragmentProjection,
            load_runtime_current_world_model,
        },
    },
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::{
        dsl::FlatFactDslRecordV1,
        record::{
            ChunkAnchor, ChunkMetadata, MemoryRecord, Provenance, RecordTimestamp, RecordType,
            Scope, SourceKind, SourceRef, TruthLayer, ValidityWindow,
        },
        repository::MemoryRepository,
        truth::{
            PromotionDecisionState, PromotionReview, ReviewGateState, T3Confidence,
            T3RevocationState, T3State, TruthRecord,
        },
    },
    search::{
        AppliedFilters, ChannelContribution, Citation, ResultTrace, ScoreBreakdown, SearchRequest,
        SearchResult, SearchService,
    },
};

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-world-model-tests")
        .join(format!("{name}-{unique}"))
        .join("agent-memos.sqlite")
}

fn sample_record(id: &str, source_uri: &str, truth_layer: TruthLayer) -> MemoryRecord {
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
        truth_layer,
        provenance: Provenance {
            origin: "test".to_string(),
            imported_via: Some("fixture".to_string()),
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

fn sample_dsl(claim: &str, source_uri: &str) -> FlatFactDslRecordV1 {
    FlatFactDslRecordV1 {
        domain: "project".to_string(),
        topic: "world_model".to_string(),
        aspect: "state".to_string(),
        kind: "decision".to_string(),
        claim: claim.to_string(),
        truth_layer: "t3".to_string(),
        source_ref: source_uri.to_string(),
        why: Some("preserve explicit projection seam".to_string()),
        time: Some("current".to_string()),
        cond: Some("internal_only".to_string()),
        impact: Some("keeps outward compatibility".to_string()),
        conf: Some(0.82),
        rel: Some(vec!["supports:working_memory".to_string()]),
    }
}

fn sample_result(
    record: MemoryRecord,
    query: &str,
    dsl: Option<FlatFactDslRecordV1>,
) -> SearchResult {
    SearchResult {
        citation: Citation::from_record(&record).expect("chunk metadata should exist"),
        record,
        snippet: "explicit world-model seam keeps retrieval metadata intact".to_string(),
        dsl,
        score: ScoreBreakdown {
            lexical_raw: -1.7,
            lexical_base: 0.37,
            keyword_bonus: 0.04,
            importance_bonus: 0.08,
            recency_bonus: 0.02,
            emotion_bonus: 0.0,
            final_score: 0.51,
        },
        trace: ResultTrace {
            matched_query: query.to_string(),
            query_strategies: vec![agent_memos::search::QueryStrategy::Simple],
            channel_contribution: ChannelContribution::LexicalOnly,
            applied_filters: AppliedFilters::default(),
        },
    }
}

fn sample_t3_truth(record: MemoryRecord) -> TruthRecord {
    TruthRecord::T3 {
        base: record,
        t3_state: Some(T3State {
            record_id: "record-world".to_string(),
            confidence: T3Confidence::High,
            revocation_state: T3RevocationState::Active,
            revoked_at: None,
            revocation_reason: None,
            shared_conflict_note: Some("conflict reviewed".to_string()),
            last_reviewed_at: Some("2026-04-16T02:00:00Z".to_string()),
            created_at: Some("2026-04-16T01:00:00Z".to_string()),
            updated_at: Some("2026-04-16T02:00:00Z".to_string()),
        }),
        open_reviews: vec![PromotionReview {
            review_id: "review-1".to_string(),
            source_record_id: "record-world".to_string(),
            target_layer: TruthLayer::T3,
            result_trigger_state: ReviewGateState::Passed,
            evidence_review_state: ReviewGateState::Pending,
            consensus_check_state: ReviewGateState::Pending,
            metacog_approval_state: ReviewGateState::Passed,
            decision_state: PromotionDecisionState::Pending,
            review_notes: None,
            approved_at: None,
            created_at: "2026-04-16T01:15:00Z".to_string(),
            updated_at: "2026-04-16T01:30:00Z".to_string(),
        }],
    }
}

#[test]
fn world_fragment_projection_preserves_metadata_and_prefers_result_dsl() {
    let record = sample_record("record-world", "memo://project/world-model", TruthLayer::T3);
    let truth = sample_t3_truth(record.clone());
    let result_dsl = sample_dsl("result dsl wins", "memo://project/world-model");
    let repository_dsl = sample_dsl(
        "repository fallback should not win",
        "memo://project/world-model",
    );
    let result = sample_result(record.clone(), "world model", Some(result_dsl.clone()));
    let expected_citation = result.citation.clone();
    let expected_trace = result.trace.clone();
    let expected_score = result.score.clone();
    let expected_truth_context = TruthContext::from_truth_record(&truth);

    let projection =
        WorldFragmentProjection::from_search_result(result, &truth, Some(&repository_dsl));
    let world_model = ProjectedWorldModel::new(CurrentWorldSlice::new(vec![projection]));
    let fragments = world_model.project_fragments();

    assert_eq!(fragments.len(), 1);
    assert_eq!(
        fragments[0].dsl,
        Some(result_dsl),
        "search result DSL should outrank repository fallback"
    );
    assert_eq!(fragments[0].record_id, record.id);
    assert_eq!(fragments[0].citation, expected_citation);
    assert_eq!(fragments[0].provenance, record.provenance);
    assert_eq!(fragments[0].truth_context, expected_truth_context);
    assert_eq!(fragments[0].trace, expected_trace);
    assert_eq!(fragments[0].score, expected_score);
    assert_eq!(
        fragments[0].truth_context.open_review_ids,
        vec!["review-1".to_string()]
    );
}

#[test]
fn world_fragment_projection_uses_repository_dsl_fallback_when_result_dsl_is_missing() {
    let record = sample_record(
        "record-fallback",
        "memo://project/world-model-fallback",
        TruthLayer::T1,
    );
    let truth = TruthRecord::T1 {
        base: record.clone(),
    };
    let repository_dsl = sample_dsl(
        "repository fallback should hydrate missing DSL",
        "memo://project/world-model-fallback",
    );
    let result = sample_result(record, "fallback", None);

    let projection =
        WorldFragmentProjection::from_search_result(result, &truth, Some(&repository_dsl));
    let fragment = projection.project_fragment();

    assert_eq!(fragment.dsl, Some(repository_dsl));
}

#[test]
fn projected_world_model_reconstructs_from_persisted_snapshot_without_metadata_loss() {
    let record = sample_record(
        "record-snapshot",
        "memo://project/world-model-snapshot",
        TruthLayer::T3,
    );
    let truth = sample_t3_truth(record.clone());
    let projection = WorldFragmentProjection::from_search_result(
        sample_result(
            record,
            "world model snapshot",
            Some(sample_dsl(
                "snapshot reconstruction preserves fragment metadata",
                "memo://project/world-model-snapshot",
            )),
        ),
        &truth,
        None,
    );
    let original = ProjectedWorldModel::new(CurrentWorldSlice::new(vec![projection]));

    let snapshot = original.to_snapshot(
        "subject://agent/demo",
        "current",
        "world-model-snapshot-001",
        "2026-04-20T10:00:00Z",
        "2026-04-20T10:00:00Z",
    );
    let restored = ProjectedWorldModel::from_snapshot(&snapshot);

    assert_eq!(snapshot.subject_ref, "subject://agent/demo");
    assert_eq!(snapshot.world_key, "current");
    assert_eq!(snapshot.snapshot_id, "world-model-snapshot-001");
    assert_eq!(restored, original);
    assert_eq!(restored.project_fragments(), original.project_fragments());
}

#[test]
fn runtime_world_model_loader_reconstructs_current_snapshot_for_subject() {
    let path = fresh_db_path("world-model-runtime-loader");
    let db = Database::open(&path).expect("database should open");
    let repository = MemoryRepository::new(db.conn());

    let record = sample_record(
        "record-runtime-loader",
        "memo://project/world-model-runtime-loader",
        TruthLayer::T3,
    );
    let truth = sample_t3_truth(record.clone());
    let original = ProjectedWorldModel::new(CurrentWorldSlice::new(vec![
        WorldFragmentProjection::from_search_result(
            sample_result(
                record,
                "runtime loader",
                Some(sample_dsl(
                    "runtime current snapshot should round-trip through the loader",
                    "memo://project/world-model-runtime-loader",
                )),
            ),
            &truth,
            None,
        ),
    ]));
    let snapshot = original.to_snapshot(
        "subject://agent/runtime-loader",
        CURRENT_WORLD_KEY,
        "world-model-snapshot-runtime-loader",
        "2026-04-20T11:00:00Z",
        "2026-04-20T11:00:00Z",
    );

    repository
        .replace_world_model_snapshot(&snapshot)
        .expect("world-model snapshot should persist");

    let restored = load_runtime_current_world_model(&repository, "subject://agent/runtime-loader")
        .expect("runtime world-model loader should succeed")
        .expect("current world-model snapshot should exist");

    assert_eq!(restored, original);
    assert_eq!(restored.project_fragments(), original.project_fragments());
}

#[test]
fn working_memory_assembly_projects_world_fragments_through_world_model_seam() {
    let path = fresh_db_path("world-model-assembly");
    let db = Database::open(&path).expect("database should open");
    let ingest = IngestService::new(db.conn());

    let ingest_result = ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/world-model-assembly".to_string(),
            source_label: Some("world-model-assembly".to_string()),
            source_kind: None,
            content: "explicit world model assembly should preserve search metadata".to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-16T12:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");

    let repository = MemoryRepository::new(db.conn());
    let mut results = SearchService::new(db.conn())
        .search(&SearchRequest::new("world model assembly").with_limit(1))
        .expect("search should succeed")
        .results;
    assert_eq!(results.len(), 1, "fixture should yield one search result");
    results[0].dsl = None;

    let record_id = ingest_result.record_ids[0].clone();
    let truth = repository
        .get_truth_record(&record_id)
        .expect("truth lookup should succeed")
        .expect("truth should exist");
    let repository_dsl = repository
        .list_layered_records_for_ids(std::slice::from_ref(&record_id))
        .expect("layered records should load")
        .into_iter()
        .find(|record| record.record.id == record_id)
        .and_then(|record| record.dsl.map(|dsl| dsl.payload))
        .expect("repository DSL should exist");
    let expected = ProjectedWorldModel::new(CurrentWorldSlice::new(vec![
        WorldFragmentProjection::from_search_result(
            results[0].clone(),
            &truth,
            Some(&repository_dsl),
        ),
    ]))
    .project_fragments();

    let assembled = WorkingMemoryAssembler::new(db.conn(), MinimalSelfStateProvider)
        .assemble(
            &WorkingMemoryRequest::new("world model assembly")
                .with_limit(1)
                .with_integrated_results(results),
        )
        .expect("assembly should succeed");

    assert_eq!(assembled.present.world_fragments, expected);
    assert!(
        assembled.present.world_fragments[0].dsl.is_some(),
        "assembly should preserve outward EvidenceFragment compatibility while hydrating DSL fallback",
    );
}
