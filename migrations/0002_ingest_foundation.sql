ALTER TABLE memory_records ADD COLUMN chunk_index INTEGER;
ALTER TABLE memory_records ADD COLUMN chunk_count INTEGER;
ALTER TABLE memory_records ADD COLUMN chunk_anchor_json TEXT;
ALTER TABLE memory_records ADD COLUMN content_hash TEXT;
ALTER TABLE memory_records ADD COLUMN valid_from TEXT;
ALTER TABLE memory_records ADD COLUMN valid_to TEXT;

CREATE INDEX IF NOT EXISTS idx_memory_records_source_chunk_order
    ON memory_records(source_uri, chunk_index, recorded_at DESC);

CREATE INDEX IF NOT EXISTS idx_memory_records_validity_window
    ON memory_records(valid_from, valid_to);
