-- 跨主题关联层（图关系存储）：record_id <-> record_id 的有向边，
-- relation 开放字符串（derived_from / relates_to / supports / contradicts…），
-- 版本化语义与 memory_records 一致（valid_from/valid_to 窗口，NULL = 开放）。
-- provenance 记录边的来源（provenance_backfill / rumination / manual…），
-- 与 (source, target, relation) 一起构成唯一键：同一来源对同一对记录的同
-- 关系边只存一行，重放可幂等 upsert。
CREATE TABLE IF NOT EXISTS memory_links (
    link_id TEXT PRIMARY KEY,
    source_record_id TEXT NOT NULL REFERENCES memory_records(id) ON DELETE CASCADE,
    target_record_id TEXT NOT NULL REFERENCES memory_records(id) ON DELETE CASCADE,
    relation TEXT NOT NULL,
    weight REAL NOT NULL DEFAULT 1.0,
    valid_from TEXT,
    valid_to TEXT,
    provenance TEXT NOT NULL,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    UNIQUE(source_record_id, target_record_id, relation, provenance)
);

CREATE INDEX IF NOT EXISTS idx_memory_links_source_relation
    ON memory_links(source_record_id, relation, valid_to);

CREATE INDEX IF NOT EXISTS idx_memory_links_target_relation
    ON memory_links(target_record_id, relation, valid_to);
