use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    core::{
        config::{
            EmbeddingBackend, EmbeddingConfig, RetrievalMode, RootRuntimeConfig, VectorBackend,
        },
        db::Database,
    },
    ingest::{IngestRequest, IngestService},
    memory::record::{RecordType, Scope, TruthLayer},
    search::{ChannelContribution, QueryStrategy, SearchRequest, SearchService},
};

#[test]
fn parses_root_config_into_dual_channel_variants() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config.toml should parse for retrieval tests");

    assert_eq!(config.store.backend, "sqlite");
    assert_eq!(config.store.sqlite_path, "~/.memos/memos.db");
    assert_eq!(config.llm.provider, "openai");
    assert_eq!(config.llm.model, "deepseek-ai/DeepSeek-V3.2");
    assert_eq!(config.embedding.provider, "openai");
    assert_eq!(config.embedding.model, "BAAI/bge-m3");
    assert_eq!(config.embedding.dimensions, Some(1024));
    assert_eq!(config.vector.backend, VectorBackend::SqliteVec);
    assert_eq!(config.vector.table, "object_embeddings_vec");
    assert_eq!(config.vector.similarity, "cosine");
}

#[test]
fn generated_mode_matrix_covers_lexical_embedding_and_hybrid() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config.toml should parse for retrieval tests");

    let variants = config.retrieval_mode_variants();
    assert_eq!(variants.len(), 3);

    assert_eq!(variants[0].name, "lexical_only");
    assert_eq!(variants[0].mode, RetrievalMode::LexicalOnly);
    assert_eq!(variants[0].embedding_backend, EmbeddingBackend::Disabled);

    assert_eq!(variants[1].name, "embedding_only");
    assert_eq!(variants[1].mode, RetrievalMode::EmbeddingOnly);
    assert_eq!(variants[1].embedding_backend, EmbeddingBackend::Builtin);
    assert_eq!(
        variants[1]
            .embedding
            .as_ref()
            .expect("embedding config should carry through")
            .model,
        "BAAI/bge-m3"
    );
    assert_eq!(
        variants[1]
            .vector
            .as_ref()
            .expect("vector config should carry through")
            .backend,
        VectorBackend::SqliteVec
    );

    assert_eq!(variants[2].name, "hybrid");
    assert_eq!(variants[2].mode, RetrievalMode::Hybrid);
    assert_eq!(variants[2].embedding_backend, EmbeddingBackend::Builtin);
}

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-dual-channel-tests")
        .join(format!("{name}-{unique}"))
        .join("dual-channel.sqlite")
}

fn ingest_record(service: &IngestService<'_>, source_uri: &str, content: &str, recorded_at: &str) {
    service
        .ingest(IngestRequest {
            source_uri: source_uri.to_string(),
            source_label: Some(source_uri.to_string()),
            source_kind: None,
            content: content.to_string(),
            scope: Scope::Project,
            record_type: RecordType::Observation,
            truth_layer: TruthLayer::T2,
            recorded_at: recorded_at.to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");
}

#[test]
fn mode_specific_search_behaviors_match_generated_configs() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config.toml should parse for retrieval tests");
    let variants = config.retrieval_mode_variants();

    let db_path = fresh_db_path("mode-specific");
    let db = Database::open(&db_path).expect("database should open");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some(
                variants[1]
                    .embedding
                    .as_ref()
                    .expect("embedding config")
                    .model
                    .clone(),
            ),
            endpoint: None,
            api_key: None,
        },
    );

    ingest_record(
        &ingest,
        "memo://project/lexical",
        "lexical recall keeps citations and readable snippets",
        "2026-04-16T10:00:00Z",
    );
    ingest_record(
        &ingest,
        "memo://project/vector",
        "semantic similarity catches rewritten retrieval meaning",
        "2026-04-16T10:05:00Z",
    );

    let lexical_results = SearchService::with_variant(db.conn(), &variants[0])
        .search(&SearchRequest::new("citations snippets"))
        .expect("lexical-only search should succeed");
    assert!(
        lexical_results.results.iter().all(|result| {
            !result
                .trace
                .query_strategies
                .contains(&QueryStrategy::Embedding)
        }),
        "lexical-only mode should not surface embedding query strategy"
    );

    let embedding_results = SearchService::with_variant(db.conn(), &variants[1])
        .search(&SearchRequest::new("retrieval meaning"))
        .expect("embedding-only search should succeed");
    assert!(
        embedding_results
            .results
            .iter()
            .all(|result| result.trace.query_strategies == vec![QueryStrategy::Embedding]),
        "embedding-only mode should surface embedding-backed results only"
    );

    let hybrid_results = SearchService::with_variant(db.conn(), &variants[2])
        .search(&SearchRequest::new("retrieval meaning"))
        .expect("hybrid search should succeed");
    assert!(
        hybrid_results.results.iter().any(|result| result
            .trace
            .query_strategies
            .contains(&QueryStrategy::Embedding)),
        "hybrid mode should preserve embedding contribution in the result trace"
    );
}

