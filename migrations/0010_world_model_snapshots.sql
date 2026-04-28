CREATE TABLE IF NOT EXISTS world_model_snapshots (
    subject_ref TEXT NOT NULL,
    world_key TEXT NOT NULL,
    snapshot_id TEXT NOT NULL,
    fragments_json TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_world_model_snapshots_subject_scope
    ON world_model_snapshots(subject_ref, world_key);
