CREATE TABLE IF NOT EXISTS fact_dsl_records (
    record_id TEXT PRIMARY KEY REFERENCES memory_records(id) ON DELETE CASCADE,
    domain TEXT NOT NULL,
    topic TEXT NOT NULL,
    aspect TEXT NOT NULL,
    kind TEXT NOT NULL,
    claim TEXT NOT NULL,
    truth_layer TEXT NOT NULL,
    source_ref TEXT NOT NULL,
    why TEXT,
    time_hint TEXT,
    cond TEXT,
    impact TEXT,
    conf REAL,
    rel_json TEXT
);

CREATE INDEX IF NOT EXISTS idx_fact_dsl_records_domain_topic
    ON fact_dsl_records(domain, topic);

CREATE INDEX IF NOT EXISTS idx_fact_dsl_records_kind
    ON fact_dsl_records(kind);

CREATE INDEX IF NOT EXISTS idx_fact_dsl_records_source_ref
    ON fact_dsl_records(source_ref);
