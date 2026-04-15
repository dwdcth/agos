use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::db::Database,
    memory::{
        record::{
            MemoryRecord, Provenance, RecordTimestamp, RecordType, Scope, SourceKind, SourceRef,
            TruthLayer, ValidityWindow,
        },
        repository::MemoryRepository,
        truth::{
            CandidateReviewState, OntologyCandidateState, PromotionDecisionState, ReviewGateState,
            T3Confidence, T3RevocationState, TruthRecord,
        },
    },
};

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-truth-tests")
        .join(format!("{name}-{unique}"))
        .join("truth.sqlite")
}

fn sample_record(id: &str, truth_layer: TruthLayer) -> MemoryRecord {
    MemoryRecord {
        id: id.to_string(),
        source: SourceRef {
            uri: format!("memo://truth/{id}"),
            kind: SourceKind::Note,
            label: Some(format!("truth-{id}")),
        },
        timestamp: RecordTimestamp {
            recorded_at: "2026-04-15T10:00:00Z".to_string(),
            created_at: "2026-04-15T10:00:00Z".to_string(),
            updated_at: "2026-04-15T10:00:00Z".to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Observation,
        truth_layer,
        provenance: Provenance {
            origin: "test".to_string(),
            imported_via: None,
            derived_from: vec!["seed-1".to_string()],
        },
        content_text: format!("record {id} for truth governance"),
        chunk: None,
        validity: ValidityWindow::default(),
    }
}

#[test]
fn truth_model_enums_parse_and_render_as_storage_values() {
    assert_eq!(T3Confidence::parse("high"), Some(T3Confidence::High));
    assert_eq!(T3Confidence::High.as_str(), "high");

    assert_eq!(
        T3RevocationState::parse("active"),
        Some(T3RevocationState::Active)
    );
    assert_eq!(T3RevocationState::Revoked.as_str(), "revoked");

    assert_eq!(
        ReviewGateState::parse("passed"),
        Some(ReviewGateState::Passed)
    );
    assert_eq!(PromotionDecisionState::Approved.as_str(), "approved");

    assert_eq!(
        CandidateReviewState::parse("pending"),
        Some(CandidateReviewState::Pending)
    );
    assert_eq!(OntologyCandidateState::Accepted.as_str(), "accepted");
}

#[test]
fn repository_projects_truth_layers_into_typed_records() {
    let path = fresh_db_path("typed-records");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());

    repo.insert_record(&sample_record("t1-record", TruthLayer::T1))
        .expect("t1 record should insert");
    repo.insert_record(&sample_record("t2-record", TruthLayer::T2))
        .expect("t2 record should insert");
    repo.insert_record(&sample_record("t3-record", TruthLayer::T3))
        .expect("t3 record should insert");

    let t1 = repo
        .get_truth_record("t1-record")
        .expect("truth record lookup should succeed")
        .expect("t1 truth record should exist");
    let t2 = repo
        .get_truth_record("t2-record")
        .expect("truth record lookup should succeed")
        .expect("t2 truth record should exist");
    let t3 = repo
        .get_truth_record("t3-record")
        .expect("truth record lookup should succeed")
        .expect("t3 truth record should exist");

    assert!(matches!(t1, TruthRecord::T1 { .. }));
    assert!(matches!(t2, TruthRecord::T2 { .. }));
    assert!(matches!(t3, TruthRecord::T3 { .. }));
}

#[test]
fn repository_persists_default_t3_governance_state() {
    let path = fresh_db_path("t3-defaults");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());

    repo.insert_record(&sample_record("t3-governed", TruthLayer::T3))
        .expect("t3 record should insert");

    let t3_state = repo
        .get_t3_state("t3-governed")
        .expect("t3 state lookup should succeed")
        .expect("t3 state should exist");

    assert_eq!(t3_state.confidence, T3Confidence::Medium);
    assert_eq!(t3_state.revocation_state, T3RevocationState::Active);
    assert_eq!(t3_state.record_id, "t3-governed");
}
