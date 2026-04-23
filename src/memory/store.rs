use std::{
    cell::RefCell,
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::memory::{
    dsl::{FactDslError, FactDslRecord, FlatFactDslRecordV1},
    taxonomy::{DomainV1, KindV1, TaxonomyPathV1, TopicV1},
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PersistedFactDslRecordV1 {
    pub record_id: String,
    pub payload: FlatFactDslRecordV1,
    pub classification_confidence: Option<f32>,
    pub needs_review: bool,
}

impl PersistedFactDslRecordV1 {
    pub fn new(
        record_id: impl Into<String>,
        payload: FlatFactDslRecordV1,
    ) -> Result<Self, FactDslStoreError> {
        let persisted = Self {
            record_id: record_id.into(),
            payload,
            classification_confidence: None,
            needs_review: false,
        };
        persisted.validate()?;
        Ok(persisted)
    }

    pub fn from_fact_dsl_record(
        record_id: impl Into<String>,
        record: &FactDslRecord,
    ) -> Result<Self, FactDslStoreError> {
        Self::new(record_id, record.flatten())
    }

    pub fn into_fact_dsl_record(self) -> Result<FactDslRecord, FactDslStoreError> {
        self.payload.into_record().map_err(FactDslStoreError::Dsl)
    }

    pub fn validate(&self) -> Result<(), FactDslStoreError> {
        if self.record_id.trim().is_empty() {
            return Err(FactDslStoreError::MissingRecordId);
        }
        if let Some(confidence) = self.classification_confidence {
            if !(0.0..=1.0).contains(&confidence) {
                return Err(FactDslStoreError::InvalidClassificationConfidence(
                    confidence,
                ));
            }
        }
        self.payload
            .clone()
            .into_record()
            .map_err(FactDslStoreError::Dsl)?;
        Ok(())
    }

    pub fn taxonomy_path(&self) -> Result<TaxonomyPathV1, FactDslStoreError> {
        TaxonomyPathV1::from_parts(
            &self.payload.domain,
            &self.payload.topic,
            &self.payload.aspect,
            &self.payload.kind,
        )
        .map_err(FactDslStoreError::Taxonomy)
    }
}

pub trait FactDslStore {
    fn put_fact_dsl(&self, persisted: &PersistedFactDslRecordV1) -> Result<(), FactDslStoreError>;

    fn get_fact_dsl(
        &self,
        record_id: &str,
    ) -> Result<Option<PersistedFactDslRecordV1>, FactDslStoreError>;

    fn list_fact_dsls(&self) -> Result<Vec<PersistedFactDslRecordV1>, FactDslStoreError>;

    fn delete_fact_dsl(
        &self,
        record_id: &str,
    ) -> Result<Option<PersistedFactDslRecordV1>, FactDslStoreError>;

    fn put_many_fact_dsls(
        &self,
        persisted: &[PersistedFactDslRecordV1],
    ) -> Result<(), FactDslStoreError> {
        for row in persisted {
            self.put_fact_dsl(row)?;
        }
        Ok(())
    }

    fn list_fact_dsls_by_domain(
        &self,
        domain: DomainV1,
    ) -> Result<Vec<PersistedFactDslRecordV1>, FactDslStoreError> {
        let rows = self.list_fact_dsls()?;
        Ok(rows
            .into_iter()
            .filter(|row| row.payload.domain == domain.as_str())
            .collect::<Vec<_>>())
    }

    fn list_fact_dsls_by_kind(
        &self,
        kind: KindV1,
    ) -> Result<Vec<PersistedFactDslRecordV1>, FactDslStoreError> {
        let rows = self.list_fact_dsls()?;
        Ok(rows
            .into_iter()
            .filter(|row| row.payload.kind == kind.as_str())
            .collect::<Vec<_>>())
    }

    fn list_fact_dsls_by_topic(
        &self,
        topic: TopicV1,
    ) -> Result<Vec<PersistedFactDslRecordV1>, FactDslStoreError> {
        let rows = self.list_fact_dsls()?;
        Ok(rows
            .into_iter()
            .filter(|row| row.payload.topic == topic.as_str())
            .collect::<Vec<_>>())
    }

    fn list_fact_dsls_by_path(
        &self,
        path: &TaxonomyPathV1,
    ) -> Result<Vec<PersistedFactDslRecordV1>, FactDslStoreError> {
        let rows = self.list_fact_dsls()?;
        Ok(rows
            .into_iter()
            .filter(|row| {
                row.payload.domain == path.domain.as_str()
                    && row.payload.topic == path.topic.as_str()
                    && row.payload.aspect == path.aspect.as_str()
                    && row.payload.kind == path.kind.as_str()
            })
            .collect::<Vec<_>>())
    }
}

#[derive(Debug, Default)]
pub struct InMemoryFactDslStore {
    rows: RefCell<BTreeMap<String, PersistedFactDslRecordV1>>,
}

impl InMemoryFactDslStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl FactDslStore for InMemoryFactDslStore {
    fn put_fact_dsl(&self, persisted: &PersistedFactDslRecordV1) -> Result<(), FactDslStoreError> {
        persisted.validate()?;
        self.rows
            .borrow_mut()
            .insert(persisted.record_id.clone(), persisted.clone());
        Ok(())
    }

    fn get_fact_dsl(
        &self,
        record_id: &str,
    ) -> Result<Option<PersistedFactDslRecordV1>, FactDslStoreError> {
        Ok(self.rows.borrow().get(record_id).cloned())
    }

    fn list_fact_dsls(&self) -> Result<Vec<PersistedFactDslRecordV1>, FactDslStoreError> {
        Ok(self.rows.borrow().values().cloned().collect())
    }

    fn delete_fact_dsl(
        &self,
        record_id: &str,
    ) -> Result<Option<PersistedFactDslRecordV1>, FactDslStoreError> {
        Ok(self.rows.borrow_mut().remove(record_id))
    }
}

#[derive(Debug, Clone)]
pub struct JsonFileFactDslStore {
    path: PathBuf,
}

impl JsonFileFactDslStore {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self { path: path.into() }
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    fn load_rows(&self) -> Result<BTreeMap<String, PersistedFactDslRecordV1>, FactDslStoreError> {
        match fs::read_to_string(&self.path) {
            Ok(contents) => {
                serde_json::from_str::<BTreeMap<String, PersistedFactDslRecordV1>>(&contents)
                    .map_err(FactDslStoreError::Json)
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
            Err(error) => Err(FactDslStoreError::Io(error)),
        }
    }

    fn save_rows(
        &self,
        rows: &BTreeMap<String, PersistedFactDslRecordV1>,
    ) -> Result<(), FactDslStoreError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(FactDslStoreError::Io)?;
        }
        let contents = serde_json::to_string_pretty(rows).map_err(FactDslStoreError::Json)?;
        fs::write(&self.path, contents).map_err(FactDslStoreError::Io)
    }
}

impl FactDslStore for JsonFileFactDslStore {
    fn put_fact_dsl(&self, persisted: &PersistedFactDslRecordV1) -> Result<(), FactDslStoreError> {
        persisted.validate()?;
        let mut rows = self.load_rows()?;
        rows.insert(persisted.record_id.clone(), persisted.clone());
        self.save_rows(&rows)
    }

    fn get_fact_dsl(
        &self,
        record_id: &str,
    ) -> Result<Option<PersistedFactDslRecordV1>, FactDslStoreError> {
        Ok(self.load_rows()?.remove(record_id))
    }

    fn list_fact_dsls(&self) -> Result<Vec<PersistedFactDslRecordV1>, FactDslStoreError> {
        Ok(self.load_rows()?.into_values().collect())
    }

    fn delete_fact_dsl(
        &self,
        record_id: &str,
    ) -> Result<Option<PersistedFactDslRecordV1>, FactDslStoreError> {
        let mut rows = self.load_rows()?;
        let removed = rows.remove(record_id);
        self.save_rows(&rows)?;
        Ok(removed)
    }
}

#[derive(Debug, Error)]
pub enum FactDslStoreError {
    #[error("missing persisted record id")]
    MissingRecordId,
    #[error("invalid persisted classification confidence: {0}")]
    InvalidClassificationConfidence(f32),
    #[error(transparent)]
    Dsl(#[from] FactDslError),
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Taxonomy(#[from] crate::memory::taxonomy::TaxonomyError),
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("{0}")]
    Store(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory::{
        dsl::{FactDslDraft, FactDslRecord},
        record::TruthLayer,
        taxonomy::{AspectV1, DomainV1, KindV1, TaxonomyPathV1, TopicV1},
    };
    use std::time::{SystemTime, UNIX_EPOCH};

    fn sample_record() -> FactDslRecord {
        FactDslRecord {
            taxonomy: TaxonomyPathV1::new(
                DomainV1::Project,
                TopicV1::Retrieval,
                AspectV1::Behavior,
                KindV1::Decision,
            )
            .expect("taxonomy path should be valid"),
            draft: FactDslDraft {
                claim: "use lexical-first as baseline".to_string(),
                why: Some("explainability matters".to_string()),
                time: Some("2026-04".to_string()),
                ..Default::default()
            },
            truth_layer: TruthLayer::T2,
            source_ref: "roadmap#phase9".to_string(),
        }
    }

    fn fresh_json_store_path(name: &str) -> PathBuf {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock before unix epoch")
            .as_nanos();
        std::env::temp_dir()
            .join("agent-memos-store-tests")
            .join(format!("{name}-{unique}.json"))
    }

    #[test]
    fn persisted_record_round_trips_to_fact_dsl_record() {
        let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_record())
            .expect("persisted wrapper should build");
        let rebuilt = persisted
            .clone()
            .into_fact_dsl_record()
            .expect("persisted wrapper should rebuild record");

        assert_eq!(persisted.record_id, "mem-1");
        assert_eq!(rebuilt, sample_record());
    }

    #[test]
    fn in_memory_store_persists_and_loads_fact_dsl_rows() {
        let store = InMemoryFactDslStore::new();
        let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_record())
            .expect("persisted wrapper should build");

        store
            .put_fact_dsl(&persisted)
            .expect("store should accept persisted rows");
        let loaded = store
            .get_fact_dsl("mem-1")
            .expect("store lookup should succeed")
            .expect("row should exist");

        assert_eq!(loaded, persisted);
    }

    #[test]
    fn in_memory_store_lists_persisted_rows() {
        let store = InMemoryFactDslStore::new();
        let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_record())
            .expect("persisted wrapper should build");

        store
            .put_fact_dsl(&persisted)
            .expect("store should accept persisted rows");
        let listed = store.list_fact_dsls().expect("listing should succeed");

        assert_eq!(listed, vec![persisted]);
    }

    #[test]
    fn json_file_store_persists_and_loads_rows() {
        let path = fresh_json_store_path("internal");
        let _ = fs::remove_file(&path);

        let store = JsonFileFactDslStore::new(&path);
        let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_record())
            .expect("persisted wrapper should build");

        store
            .put_fact_dsl(&persisted)
            .expect("json store should accept persisted rows");

        let loaded = store
            .get_fact_dsl("mem-1")
            .expect("json store lookup should succeed")
            .expect("row should exist");
        assert_eq!(loaded, persisted);

        let listed = store.list_fact_dsls().expect("listing should succeed");
        assert_eq!(listed, vec![persisted]);

        let _ = fs::remove_file(path);
    }

    #[test]
    fn persisted_row_exposes_taxonomy_path() {
        let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_record())
            .expect("persisted wrapper should build");

        let path = persisted
            .taxonomy_path()
            .expect("taxonomy path should parse");
        assert_eq!(path.domain, DomainV1::Project);
        assert_eq!(path.kind, KindV1::Decision);
    }

    #[test]
    fn stores_can_filter_rows_by_domain_and_kind() {
        let store = InMemoryFactDslStore::new();
        let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_record())
            .expect("persisted wrapper should build");

        store
            .put_fact_dsl(&persisted)
            .expect("store should persist");

        let by_domain = store
            .list_fact_dsls_by_domain(DomainV1::Project)
            .expect("domain filter should succeed");
        let by_kind = store
            .list_fact_dsls_by_kind(KindV1::Decision)
            .expect("kind filter should succeed");

        assert_eq!(by_domain, vec![persisted.clone()]);
        assert_eq!(by_kind, vec![persisted]);
    }

    #[test]
    fn in_memory_store_supports_deletion() {
        let store = InMemoryFactDslStore::new();
        let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_record())
            .expect("persisted wrapper should build");

        store
            .put_fact_dsl(&persisted)
            .expect("store should persist");
        let removed = store
            .delete_fact_dsl("mem-1")
            .expect("delete should succeed")
            .expect("row should be removed");
        assert_eq!(removed, persisted);
        assert!(
            store
                .get_fact_dsl("mem-1")
                .expect("lookup should succeed")
                .is_none()
        );
    }

    #[test]
    fn stores_can_filter_rows_by_topic_and_exact_path() {
        let store = InMemoryFactDslStore::new();
        let persisted = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_record())
            .expect("persisted wrapper should build");

        store
            .put_fact_dsl(&persisted)
            .expect("store should persist");

        let by_topic = store
            .list_fact_dsls_by_topic(TopicV1::Retrieval)
            .expect("topic filter should succeed");
        let by_path = store
            .list_fact_dsls_by_path(
                &TaxonomyPathV1::new(
                    DomainV1::Project,
                    TopicV1::Retrieval,
                    crate::memory::taxonomy::AspectV1::Behavior,
                    KindV1::Decision,
                )
                .expect("path should build"),
            )
            .expect("path filter should succeed");

        assert_eq!(by_topic, vec![persisted.clone()]);
        assert_eq!(by_path, vec![persisted]);
    }

    #[test]
    fn stores_can_persist_many_rows() {
        let store = InMemoryFactDslStore::new();
        let first = PersistedFactDslRecordV1::from_fact_dsl_record("mem-1", &sample_record())
            .expect("first wrapper should build");
        let second = PersistedFactDslRecordV1::from_fact_dsl_record("mem-2", &sample_record())
            .expect("second wrapper should build");

        store
            .put_many_fact_dsls(&[first.clone(), second.clone()])
            .expect("bulk persist should succeed");

        let listed = store.list_fact_dsls().expect("listing should succeed");
        assert_eq!(listed, vec![first, second]);
    }
}
