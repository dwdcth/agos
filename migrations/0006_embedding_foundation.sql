CREATE TABLE IF NOT EXISTS record_embeddings (
    record_id TEXT PRIMARY KEY REFERENCES memory_records(id) ON DELETE CASCADE,
    backend TEXT NOT NULL,
    model TEXT NOT NULL,
    dimensions INTEGER NOT NULL,
    embedding_json TEXT NOT NULL,
    source_text_hash TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_record_embeddings_backend_model
    ON record_embeddings(backend, model);

CREATE TABLE IF NOT EXISTS record_embedding_index_state (
    index_name TEXT PRIMARY KEY,
    backend TEXT NOT NULL,
    model TEXT,
    state TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
