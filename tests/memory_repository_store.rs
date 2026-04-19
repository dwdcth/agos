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
        .join("agent-memos-memory-store-tests")
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
fn sqlite_repository_implements_fact_dsl_store_contract() {
    let path = fresh_db_path("dsl-store");
    let db = Database::open(&path).expect("database should open");
    assert_eq!(db.schema_version().expect("schema version"), 7);

    let repo = MemoryRepository::new(db.conn());
    repo.insert_record(&sample_memory_record())
        .expect("authority record should insert");

    let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_dsl_record())
        .expect("persisted wrapper should build");

    repo.put_fact_dsl(&persisted)
        .expect("repository should persist fact DSL");

    let loaded = repo
        .get_fact_dsl("mem-1")
        .expect("lookup should succeed")
        .expect("row should exist");
    assert_eq!(loaded, persisted);

    let listed = repo.list_fact_dsls().expect("listing should succeed");
    assert_eq!(listed, vec![persisted.clone()]);

    let by_topic = repo
        .list_fact_dsls_by_topic(TopicV1::Retrieval)
        .expect("topic filter should succeed");
    assert_eq!(by_topic, vec![persisted.clone()]);

    let by_path = repo
        .list_fact_dsls_by_path(&sample_dsl_record().taxonomy)
        .expect("path filter should succeed");
    assert_eq!(by_path, vec![persisted.clone()]);

    let removed = repo
        .delete_fact_dsl("mem-1")
        .expect("delete should succeed")
        .expect("row should exist");
    assert_eq!(removed, persisted);
    assert!(repo.get_fact_dsl("mem-1").expect("lookup should succeed").is_none());
}

#[test]
fn sqlite_repository_fact_dsl_rows_follow_memory_record_lifecycle() {
    let path = fresh_db_path("dsl-cascade");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());
    let record = sample_memory_record();
    repo.insert_record(&record)
        .expect("authority record should insert");

    let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_dsl_record())
        .expect("persisted wrapper should build");
    repo.put_fact_dsl(&persisted)
        .expect("repository should persist fact DSL");

    db.conn()
        .execute("DELETE FROM memory_records WHERE id = ?1", [&record.id])
        .expect("authority row delete should succeed");

    assert!(
        repo.get_fact_dsl("mem-1")
            .expect("lookup should succeed")
            .is_none(),
        "fact DSL rows should cascade when the authority record is deleted"
    );
}

#[test]
fn sqlite_repository_fact_dsl_store_upserts_existing_rows() {
    let path = fresh_db_path("dsl-upsert");
    let db = Database::open(&path).expect("database should open");
    let repo = MemoryRepository::new(db.conn());
    repo.insert_record(&sample_memory_record())
        .expect("authority record should insert");

    let mut first = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_dsl_record())
        .expect("persisted wrapper should build");
    repo.put_fact_dsl(&first)
        .expect("initial persist should succeed");

    first.payload.why = Some("updated rationale".to_string());
    first.payload.impact = Some("updated impact".to_string());
    repo.put_fact_dsl(&first)
        .expect("upsert should succeed");

    let loaded = repo
        .get_fact_dsl("mem-1")
        .expect("lookup should succeed")
        .expect("row should exist");
    assert_eq!(loaded.payload.why.as_deref(), Some("updated rationale"));
    assert_eq!(loaded.payload.impact.as_deref(), Some("updated impact"));
    assert_eq!(repo.list_fact_dsls().expect("listing should succeed").len(), 1);
}
