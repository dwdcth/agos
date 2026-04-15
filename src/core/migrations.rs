use rusqlite::Connection;
use rusqlite_migration::{M, Migrations};

const FOUNDATION_SCHEMA_SQL: &str = include_str!("../../migrations/0001_foundation.sql");
const INGEST_FOUNDATION_SQL: &str = include_str!("../../migrations/0002_ingest_foundation.sql");

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
    ])
}
