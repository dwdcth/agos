CREATE TABLE IF NOT EXISTS memory_records (
    id TEXT PRIMARY KEY,
    source_uri TEXT NOT NULL,
    source_kind TEXT NOT NULL,
    source_label TEXT,
    recorded_at TEXT NOT NULL,
    scope TEXT NOT NULL,
    record_type TEXT NOT NULL,
    truth_layer TEXT NOT NULL,
    provenance_json TEXT NOT NULL,
    content_text TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_memory_records_scope_recorded_at
    ON memory_records(scope, recorded_at DESC);

CREATE INDEX IF NOT EXISTS idx_memory_records_truth_layer_recorded_at
    ON memory_records(truth_layer, recorded_at DESC);
