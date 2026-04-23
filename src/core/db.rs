use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use rusqlite::Connection;
use thiserror::Error;

use crate::core::migrations::apply_migrations;

static LIBSIMPLE_DICT_DIR: OnceLock<Result<PathBuf, String>> = OnceLock::new();

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
    #[error("failed to bootstrap lexical tokenizer support: {message}")]
    LexicalBootstrap { message: String },
    #[error("failed to configure lexical tokenizer dictionary")]
    LexicalDictionary {
        #[source]
        source: anyhow::Error,
    },
}

pub struct Database {
    conn: Connection,
    path: PathBuf,
}

impl Database {
    pub fn open(path: &Path) -> Result<Self, DbError> {
        if let Some(parent) = path
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
        {
            fs::create_dir_all(parent).map_err(|source| DbError::CreateDir {
                path: parent.to_path_buf(),
                source,
            })?;
        }

        let dict_dir = prepare_lexical_support()?;
        let mut conn = Connection::open(path).map_err(|source| DbError::Open {
            path: path.to_path_buf(),
            source,
        })?;
        libsimple::set_jieba_dict(&conn, dict_dir).map_err(|source| {
            DbError::LexicalDictionary {
                source: anyhow::Error::new(source)
                    .context(format!("dictionary dir: {}", dict_dir.display())),
            }
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

fn prepare_lexical_support() -> Result<&'static PathBuf, DbError> {
    let result = LIBSIMPLE_DICT_DIR.get_or_init(|| {
        libsimple::enable_auto_extension()
            .map_err(|error| format!("enable_auto_extension failed: {error}"))?;

        let dict_dir = std::env::temp_dir()
            .join("agent-memos")
            .join("libsimple-jieba");
        fs::create_dir_all(&dict_dir).map_err(|error| {
            format!("failed to create dict dir {}: {error}", dict_dir.display())
        })?;
        libsimple::release_jieba_dict(&dict_dir).map_err(|error| {
            format!(
                "release_jieba_dict failed for {}: {error}",
                dict_dir.display()
            )
        })?;

        Ok(dict_dir)
    });

    match result {
        Ok(path) => Ok(path),
        Err(message) => Err(DbError::LexicalBootstrap {
            message: message.clone(),
        }),
    }
}
