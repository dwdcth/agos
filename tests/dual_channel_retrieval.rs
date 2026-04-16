use std::path::PathBuf;

use agent_memos::core::config::{
    EmbeddingBackend, RetrievalMode, RootRuntimeConfig, VectorBackend,
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
