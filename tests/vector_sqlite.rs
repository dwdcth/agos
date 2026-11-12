//! sqlite-vec vector index integration: dual-write on ingest, KNN recall,
//! and the documented fallback when the index is unusable.

use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::config::{
        Config, EmbeddingBackend, EmbeddingConfig, RetrievalConfig, RetrievalMode,
        RootVectorConfig, VectorBackend,
    },
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::record::{RecordType, Scope, TruthLayer},
    memory::vector::VectorIndex,
    search::{QueryStrategy, SearchRequest, SearchService},
};

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-vector-sqlite-tests")
        .join(format!("{name}-{unique}"))
        .join("vector-sqlite.sqlite")
}

fn builtin_vector_config(db_path: &std::path::Path, mode: RetrievalMode) -> Config {
    Config {
        db_path: db_path.display().to_string(),
        retrieval: RetrievalConfig { mode },
        embedding: EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some("hash-16".to_string()),
            ..Default::default()
        },
        vector: RootVectorConfig {
            backend: VectorBackend::SqliteVec,
            table: "object_embeddings_vec".to_string(),
            similarity: "cosine".to_string(),
        },
        ..Default::default()
    }
}

fn ingest(service: &IngestService<'_>, uri: &str, content: &str) {
    service
        .ingest(IngestRequest {
            source_uri: uri.to_string(),
            source_label: Some(uri.to_string()),
            source_kind: None,
            content: content.to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-11-12T10:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");
}

#[test]
fn vector_index_roundtrips_upsert_and_knn() {
    let path = fresh_db_path("roundtrip");
    let db = Database::open(&path).expect("database should open");
    let index = VectorIndex::new(db.conn(), "object_embeddings_vec");

    assert!(
        index.extension_available(),
        "sqlite-vec extension registers on Database::open"
    );
    assert!(!index.table_exists(), "table starts absent");

    index.ensure_table(4).expect("vec0 table should create");
    index
        .upsert("record-a", &[0.1, 0.2, 0.3, 0.4])
        .expect("upsert a");
    index
        .upsert("record-b", &[0.9, 0.05, 0.05, 0.05])
        .expect("upsert b");
    // Re-upsert replaces the vector instead of duplicating the row (vec0 has
    // no UPSERT support; the index implements delete+insert).
    index
        .upsert("record-a", &[0.1, 0.2, 0.31, 0.4])
        .expect("re-upsert a");

    let rows: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM object_embeddings_vec", [], |row| {
            row.get(0)
        })
        .expect("count vec0 rows");
    assert_eq!(rows, 2, "re-upsert must not duplicate rows");

    let neighbors = index.knn(&[0.1, 0.2, 0.3, 0.4], 2).expect("knn");
    assert_eq!(neighbors.len(), 2);
    assert_eq!(neighbors[0].0, "record-a", "nearest neighbor wins");
    assert!(
        neighbors[0].1 > 0.99,
        "cosine similarity near 1 for the aligned vector: {}",
        neighbors[0].1
    );

    // Dimension mismatch surfaces as a query error the search path maps to
    // the JSON-scan fallback.
    let mismatch = index.knn(&[0.5; 8], 1);
    assert!(
        mismatch.is_err(),
        "8-dim query against a 4-dim table must fail loudly"
    );
}

#[test]
fn ingest_dual_writes_vectors_into_the_vec0_table() {
    let path = fresh_db_path("dual-write");
    let config = builtin_vector_config(&path, RetrievalMode::Hybrid);
    let db = Database::open(&path).expect("database should open");

    let service = IngestService::with_config(db.conn(), &config);
    ingest(
        &service,
        "memo://project/vec-dual",
        "sqlite vec index mirrors ingested chunk vectors",
    );

    let vec_rows: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM object_embeddings_vec", [], |row| {
            row.get(0)
        })
        .expect("vec0 table should exist");
    assert_eq!(vec_rows, 1, "the chunk vector lands in the vec0 index");

    let json_rows: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM record_embeddings", [], |row| {
            row.get(0)
        })
        .expect("record_embeddings rows");
    assert_eq!(json_rows, 1, "the JSON source-of-truth row stays");
}

#[test]
fn hybrid_recall_survives_index_loss_through_the_json_fallback() {
    let path = fresh_db_path("index-loss");
    let config = builtin_vector_config(&path, RetrievalMode::Hybrid);
    let db = Database::open(&path).expect("database should open");

    let service = IngestService::with_config(db.conn(), &config);
    ingest(
        &service,
        "memo://project/vec-loss",
        "fallback scan keeps recall while the index rebuilds",
    );

    let with_index = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("fallback scan keeps recall"))
        .expect("indexed hybrid search should succeed");
    assert!(
        with_index.results.iter().any(|result| result
            .trace
            .query_strategies
            .contains(&QueryStrategy::Embedding)),
        "vec0-backed recall should surface the embedding strategy"
    );

    // Destroy the index (as a model switch or corruption would) — recall must
    // degrade to the JSON cosine scan, not fail.
    db.conn()
        .execute("DROP TABLE object_embeddings_vec", [])
        .expect("drop vec0 table");

    let fallback = SearchService::with_runtime_config(db.conn(), &config, None)
        .search(&SearchRequest::new("fallback scan keeps recall"))
        .expect("hybrid search must survive index loss");
    assert!(
        fallback.results.iter().any(|result| result
            .trace
            .query_strategies
            .contains(&QueryStrategy::Embedding)),
        "the JSON scan fallback keeps the embedding channel alive"
    );
}

#[test]
fn backfill_mirrors_pre_existing_embeddings_into_the_index() {
    let path = fresh_db_path("backfill");
    let db = Database::open(&path).expect("database should open");

    // Ingest WITHOUT a vector table (legacy rows only in JSON).
    let legacy_service = IngestService::new(db.conn());
    ingest(
        &legacy_service,
        "memo://project/legacy",
        "legacy row predates the vec0 index and needs a backfill pass",
    );
    // No embedding backend configured -> no embeddings at all; add one with
    // the builtin backend but still without the index.
    let config = builtin_vector_config(&path, RetrievalMode::LexicalOnly);
    let json_only_service = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        config.embedding.clone(),
    );
    ingest(
        &json_only_service,
        "memo://project/json-only",
        "json only row with a builtin embedding vector",
    );

    let vec_rows_before: i64 = db
        .conn()
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type = 'table' AND name = 'object_embeddings_vec'",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);
    assert_eq!(vec_rows_before, 0, "no vec0 table before the backfill");

    // The maintenance pass (ruminate) backfills through a vector-aware
    // repository view of the same database.
    let repository = agent_memos::memory::repository::MemoryRepository::with_vector_table(
        db.conn(),
        Some("object_embeddings_vec"),
    );
    let mirrored = repository
        .backfill_vector_index()
        .expect("backfill should succeed");
    assert_eq!(mirrored, 1, "only the row with an embedding gets mirrored");

    let vec_rows_after: i64 = db
        .conn()
        .query_row("SELECT COUNT(*) FROM object_embeddings_vec", [], |row| {
            row.get(0)
        })
        .expect("vec0 rows after backfill");
    assert_eq!(vec_rows_after, 1);

    // Idempotent: a second pass mirrors nothing new.
    let mirrored_again = repository
        .backfill_vector_index()
        .expect("second backfill should succeed");
    assert_eq!(mirrored_again, 0, "backfill is idempotent");
}
