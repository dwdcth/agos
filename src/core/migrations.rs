use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

const FOUNDATION_SCHEMA_SQL: &str = include_str!("../../migrations/0001_foundation.sql");
const INGEST_FOUNDATION_SQL: &str = include_str!("../../migrations/0002_ingest_foundation.sql");
const LEXICAL_SIDECAR_SQL: &str = include_str!("../../migrations/0003_lexical_sidecar.sql");
const TRUTH_LAYER_GOVERNANCE_SQL: &str =
    include_str!("../../migrations/0004_truth_layer_governance.sql");
const RUMINATION_WRITEBACK_SQL: &str =
    include_str!("../../migrations/0005_rumination_writeback.sql");
const EMBEDDING_FOUNDATION_SQL: &str =
    include_str!("../../migrations/0006_embedding_foundation.sql");
const LAYERED_MEMORY_DSL_SQL: &str = include_str!("../../migrations/0007_layered_memory_dsl.sql");
const LAYERED_MEMORY_REVIEW_METADATA_SQL: &str =
    include_str!("../../migrations/0008_layered_memory_review_metadata.sql");

pub fn apply_migrations(conn: &mut Connection) -> Result<(), rusqlite_migration::Error> {
    migrations().to_latest(conn)
}

fn migrations() -> Migrations<'static> {
    Migrations::new(vec![
        M::up(FOUNDATION_SCHEMA_SQL)
            .foreign_key_check()
            .comment("foundation schema bootstrap"),
        M::up(INGEST_FOUNDATION_SQL)
            .foreign_key_check()
            .comment("ingest authority metadata"),
        M::up(LEXICAL_SIDECAR_SQL)
            .foreign_key_check()
            .comment("lexical fts sidecar"),
        M::up(TRUTH_LAYER_GOVERNANCE_SQL)
            .foreign_key_check()
            .comment("truth governance"),
        M::up(RUMINATION_WRITEBACK_SQL)
            .foreign_key_check()
            .comment("rumination and adaptive write-back"),
        M::up(EMBEDDING_FOUNDATION_SQL)
            .foreign_key_check()
            .comment("embedding backend and vector sidecars"),
        M::up(LAYERED_MEMORY_DSL_SQL)
            .foreign_key_check()
            .comment("layered memory taxonomy and dsl records"),
        M::up(LAYERED_MEMORY_REVIEW_METADATA_SQL)
            .foreign_key_check()
            .comment("layered memory review metadata"),
    ])
}
