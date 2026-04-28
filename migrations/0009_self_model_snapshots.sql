CREATE TABLE IF NOT EXISTS self_model_snapshots (
    subject_ref TEXT PRIMARY KEY,
    snapshot_id TEXT NOT NULL,
    entries_json TEXT NOT NULL,
    compacted_through_updated_at TEXT NOT NULL,
    compacted_through_entry_id TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_self_model_snapshots_compaction_cursor
    ON self_model_snapshots(compacted_through_updated_at, compacted_through_entry_id);