#[test]
fn hybrid_search_merges_lexical_and_embedding_candidates_by_record_identity() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config.toml should parse for retrieval tests");
    let hybrid = config
        .retrieval_mode_variants()
        .into_iter()
        .find(|variant| variant.mode == RetrievalMode::Hybrid)
        .expect("hybrid variant should exist");

    let db_path = fresh_db_path("hybrid-dedupe");
    let db = Database::open(&db_path).expect("database should open");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some(
                hybrid
                    .embedding
                    .as_ref()
                    .expect("embedding config")
                    .model
                    .clone(),
            ),
            endpoint: None,
            api_key: None,
        },
    );

    ingest_record(
        &ingest,
        "memo://project/shared",
        "retrieval fusion semantic retrieval fusion citations",
        "2026-04-16T11:00:00Z",
    );

    let results = SearchService::with_variant(db.conn(), &hybrid)
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("hybrid search should succeed");

    let shared = results
        .results
        .iter()
        .filter(|result| result.record.source.uri == "memo://project/shared")
        .collect::<Vec<_>>();
    assert_eq!(
        shared.len(),
        1,
        "hybrid mode should dedupe lexical and embedding hits for the same record identity"
    );
    assert!(
        shared[0]
            .trace
            .query_strategies
            .contains(&QueryStrategy::Embedding),
        "deduped result should preserve embedding contribution"
    );
}

#[test]
fn result_trace_reports_channel_contribution() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config.toml should parse for retrieval tests");
    let variants = config.retrieval_mode_variants();

    let db_path = fresh_db_path("trace-contribution");
    let db = Database::open(&db_path).expect("database should open");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some(
                variants[1]
                    .embedding
                    .as_ref()
                    .expect("embedding config")
                    .model
                    .clone(),
            ),
            endpoint: None,
            api_key: None,
        },
    );

    ingest_record(
        &ingest,
        "memo://project/shared-trace",
        "retrieval fusion semantic retrieval fusion citations",
        "2026-04-16T12:00:00Z",
    );

    let lexical = SearchService::with_variant(db.conn(), &variants[0])
        .search(&SearchRequest::new("citations"))
        .expect("lexical-only search should succeed");
    let embedding = SearchService::with_variant(db.conn(), &variants[1])
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("embedding-only search should succeed");
    let hybrid = SearchService::with_variant(db.conn(), &variants[2])
        .search(&SearchRequest::new("retrieval fusion"))
        .expect("hybrid search should succeed");

    assert_eq!(
        lexical.results[0].trace.channel_contribution,
        ChannelContribution::LexicalOnly
    );
    assert_eq!(
        embedding.results[0].trace.channel_contribution,
        ChannelContribution::EmbeddingOnly
    );
    assert_eq!(
        hybrid.results[0].trace.channel_contribution,
        ChannelContribution::Hybrid
    );
}
