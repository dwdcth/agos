use rusqlite::Connection;

pub struct MemoryRepository<'db> {
    #[allow(dead_code)]
    conn: &'db Connection,
}

impl<'db> MemoryRepository<'db> {
    pub fn new(conn: &'db Connection) -> Self {
        Self { conn }
    }
}
