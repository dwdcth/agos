use std::path::Path;

use rusqlite::Connection;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn open(_path: &Path) -> anyhow::Result<Self> {
        anyhow::bail!("not implemented")
    }

    pub fn schema_version(&self) -> anyhow::Result<u32> {
        let _ = &self.conn;
        anyhow::bail!("not implemented")
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }
}
