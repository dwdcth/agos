use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::db::Database,
    memory::{
        dsl::{FactDslDraft, FactDslRecord},
        record::{
            MemoryRecord, Provenance, RecordTimestamp, RecordType, Scope, SourceKind, SourceRef,
            TruthLayer, ValidityWindow,
        },
        repository::MemoryRepository,
        store::{FactDslStore, PersistedFactDslRecordV1},
        taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
    },
};

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-layered-repo-tests")
        .join(format!("{name}-{unique}"))
        .join("memory.sqlite")
}

fn sample_memory_record() -> MemoryRecord {
    MemoryRecord {
        id: "mem-1".to_string(),
        source: SourceRef {
            uri: "memo://project/retrieval".to_string(),
            kind: SourceKind::Note,
            label: Some("retrieval note".to_string()),
        },
        timestamp: RecordTimestamp {
            recorded_at: "2026-04-19T00:00:00Z".to_string(),
            created_at: "2026-04-19T00:00:00Z".to_string(),
            updated_at: "2026-04-19T00:00:00Z".to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Observation,
        truth_layer: TruthLayer::T2,
        provenance: Provenance {
            origin: "test".to_string(),
            imported_via: None,
            derived_from: Vec::new(),
        },
        content_text: "Use lexical-first as baseline because explainability matters.".to_string(),
        chunk: None,
        validity: ValidityWindow::default(),
    }
}

fn sample_dsl_record() -> FactDslRecord {
    FactDslRecord {
        taxonomy: TaxonomyPathV1::new(
            DomainV1::Project,
            TopicV1::Retrieval,
            AspectV1::Behavior,
            KindV1::Decision,
        )
        .expect("taxonomy path should be valid"),
        draft: FactDslDraft {
            claim: "use lexical-first as baseline".to_string(),
            why: Some("explainability matters".to_string()),
            time: Some("2026-04".to_string()),
            ..Default::default()
        },
        truth_layer: TruthLayer::T2,
        source_ref: "roadmap#phase9".to_string(),
    }
}

#[test]
fn sqlite_repository_exposes_layered_record_views() {
    let path = fresh_db_path("layered-view");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());
    let record = sample_memory_record();
    repo.insert_record(&record)
        .expect("authority record should insert");

    let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_dsl_record())
        .expect("persisted wrapper should build");
    repo.put_fact_dsl(&persisted)
        .expect("repository should persist fact DSL");

    let layered = repo
        .get_layered_record("mem-1")
        .expect("layered lookup should succeed")
        .expect("layered record should exist");
    assert_eq!(layered.record.id, "mem-1");
    assert_eq!(
        layered
            .dsl
            .as_ref()
            .expect("dsl sidecar should exist")
            .payload
            .topic,
        "retrieval"
    );

    let listed = repo
        .list_layered_records()
        .expect("layered listing should succeed");
    assert_eq!(listed.len(), 1);
    assert_eq!(listed[0], layered);

    let batch = repo
        .list_layered_records_for_ids(&["mem-1".to_string(), "missing".to_string()])
        .expect("batch layered lookup should succeed");
    assert_eq!(batch, vec![layered]);
}
