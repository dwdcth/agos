use std::{
    fs,
    path::{Path, PathBuf},
};

use rusqlite::Connection;
use thiserror::Error;

use crate::core::migrations::apply_migrations;

#[derive(Debug, Error)]
pub enum DbError {
    #[error("failed to create database directory for {path}")]
    CreateDir {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to open sqlite database at {path}")]
    Open {
        path: PathBuf,
        #[source]
        source: rusqlite::Error,
    },
    #[error("failed to read schema version for {path}")]
    SchemaVersion {
        path: PathBuf,
        #[source]
        source: rusqlite::Error,
    },
    #[error("failed to apply database migrations")]
    Migration(#[from] rusqlite_migration::Error),
}

pub struct Database {
    conn: Connection,
    path: PathBuf,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        if let Some(parent) = path.parent().filter(|parent| !parent.as_os_str().is_empty()) {
            fs::create_dir_all(parent).map_err(|source| DbError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let mut conn = Connection::open(path).map_err(|source| DbError::Open {
            path: path.to_path_buf(),
            source,
        })?;
        apply_migrations(&mut conn)?;

        Ok(Self {
            conn,
            path: path.to_path_buf(),
        })
    }

    pub fn schema_version(&self) -> Result<u32, DbError> {
        self.conn
            .query_row("PRAGMA user_version;", [], |row| row.get(0))
            .map_err(|source| DbError::SchemaVersion {
                path: self.path.clone(),
                source,
            })
    }

    pub fn conn(&self) -> &Connection {
        &self.conn
    }

    pub fn path(&self) -> &Path {
        &self.path
    }
}
