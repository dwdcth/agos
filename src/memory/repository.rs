use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;

use crate::core::config::EmbeddingBackend;
use crate::memory::record::{
    ChunkAnchor, ChunkMetadata, MemoryRecord, Provenance, RecordTimestamp, RecordType, Scope,
    SourceKind, SourceRef, TruthLayer, ValidityWindow,
};
use crate::memory::store::{FactDslStore, FactDslStoreError, PersistedFactDslRecordV1};
use crate::memory::truth::{
    CandidateReviewState, EvidenceRole, OntologyCandidate, OntologyCandidateState,
    PromotionDecisionState, PromotionEvidence, PromotionReview, ReviewGateState, T3Confidence,
    T3RevocationState, T3State, TruthRecord,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScopeCount {
    pub scope: Scope,
    pub count: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LayeredMemoryRecord {
    pub record: MemoryRecord,
    pub dsl: Option<PersistedFactDslRecordV1>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RecordEmbedding {
    pub record_id: String,
    pub backend: EmbeddingBackend,
    pub model: String,
    pub dimensions: u32,
    pub embedding: Vec<f32>,
    pub source_text_hash: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuminationQueueStatus {
    Queued,
    Claimed,
    Completed,
    Failed,
    Cancelled,
}

impl RuminationQueueStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Claimed => "claimed",
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Cancelled => "cancelled",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "queued" => Some(Self::Queued),
            "claimed" => Some(Self::Claimed),
            "completed" => Some(Self::Completed),
            "failed" => Some(Self::Failed),
            "cancelled" => Some(Self::Cancelled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct PersistedRuminationQueueItem {
    pub queue_tier: String,
    pub item_id: String,
    pub trigger_kind: String,
    pub status: RuminationQueueStatus,
    pub subject_ref: String,
    pub dedupe_key: String,
    pub cooldown_key: String,
    pub budget_bucket: String,
    pub priority: i64,
    pub budget_cost: u32,
    pub attempt_count: u32,
    pub cooldown_until: Option<String>,
    pub next_eligible_at: String,
    pub payload_json: Value,
    pub evidence_refs_json: Option<Vec<String>>,
    pub source_report_json: Option<Value>,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub processed_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistedRuminationTriggerState {
    pub queue_tier: String,
    pub trigger_kind: String,
    pub dedupe_key: String,
    pub cooldown_key: String,
    pub budget_bucket: String,
    pub budget_window_started_at: Option<String>,
    pub budget_spent: u32,
    pub cooldown_until: Option<String>,
    pub last_enqueued_at: Option<String>,
    pub last_seen_at: String,
    pub last_decision: String,
    pub last_item_id: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum LocalAdaptationTargetKind {
    SelfState,
    RiskBoundary,
    PrivateT3,
}

impl LocalAdaptationTargetKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::SelfState => "self_state",
            Self::RiskBoundary => "risk_boundary",
            Self::PrivateT3 => "private_t3",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "self_state" => Some(Self::SelfState),
            "risk_boundary" => Some(Self::RiskBoundary),
            "private_t3" => Some(Self::PrivateT3),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LocalAdaptationPayload {
    pub value: Value,
    pub trigger_kind: String,
    pub evidence_refs: Vec<String>,
}

impl LocalAdaptationPayload {
    pub fn display_value(&self) -> String {
        match &self.value {
            Value::String(value) => value.clone(),
            other => other.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LocalAdaptationEntry {
    pub entry_id: String,
    pub subject_ref: String,
    pub target_kind: LocalAdaptationTargetKind,
    pub key: String,
    pub payload: LocalAdaptationPayload,
    pub source_queue_item_id: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuminationCandidateKind {
    PromotionCandidate,
    SkillTemplate,
    ValueAdjustmentCandidate,
}

impl RuminationCandidateKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::PromotionCandidate => "promotion_candidate",
            Self::SkillTemplate => "skill_template",
            Self::ValueAdjustmentCandidate => "value_adjustment_candidate",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "promotion_candidate" => Some(Self::PromotionCandidate),
            "skill_template" => Some(Self::SkillTemplate),
            "value_adjustment_candidate" => Some(Self::ValueAdjustmentCandidate),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuminationCandidateStatus {
    Pending,
    Consumed,
    Rejected,
    Archived,
}

impl RuminationCandidateStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Consumed => "consumed",
            Self::Rejected => "rejected",
            Self::Archived => "archived",
        }
    }

    pub fn parse(value: &str) -> Option<Self> {
        match value {
            "pending" => Some(Self::Pending),
            "consumed" => Some(Self::Consumed),
            "rejected" => Some(Self::Rejected),
            "archived" => Some(Self::Archived),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RuminationCandidate {
    pub candidate_id: String,
    pub source_queue_item_id: Option<String>,
    pub candidate_kind: RuminationCandidateKind,
    pub subject_ref: String,
    pub payload: Value,
    pub evidence_refs: Vec<String>,
    pub governance_ref_id: Option<String>,
    pub status: RuminationCandidateStatus,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Error)]
pub enum RepositoryError {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("invalid {field} stored in database: {value}")]
    InvalidEnum { field: &'static str, value: String },
    #[error("incomplete chunk metadata stored for record {record_id}")]
    IncompleteChunkMetadata { record_id: String },
    #[error("missing t3 governance state for record {record_id}")]
    MissingT3State { record_id: String },
}

pub struct MemoryRepository<'db> {
    conn: &'db Connection,
}

impl<'db> MemoryRepository<'db> {
    pub fn new(conn: &'db Connection) -> Self {
        Self { conn }
    }

    pub fn insert_record(&self, record: &MemoryRecord) -> Result<(), RepositoryError> {
        let provenance_json = serde_json::to_string(&record.provenance)?;
        let chunk_index = record.chunk.as_ref().map(|chunk| chunk.chunk_index);
        let chunk_count = record.chunk.as_ref().map(|chunk| chunk.chunk_count);
        let chunk_anchor_json = record
            .chunk
            .as_ref()
            .map(|chunk| serde_json::to_string(&chunk.anchor))
            .transpose()?;
        let content_hash = record
            .chunk
            .as_ref()
            .map(|chunk| chunk.content_hash.as_str());

        self.conn.execute(
            r#"
            INSERT INTO memory_records (
                id,
                source_uri,
                source_kind,
                source_label,
                recorded_at,
                scope,
                record_type,
                truth_layer,
                provenance_json,
                content_text,
                chunk_index,
                chunk_count,
                chunk_anchor_json,
                content_hash,
                valid_from,
                valid_to,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)
            "#,
            params![
                &record.id,
                &record.source.uri,
                record.source.kind.as_str(),
                &record.source.label,
                &record.timestamp.recorded_at,
                record.scope.as_str(),
                record.record_type.as_str(),
                record.truth_layer.as_str(),
                provenance_json,
                &record.content_text,
                chunk_index,
                chunk_count,
                chunk_anchor_json,
                content_hash,
                &record.validity.valid_from,
                &record.validity.valid_to,
                &record.timestamp.created_at,
                &record.timestamp.updated_at,
            ],
        )?;

        if matches!(record.truth_layer, TruthLayer::T3) {
            self.conn.execute(
                r#"
                INSERT INTO truth_t3_state (
                    record_id,
                    confidence,
                    revocation_state,
                    revoked_at,
                    revocation_reason,
                    shared_conflict_note,
                    last_reviewed_at
                )
                VALUES (?1, ?2, ?3, NULL, NULL, NULL, NULL)
                ON CONFLICT(record_id) DO NOTHING
                "#,
                params![
                    &record.id,
                    T3Confidence::Medium.as_str(),
                    T3RevocationState::Active.as_str(),
                ],
            )?;
        }

        Ok(())
    }

    pub fn get_record(&self, id: &str) -> Result<Option<MemoryRecord>, RepositoryError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                id,
                source_uri,
                source_kind,
                source_label,
                recorded_at,
                scope,
                record_type,
                truth_layer,
                provenance_json,
                content_text,
                chunk_index,
                chunk_count,
                chunk_anchor_json,
                content_hash,
                valid_from,
                valid_to,
                created_at,
                updated_at
            FROM memory_records
            WHERE id = ?1
            "#,
        )?;

        let mut rows = statement.query([id])?;
        match rows.next()? {
            Some(row) => Ok(Some(map_record_row(row)?)),
            None => Ok(None),
        }
    }

    pub fn list_records(&self) -> Result<Vec<MemoryRecord>, RepositoryError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                id,
                source_uri,
                source_kind,
                source_label,
                recorded_at,
                scope,
                record_type,
                truth_layer,
                provenance_json,
                content_text,
                chunk_index,
                chunk_count,
                chunk_anchor_json,
                content_hash,
                valid_from,
                valid_to,
                created_at,
                updated_at
            FROM memory_records
            ORDER BY recorded_at ASC, id ASC
            "#,
        )?;

        let mut rows = statement.query([])?;
        let mut records = Vec::new();
        while let Some(row) = rows.next()? {
            records.push(map_record_row(row)?);
        }

        Ok(records)
    }

    pub fn get_layered_record(
        &self,
        id: &str,
    ) -> Result<Option<LayeredMemoryRecord>, RepositoryError> {
        let Some(record) = self.get_record(id)? else {
            return Ok(None);
        };
        let dsl = <Self as FactDslStore>::get_fact_dsl(self, id).map_err(|error| {
            RepositoryError::Json(serde_json::Error::io(std::io::Error::other(
                error.to_string(),
            )))
        })?;

        Ok(Some(LayeredMemoryRecord { record, dsl }))
    }

    pub fn list_layered_records(&self) -> Result<Vec<LayeredMemoryRecord>, RepositoryError> {
        let records = self.list_records()?;
        records
            .into_iter()
            .map(|record| {
                let dsl =
                    <Self as FactDslStore>::get_fact_dsl(self, &record.id).map_err(|error| {
                        RepositoryError::Json(serde_json::Error::io(std::io::Error::other(
                            error.to_string(),
                        )))
                    })?;
                Ok(LayeredMemoryRecord { record, dsl })
            })
            .collect()
    }

    pub fn list_layered_records_for_ids(
        &self,
        ids: &[String],
    ) -> Result<Vec<LayeredMemoryRecord>, RepositoryError> {
        let mut layered = Vec::new();
        for id in ids {
            if let Some(record) = self.get_layered_record(id)? {
                layered.push(record);
            }
        }
        Ok(layered)
    }

    pub fn insert_record_embedding(
        &self,
        embedding: &RecordEmbedding,
    ) -> Result<(), RepositoryError> {
        self.conn.execute(
            r#"
            INSERT INTO record_embeddings (
                record_id,
                backend,
                model,
                dimensions,
                embedding_json,
                source_text_hash,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(record_id) DO UPDATE SET
                backend = excluded.backend,
                model = excluded.model,
                dimensions = excluded.dimensions,
                embedding_json = excluded.embedding_json,
                source_text_hash = excluded.source_text_hash,
                updated_at = excluded.updated_at
            "#,
            params![
                &embedding.record_id,
                embedding.backend.as_str(),
                &embedding.model,
                embedding.dimensions,
                serde_json::to_string(&embedding.embedding)?,
                &embedding.source_text_hash,
                &embedding.created_at,
                &embedding.updated_at,
            ],
        )?;

        Ok(())
    }

    pub fn list_record_embeddings(&self) -> Result<Vec<RecordEmbedding>, RepositoryError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                record_id,
                backend,
                model,
                dimensions,
                embedding_json,
                source_text_hash,
                created_at,
                updated_at
            FROM record_embeddings
            ORDER BY record_id ASC
            "#,
        )?;

        let mut rows = statement.query([])?;
        let mut embeddings = Vec::new();
        while let Some(row) = rows.next()? {
            embeddings.push(RecordEmbedding {
                record_id: row.get(0)?,
                backend: parse_embedding_backend(&row.get::<_, String>(1)?)?,
                model: row.get(2)?,
                dimensions: row.get::<_, u32>(3)?,
                embedding: serde_json::from_str(&row.get::<_, String>(4)?)?,
                source_text_hash: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            });
        }

        Ok(embeddings)
    }

    pub fn list_fact_dsl_records(&self) -> Result<Vec<PersistedFactDslRecordV1>, RepositoryError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                record_id,
                domain,
                topic,
                aspect,
                kind,
                claim,
                truth_layer,
                source_ref,
                why,
                time_hint,
                cond,
                impact,
                conf,
                rel_json,
                classification_confidence,
                needs_review
            FROM fact_dsl_records
            ORDER BY record_id ASC
            "#,
        )?;

        let mut rows = statement.query([])?;
        let mut records = Vec::new();
        while let Some(row) = rows.next()? {
            records.push(map_fact_dsl_row(row)?);
        }

        Ok(records)
    }

    pub fn get_t3_state(&self, record_id: &str) -> Result<Option<T3State>, RepositoryError> {
        self.conn
            .query_row(
                r#"
                SELECT
                    record_id,
                    confidence,
                    revocation_state,
                    revoked_at,
                    revocation_reason,
                    shared_conflict_note,
                    last_reviewed_at,
                    created_at,
                    updated_at
                FROM truth_t3_state
                WHERE record_id = ?1
                "#,
                [record_id],
                map_t3_state_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn update_t3_last_reviewed_at(
        &self,
        record_id: &str,
        reviewed_at: &str,
    ) -> Result<(), RepositoryError> {
        self.conn.execute(
            r#"
            UPDATE truth_t3_state
            SET last_reviewed_at = ?2,
                updated_at = ?2
            WHERE record_id = ?1
            "#,
            params![record_id, reviewed_at],
        )?;

        Ok(())
    }

    pub fn insert_promotion_review(&self, review: &PromotionReview) -> Result<(), RepositoryError> {
        let review_notes_json = review
            .review_notes
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        self.conn.execute(
            r#"
            INSERT INTO truth_promotion_reviews (
                review_id,
                source_record_id,
                target_layer,
                result_trigger_state,
                evidence_review_state,
                consensus_check_state,
                metacog_approval_state,
                decision_state,
                review_notes_json,
                approved_at,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)
            "#,
            params![
                &review.review_id,
                &review.source_record_id,
                review.target_layer.as_str(),
                review.result_trigger_state.as_str(),
                review.evidence_review_state.as_str(),
                review.consensus_check_state.as_str(),
                review.metacog_approval_state.as_str(),
                review.decision_state.as_str(),
                review_notes_json,
                &review.approved_at,
                &review.created_at,
                &review.updated_at,
            ],
        )?;

        Ok(())
    }

    pub fn get_promotion_review(
        &self,
        review_id: &str,
    ) -> Result<Option<PromotionReview>, RepositoryError> {
        self.conn
            .query_row(
                r#"
                SELECT
                    review_id,
                    source_record_id,
                    target_layer,
                    result_trigger_state,
                    evidence_review_state,
                    consensus_check_state,
                    metacog_approval_state,
                    decision_state,
                    review_notes_json,
                    approved_at,
                    created_at,
                    updated_at
                FROM truth_promotion_reviews
                WHERE review_id = ?1
                "#,
                [review_id],
                map_promotion_review_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn list_promotion_reviews(
        &self,
        source_record_id: &str,
    ) -> Result<Vec<PromotionReview>, RepositoryError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                review_id,
                source_record_id,
                target_layer,
                result_trigger_state,
                evidence_review_state,
                consensus_check_state,
                metacog_approval_state,
                decision_state,
                review_notes_json,
                approved_at,
                created_at,
                updated_at
            FROM truth_promotion_reviews
            WHERE source_record_id = ?1
            ORDER BY updated_at DESC, review_id DESC
            "#,
        )?;
        let rows = statement.query_map([source_record_id], map_promotion_review_row)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn list_pending_promotion_reviews(&self) -> Result<Vec<PromotionReview>, RepositoryError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                review_id,
                source_record_id,
                target_layer,
                result_trigger_state,
                evidence_review_state,
                consensus_check_state,
                metacog_approval_state,
                decision_state,
                review_notes_json,
                approved_at,
                created_at,
                updated_at
            FROM truth_promotion_reviews
            WHERE decision_state = ?1
            ORDER BY updated_at DESC, review_id DESC
            "#,
        )?;
        let rows = statement.query_map(
            [PromotionDecisionState::Pending.as_str()],
            map_promotion_review_row,
        )?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn update_promotion_review(&self, review: &PromotionReview) -> Result<(), RepositoryError> {
        let review_notes_json = review
            .review_notes
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        self.conn.execute(
            r#"
            UPDATE truth_promotion_reviews
            SET result_trigger_state = ?2,
                evidence_review_state = ?3,
                consensus_check_state = ?4,
                metacog_approval_state = ?5,
                decision_state = ?6,
                review_notes_json = ?7,
                approved_at = ?8,
                updated_at = ?9
            WHERE review_id = ?1
            "#,
            params![
                &review.review_id,
                review.result_trigger_state.as_str(),
                review.evidence_review_state.as_str(),
                review.consensus_check_state.as_str(),
                review.metacog_approval_state.as_str(),
                review.decision_state.as_str(),
                review_notes_json,
                &review.approved_at,
                &review.updated_at,
            ],
        )?;

        Ok(())
    }

    pub fn insert_promotion_evidence(
        &self,
        evidence: &PromotionEvidence,
    ) -> Result<(), RepositoryError> {
        let evidence_note_json = evidence
            .evidence_note
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        self.conn.execute(
            r#"
            INSERT INTO truth_promotion_evidence (
                review_id,
                evidence_record_id,
                evidence_role,
                evidence_note_json,
                created_at
            )
            VALUES (?1, ?2, ?3, ?4, COALESCE(?5, CURRENT_TIMESTAMP))
            "#,
            params![
                &evidence.review_id,
                &evidence.evidence_record_id,
                evidence.evidence_role.as_str(),
                evidence_note_json,
                &evidence.created_at,
            ],
        )?;

        Ok(())
    }

    pub fn list_promotion_evidence(
        &self,
        review_id: &str,
    ) -> Result<Vec<PromotionEvidence>, RepositoryError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                review_id,
                evidence_record_id,
                evidence_role,
                evidence_note_json,
                created_at
            FROM truth_promotion_evidence
            WHERE review_id = ?1
            ORDER BY created_at ASC, evidence_record_id ASC
            "#,
        )?;
        let rows = statement.query_map([review_id], map_promotion_evidence_row)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn list_ontology_candidates(
        &self,
        source_record_id: &str,
    ) -> Result<Vec<OntologyCandidate>, RepositoryError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                candidate_id,
                source_record_id,
                basis_record_ids_json,
                proposed_structure_json,
                time_stability_state,
                agent_reproducibility_state,
                context_invariance_state,
                predictive_utility_state,
                structural_review_state,
                candidate_state,
                decided_at,
                created_at,
                updated_at
            FROM truth_ontology_candidates
            WHERE source_record_id = ?1
            ORDER BY updated_at DESC, candidate_id DESC
            "#,
        )?;
        let rows = statement.query_map([source_record_id], map_ontology_candidate_row)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn list_pending_ontology_candidates(
        &self,
    ) -> Result<Vec<OntologyCandidate>, RepositoryError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                candidate_id,
                source_record_id,
                basis_record_ids_json,
                proposed_structure_json,
                time_stability_state,
                agent_reproducibility_state,
                context_invariance_state,
                predictive_utility_state,
                structural_review_state,
                candidate_state,
                decided_at,
                created_at,
                updated_at
            FROM truth_ontology_candidates
            WHERE candidate_state = ?1
            ORDER BY updated_at DESC, candidate_id DESC
            "#,
        )?;
        let rows = statement.query_map(
            [OntologyCandidateState::Pending.as_str()],
            map_ontology_candidate_row,
        )?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn insert_ontology_candidate(
        &self,
        candidate: &OntologyCandidate,
    ) -> Result<(), RepositoryError> {
        let basis_record_ids_json = serde_json::to_string(&candidate.basis_record_ids)?;
        let proposed_structure_json = serde_json::to_string(&candidate.proposed_structure)?;

        self.conn.execute(
            r#"
            INSERT INTO truth_ontology_candidates (
                candidate_id,
                source_record_id,
                basis_record_ids_json,
                proposed_structure_json,
                time_stability_state,
                agent_reproducibility_state,
                context_invariance_state,
                predictive_utility_state,
                structural_review_state,
                candidate_state,
                decided_at,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            "#,
            params![
                &candidate.candidate_id,
                &candidate.source_record_id,
                basis_record_ids_json,
                proposed_structure_json,
                candidate.time_stability_state.as_str(),
                candidate.agent_reproducibility_state.as_str(),
                candidate.context_invariance_state.as_str(),
                candidate.predictive_utility_state.as_str(),
                candidate.structural_review_state.as_str(),
                candidate.candidate_state.as_str(),
                &candidate.decided_at,
                &candidate.created_at,
                &candidate.updated_at,
            ],
        )?;

        Ok(())
    }

    pub fn insert_rumination_queue_item(
        &self,
        item: &PersistedRuminationQueueItem,
    ) -> Result<(), RepositoryError> {
        let table = queue_table(&item.queue_tier)?;
        let evidence_refs_json = item
            .evidence_refs_json
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let source_report_json = item
            .source_report_json
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;
        let payload_json = serde_json::to_string(&item.payload_json)?;

        let sql = format!(
            r#"
            INSERT INTO {table} (
                item_id,
                trigger_kind,
                status,
                subject_ref,
                dedupe_key,
                cooldown_key,
                budget_bucket,
                priority,
                budget_cost,
                attempt_count,
                cooldown_until,
                next_eligible_at,
                payload_json,
                evidence_refs_json,
                source_report_json,
                last_error,
                created_at,
                updated_at,
                processed_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18, ?19)
            "#
        );

        self.conn.execute(
            &sql,
            params![
                &item.item_id,
                &item.trigger_kind,
                item.status.as_str(),
                &item.subject_ref,
                &item.dedupe_key,
                &item.cooldown_key,
                &item.budget_bucket,
                item.priority,
                item.budget_cost,
                item.attempt_count,
                &item.cooldown_until,
                &item.next_eligible_at,
                payload_json,
                evidence_refs_json,
                source_report_json,
                &item.last_error,
                &item.created_at,
                &item.updated_at,
                &item.processed_at,
            ],
        )?;

        Ok(())
    }

    pub fn list_rumination_queue_items(
        &self,
        queue_tier: &str,
    ) -> Result<Vec<PersistedRuminationQueueItem>, RepositoryError> {
        let table = queue_table(queue_tier)?;
        let sql = format!(
            r#"
            SELECT
                item_id,
                trigger_kind,
                status,
                subject_ref,
                dedupe_key,
                cooldown_key,
                budget_bucket,
                priority,
                budget_cost,
                attempt_count,
                cooldown_until,
                next_eligible_at,
                payload_json,
                evidence_refs_json,
                source_report_json,
                last_error,
                created_at,
                updated_at,
                processed_at
            FROM {table}
            ORDER BY created_at ASC, item_id ASC
            "#
        );
        let mut statement = self.conn.prepare(&sql)?;
        let rows = statement.query_map([], |row| map_rumination_queue_row(row, queue_tier))?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn find_active_rumination_item(
        &self,
        queue_tier: &str,
        dedupe_key: &str,
    ) -> Result<Option<PersistedRuminationQueueItem>, RepositoryError> {
        let table = queue_table(queue_tier)?;
        let sql = format!(
            r#"
            SELECT
                item_id,
                trigger_kind,
                status,
                subject_ref,
                dedupe_key,
                cooldown_key,
                budget_bucket,
                priority,
                budget_cost,
                attempt_count,
                cooldown_until,
                next_eligible_at,
                payload_json,
                evidence_refs_json,
                source_report_json,
                last_error,
                created_at,
                updated_at,
                processed_at
            FROM {table}
            WHERE dedupe_key = ?1
              AND status IN (?2, ?3)
            ORDER BY priority DESC, created_at ASC
            LIMIT 1
            "#
        );

        self.conn
            .query_row(
                &sql,
                params![
                    dedupe_key,
                    RuminationQueueStatus::Queued.as_str(),
                    RuminationQueueStatus::Claimed.as_str()
                ],
                |row| map_rumination_queue_row(row, queue_tier),
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn claim_next_rumination_item(
        &self,
        now: &str,
    ) -> Result<Option<PersistedRuminationQueueItem>, RepositoryError> {
        if let Some(item) = self.claim_next_rumination_item_for_tier_impl("spq", now)? {
            return Ok(Some(item));
        }

        self.claim_next_rumination_item_for_tier_impl("lpq", now)
    }

    pub fn claim_next_rumination_item_for_tier(
        &self,
        queue_tier: &str,
        now: &str,
    ) -> Result<Option<PersistedRuminationQueueItem>, RepositoryError> {
        self.claim_next_rumination_item_for_tier_impl(queue_tier, now)
    }

    pub fn complete_rumination_queue_item(
        &self,
        queue_tier: &str,
        item_id: &str,
        processed_at: &str,
    ) -> Result<(), RepositoryError> {
        let table = queue_table(queue_tier)?;
        let sql = format!(
            r#"
            UPDATE {table}
            SET status = ?2,
                processed_at = ?3,
                updated_at = ?3
            WHERE item_id = ?1
            "#
        );

        self.conn.execute(
            &sql,
            params![
                item_id,
                RuminationQueueStatus::Completed.as_str(),
                processed_at
            ],
        )?;

        Ok(())
    }

    pub fn retry_rumination_queue_item(
        &self,
        queue_tier: &str,
        item_id: &str,
        next_eligible_at: &str,
        last_error: &str,
        updated_at: &str,
    ) -> Result<(), RepositoryError> {
        let table = queue_table(queue_tier)?;
        let sql = format!(
            r#"
            UPDATE {table}
            SET status = ?2,
                attempt_count = attempt_count + 1,
                next_eligible_at = ?3,
                last_error = ?4,
                updated_at = ?5
            WHERE item_id = ?1
            "#
        );

        self.conn.execute(
            &sql,
            params![
                item_id,
                RuminationQueueStatus::Queued.as_str(),
                next_eligible_at,
                last_error,
                updated_at
            ],
        )?;

        Ok(())
    }

    pub fn get_latest_rumination_cooldown(
        &self,
        queue_tier: &str,
        cooldown_key: &str,
    ) -> Result<Option<String>, RepositoryError> {
        self.conn
            .query_row(
                r#"
                SELECT cooldown_until
                FROM rumination_trigger_state
                WHERE queue_tier = ?1
                  AND cooldown_key = ?2
                  AND cooldown_until IS NOT NULL
                ORDER BY cooldown_until DESC
                LIMIT 1
                "#,
                params![queue_tier, cooldown_key],
                |row| row.get(0),
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn total_rumination_budget_spent(
        &self,
        queue_tier: &str,
        budget_bucket: &str,
    ) -> Result<u32, RepositoryError> {
        self.conn
            .query_row(
                r#"
                SELECT COALESCE(MAX(budget_spent), 0)
                FROM rumination_trigger_state
                WHERE queue_tier = ?1
                  AND budget_bucket = ?2
                "#,
                params![queue_tier, budget_bucket],
                |row| row.get(0),
            )
            .map_err(Into::into)
    }

    pub fn get_rumination_trigger_state(
        &self,
        queue_tier: &str,
        dedupe_key: &str,
    ) -> Result<Option<PersistedRuminationTriggerState>, RepositoryError> {
        self.conn
            .query_row(
                r#"
                SELECT
                    queue_tier,
                    trigger_kind,
                    dedupe_key,
                    cooldown_key,
                    budget_bucket,
                    budget_window_started_at,
                    budget_spent,
                    cooldown_until,
                    last_enqueued_at,
                    last_seen_at,
                    last_decision,
                    last_item_id,
                    updated_at
                FROM rumination_trigger_state
                WHERE queue_tier = ?1
                  AND dedupe_key = ?2
                "#,
                params![queue_tier, dedupe_key],
                map_rumination_trigger_state_row,
            )
            .optional()
            .map_err(Into::into)
    }

    pub fn upsert_rumination_trigger_state(
        &self,
        state: &PersistedRuminationTriggerState,
    ) -> Result<(), RepositoryError> {
        self.conn.execute(
            r#"
            INSERT INTO rumination_trigger_state (
                queue_tier,
                trigger_kind,
                dedupe_key,
                cooldown_key,
                budget_bucket,
                budget_window_started_at,
                budget_spent,
                cooldown_until,
                last_enqueued_at,
                last_seen_at,
                last_decision,
                last_item_id,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13)
            ON CONFLICT(queue_tier, dedupe_key) DO UPDATE SET
                trigger_kind = excluded.trigger_kind,
                cooldown_key = excluded.cooldown_key,
                budget_bucket = excluded.budget_bucket,
                budget_window_started_at = excluded.budget_window_started_at,
                budget_spent = excluded.budget_spent,
                cooldown_until = excluded.cooldown_until,
                last_enqueued_at = excluded.last_enqueued_at,
                last_seen_at = excluded.last_seen_at,
                last_decision = excluded.last_decision,
                last_item_id = excluded.last_item_id,
                updated_at = excluded.updated_at
            "#,
            params![
                &state.queue_tier,
                &state.trigger_kind,
                &state.dedupe_key,
                &state.cooldown_key,
                &state.budget_bucket,
                &state.budget_window_started_at,
                state.budget_spent,
                &state.cooldown_until,
                &state.last_enqueued_at,
                &state.last_seen_at,
                &state.last_decision,
                &state.last_item_id,
                &state.updated_at,
            ],
        )?;

        Ok(())
    }

    pub fn insert_local_adaptation_entry(
        &self,
        entry: &LocalAdaptationEntry,
    ) -> Result<(), RepositoryError> {
        let value_json = serde_json::to_string(&entry.payload)?;

        self.conn.execute(
            r#"
            INSERT INTO local_adaptation_entries (
                entry_id,
                subject_ref,
                target_kind,
                key,
                value_json,
                source_queue_item_id,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            "#,
            params![
                &entry.entry_id,
                &entry.subject_ref,
                entry.target_kind.as_str(),
                &entry.key,
                value_json,
                &entry.source_queue_item_id,
                &entry.created_at,
                &entry.updated_at,
            ],
        )?;

        Ok(())
    }

    pub fn list_local_adaptation_entries(
        &self,
        subject_ref: &str,
    ) -> Result<Vec<LocalAdaptationEntry>, RepositoryError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                entry_id,
                subject_ref,
                target_kind,
                key,
                value_json,
                source_queue_item_id,
                created_at,
                updated_at
            FROM local_adaptation_entries
            WHERE subject_ref = ?1
            ORDER BY updated_at DESC, entry_id DESC
            "#,
        )?;
        let rows = statement.query_map([subject_ref], map_local_adaptation_row)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn insert_rumination_candidate(
        &self,
        candidate: &RuminationCandidate,
    ) -> Result<(), RepositoryError> {
        let payload_json = serialize_rumination_candidate_payload(candidate)?;
        let evidence_refs_json = serde_json::to_string(&candidate.evidence_refs)?;

        self.conn.execute(
            r#"
            INSERT INTO rumination_candidates (
                candidate_id,
                source_queue_item_id,
                candidate_kind,
                subject_ref,
                payload_json,
                evidence_refs_json,
                status,
                created_at,
                updated_at
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)
            "#,
            params![
                &candidate.candidate_id,
                &candidate.source_queue_item_id,
                candidate.candidate_kind.as_str(),
                &candidate.subject_ref,
                payload_json,
                evidence_refs_json,
                candidate.status.as_str(),
                &candidate.created_at,
                &candidate.updated_at,
            ],
        )?;

        Ok(())
    }

    pub fn update_rumination_candidate(
        &self,
        candidate: &RuminationCandidate,
    ) -> Result<(), RepositoryError> {
        let payload_json = serialize_rumination_candidate_payload(candidate)?;
        let evidence_refs_json = serde_json::to_string(&candidate.evidence_refs)?;

        self.conn.execute(
            r#"
            UPDATE rumination_candidates
            SET source_queue_item_id = ?2,
                candidate_kind = ?3,
                subject_ref = ?4,
                payload_json = ?5,
                evidence_refs_json = ?6,
                status = ?7,
                created_at = ?8,
                updated_at = ?9
            WHERE candidate_id = ?1
            "#,
            params![
                &candidate.candidate_id,
                &candidate.source_queue_item_id,
                candidate.candidate_kind.as_str(),
                &candidate.subject_ref,
                payload_json,
                evidence_refs_json,
                candidate.status.as_str(),
                &candidate.created_at,
                &candidate.updated_at,
            ],
        )?;

        Ok(())
    }

    pub fn list_rumination_candidates(&self) -> Result<Vec<RuminationCandidate>, RepositoryError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                candidate_id,
                source_queue_item_id,
                candidate_kind,
                subject_ref,
                payload_json,
                evidence_refs_json,
                status,
                created_at,
                updated_at
            FROM rumination_candidates
            ORDER BY created_at ASC, candidate_kind ASC, candidate_id ASC
            "#,
        )?;
        let rows = statement.query_map([], map_rumination_candidate_row)?;

        rows.collect::<Result<Vec<_>, _>>().map_err(Into::into)
    }

    pub fn get_rumination_candidate(
        &self,
        candidate_id: &str,
    ) -> Result<Option<RuminationCandidate>, RepositoryError> {
        self.conn
            .query_row(
                r#"
                SELECT
                    candidate_id,
                    source_queue_item_id,
                    candidate_kind,
                    subject_ref,
                    payload_json,
                    evidence_refs_json,
                    status,
                    created_at,
                    updated_at
                FROM rumination_candidates
                WHERE candidate_id = ?1
                "#,
                [candidate_id],
                map_rumination_candidate_row,
            )
            .optional()
            .map_err(Into::into)
    }

    fn claim_next_rumination_item_for_tier_impl(
        &self,
        queue_tier: &str,
        now: &str,
    ) -> Result<Option<PersistedRuminationQueueItem>, RepositoryError> {
        let table = queue_table(queue_tier)?;
        let select_sql = format!(
            r#"
            SELECT item_id
            FROM {table}
            WHERE status = ?1
              AND next_eligible_at <= ?2
            ORDER BY priority DESC, next_eligible_at ASC, created_at ASC
            LIMIT 1
            "#
        );
        let maybe_item_id = self
            .conn
            .query_row(
                &select_sql,
                params![RuminationQueueStatus::Queued.as_str(), now],
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        let Some(item_id) = maybe_item_id else {
            return Ok(None);
        };

        let update_sql = format!(
            r#"
            UPDATE {table}
            SET status = ?2,
                updated_at = ?3
            WHERE item_id = ?1
              AND status = ?4
              AND next_eligible_at <= ?3
            "#
        );
        let updated = self.conn.execute(
            &update_sql,
            params![
                &item_id,
                RuminationQueueStatus::Claimed.as_str(),
                now,
                RuminationQueueStatus::Queued.as_str()
            ],
        )?;
        if updated == 0 {
            return Ok(None);
        }

        let fetch_sql = format!(
            r#"
            SELECT
                item_id,
                trigger_kind,
                status,
                subject_ref,
                dedupe_key,
                cooldown_key,
                budget_bucket,
                priority,
                budget_cost,
                attempt_count,
                cooldown_until,
                next_eligible_at,
                payload_json,
                evidence_refs_json,
                source_report_json,
                last_error,
                created_at,
                updated_at,
                processed_at
            FROM {table}
            WHERE item_id = ?1
            "#
        );

        self.conn
            .query_row(&fetch_sql, [item_id], |row| {
                map_rumination_queue_row(row, queue_tier)
            })
            .optional()
            .map_err(Into::into)
    }

    pub fn get_truth_record(&self, id: &str) -> Result<Option<TruthRecord>, RepositoryError> {
        let Some(base) = self.get_record(id)? else {
            return Ok(None);
        };

        let truth_record = match base.truth_layer {
            TruthLayer::T1 => TruthRecord::T1 { base },
            TruthLayer::T2 => TruthRecord::T2 {
                open_candidates: self.list_ontology_candidates(id)?,
                base,
            },
            TruthLayer::T3 => TruthRecord::T3 {
                t3_state: Some(self.get_t3_state(id)?.ok_or_else(|| {
                    RepositoryError::MissingT3State {
                        record_id: id.to_string(),
                    }
                })?),
                open_reviews: self.list_promotion_reviews(id)?,
                base,
            },
        };

        Ok(Some(truth_record))
    }

    pub fn count_records(&self) -> Result<u64, RepositoryError> {
        self.conn
            .query_row("SELECT COUNT(*) FROM memory_records", [], |row| row.get(0))
            .map_err(Into::into)
    }

    pub fn scope_counts(&self) -> Result<Vec<ScopeCount>, RepositoryError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT scope, COUNT(*)
            FROM memory_records
            GROUP BY scope
            ORDER BY scope
            "#,
        )?;
        let rows = statement.query_map([], |row| {
            let scope = row.get::<_, String>(0)?;
            let count = row.get::<_, u64>(1)?;
            Ok((scope, count))
        })?;

        rows.map(|row| {
            let (scope, count) = row?;
            Ok(ScopeCount {
                scope: parse_scope(&scope)?,
                count,
            })
        })
        .collect()
    }
}

impl FactDslStore for MemoryRepository<'_> {
    fn put_fact_dsl(&self, persisted: &PersistedFactDslRecordV1) -> Result<(), FactDslStoreError> {
        persisted.validate()?;

        self.conn.execute(
            r#"
            INSERT INTO fact_dsl_records (
                record_id,
                domain,
                topic,
                aspect,
                kind,
                claim,
                truth_layer,
                source_ref,
                why,
                time_hint,
                cond,
                impact,
                conf,
                rel_json,
                classification_confidence,
                needs_review
            )
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
            ON CONFLICT(record_id) DO UPDATE SET
                domain = excluded.domain,
                topic = excluded.topic,
                aspect = excluded.aspect,
                kind = excluded.kind,
                claim = excluded.claim,
                truth_layer = excluded.truth_layer,
                source_ref = excluded.source_ref,
                why = excluded.why,
                time_hint = excluded.time_hint,
                cond = excluded.cond,
                impact = excluded.impact,
                conf = excluded.conf,
                rel_json = excluded.rel_json,
                classification_confidence = excluded.classification_confidence,
                needs_review = excluded.needs_review
            "#,
            params![
                &persisted.record_id,
                &persisted.payload.domain,
                &persisted.payload.topic,
                &persisted.payload.aspect,
                &persisted.payload.kind,
                &persisted.payload.claim,
                &persisted.payload.truth_layer,
                &persisted.payload.source_ref,
                &persisted.payload.why,
                &persisted.payload.time,
                &persisted.payload.cond,
                &persisted.payload.impact,
                &persisted.payload.conf,
                persisted
                    .payload
                    .rel
                    .as_ref()
                    .map(serde_json::to_string)
                    .transpose()?,
                &persisted.classification_confidence,
                persisted.needs_review,
            ],
        )?;

        Ok(())
    }

    fn get_fact_dsl(
        &self,
        record_id: &str,
    ) -> Result<Option<PersistedFactDslRecordV1>, FactDslStoreError> {
        let mut statement = self.conn.prepare(
            r#"
            SELECT
                record_id,
                domain,
                topic,
                aspect,
                kind,
                claim,
                truth_layer,
                source_ref,
                why,
                time_hint,
                cond,
                    impact,
                    conf,
                    rel_json,
                    classification_confidence,
                    needs_review
                FROM fact_dsl_records
                WHERE record_id = ?1
                "#,
        )?;

        let mut rows = statement.query([record_id])?;
        match rows.next()? {
            Some(row) => map_fact_dsl_row(row)
                .map(Some)
                .map_err(|error| FactDslStoreError::Store(error.to_string())),
            None => Ok(None),
        }
    }

    fn list_fact_dsls(&self) -> Result<Vec<PersistedFactDslRecordV1>, FactDslStoreError> {
        self.list_fact_dsl_records()
            .map_err(|error| FactDslStoreError::Store(error.to_string()))
    }

    fn delete_fact_dsl(
        &self,
        record_id: &str,
    ) -> Result<Option<PersistedFactDslRecordV1>, FactDslStoreError> {
        let existing = self.get_fact_dsl(record_id)?;
        if existing.is_some() {
            self.conn.execute(
                "DELETE FROM fact_dsl_records WHERE record_id = ?1",
                [record_id],
            )?;
        }
        Ok(existing)
    }
}

fn map_record_row(row: &rusqlite::Row<'_>) -> Result<MemoryRecord, RepositoryError> {
    let source_kind = row.get::<_, String>(2)?;
    let scope = row.get::<_, String>(5)?;
    let record_type = row.get::<_, String>(6)?;
    let truth_layer = row.get::<_, String>(7)?;
    let provenance_json = row.get::<_, String>(8)?;
    let record_id = row.get::<_, String>(0)?;
    let chunk = map_chunk_metadata(row, &record_id)?;

    Ok(MemoryRecord {
        id: record_id,
        source: SourceRef {
            uri: row.get(1)?,
            kind: parse_source_kind(&source_kind)?,
            label: row.get(3)?,
        },
        timestamp: RecordTimestamp {
            recorded_at: row.get(4)?,
            created_at: row.get(16)?,
            updated_at: row.get(17)?,
        },
        scope: parse_scope(&scope)?,
        record_type: parse_record_type(&record_type)?,
        truth_layer: parse_truth_layer(&truth_layer)?,
        provenance: serde_json::from_str::<Provenance>(&provenance_json)?,
        content_text: row.get(9)?,
        chunk,
        validity: ValidityWindow {
            valid_from: row.get(14)?,
            valid_to: row.get(15)?,
        },
    })
}

fn map_chunk_metadata(
    row: &rusqlite::Row<'_>,
    record_id: &str,
) -> Result<Option<ChunkMetadata>, RepositoryError> {
    let chunk_index = row.get::<_, Option<u32>>(10)?;
    let chunk_count = row.get::<_, Option<u32>>(11)?;
    let anchor_json = row.get::<_, Option<String>>(12)?;
    let content_hash = row.get::<_, Option<String>>(13)?;

    match (chunk_index, chunk_count, anchor_json, content_hash) {
        (None, None, None, None) => Ok(None),
        (Some(chunk_index), Some(chunk_count), Some(anchor_json), Some(content_hash)) => {
            let anchor = serde_json::from_str::<ChunkAnchor>(&anchor_json)?;
            Ok(Some(ChunkMetadata {
                chunk_index,
                chunk_count,
                anchor,
                content_hash,
            }))
        }
        _ => Err(RepositoryError::IncompleteChunkMetadata {
            record_id: record_id.to_string(),
        }),
    }
}

fn map_t3_state_row(row: &rusqlite::Row<'_>) -> Result<T3State, rusqlite::Error> {
    Ok(T3State {
        record_id: row.get(0)?,
        confidence: T3Confidence::parse(&row.get::<_, String>(1)?).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                1,
                rusqlite::types::Type::Text,
                "invalid t3 confidence".into(),
            )
        })?,
        revocation_state: T3RevocationState::parse(&row.get::<_, String>(2)?).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                "invalid t3 revocation state".into(),
            )
        })?,
        revoked_at: row.get(3)?,
        revocation_reason: row.get(4)?,
        shared_conflict_note: row.get(5)?,
        last_reviewed_at: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn map_promotion_review_row(row: &rusqlite::Row<'_>) -> Result<PromotionReview, rusqlite::Error> {
    let target_layer = row.get::<_, String>(2)?;
    let result_trigger_state = row.get::<_, String>(3)?;
    let evidence_review_state = row.get::<_, String>(4)?;
    let consensus_check_state = row.get::<_, String>(5)?;
    let metacog_approval_state = row.get::<_, String>(6)?;
    let decision_state = row.get::<_, String>(7)?;
    let review_notes_json = row.get::<_, Option<String>>(8)?;

    Ok(PromotionReview {
        review_id: row.get(0)?,
        source_record_id: row.get(1)?,
        target_layer: TruthLayer::parse(&target_layer).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                "invalid promotion target layer".into(),
            )
        })?,
        result_trigger_state: ReviewGateState::parse(&result_trigger_state).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Text,
                "invalid result trigger state".into(),
            )
        })?,
        evidence_review_state: ReviewGateState::parse(&evidence_review_state).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                4,
                rusqlite::types::Type::Text,
                "invalid evidence review state".into(),
            )
        })?,
        consensus_check_state: ReviewGateState::parse(&consensus_check_state).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                5,
                rusqlite::types::Type::Text,
                "invalid consensus review state".into(),
            )
        })?,
        metacog_approval_state: ReviewGateState::parse(&metacog_approval_state).ok_or_else(
            || {
                rusqlite::Error::FromSqlConversionFailure(
                    6,
                    rusqlite::types::Type::Text,
                    "invalid metacognitive approval state".into(),
                )
            },
        )?,
        decision_state: PromotionDecisionState::parse(&decision_state).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                7,
                rusqlite::types::Type::Text,
                "invalid promotion decision state".into(),
            )
        })?,
        review_notes: review_notes_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()
            .map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    8,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?,
        approved_at: row.get(9)?,
        created_at: row.get(10)?,
        updated_at: row.get(11)?,
    })
}

fn map_promotion_evidence_row(
    row: &rusqlite::Row<'_>,
) -> Result<PromotionEvidence, rusqlite::Error> {
    let evidence_role = row.get::<_, String>(2)?;
    let evidence_note_json = row.get::<_, Option<String>>(3)?;

    Ok(PromotionEvidence {
        review_id: row.get(0)?,
        evidence_record_id: row.get(1)?,
        evidence_role: EvidenceRole::parse(&evidence_role).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                "invalid evidence role".into(),
            )
        })?,
        evidence_note: evidence_note_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()
            .map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    3,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?,
        created_at: row.get(4)?,
    })
}

fn map_ontology_candidate_row(
    row: &rusqlite::Row<'_>,
) -> Result<OntologyCandidate, rusqlite::Error> {
    let basis_record_ids_json = row.get::<_, String>(2)?;
    let proposed_structure_json = row.get::<_, String>(3)?;
    let time_stability_state = row.get::<_, String>(4)?;
    let agent_reproducibility_state = row.get::<_, String>(5)?;
    let context_invariance_state = row.get::<_, String>(6)?;
    let predictive_utility_state = row.get::<_, String>(7)?;
    let structural_review_state = row.get::<_, String>(8)?;
    let candidate_state = row.get::<_, String>(9)?;

    Ok(OntologyCandidate {
        candidate_id: row.get(0)?,
        source_record_id: row.get(1)?,
        basis_record_ids: serde_json::from_str(&basis_record_ids_json).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        proposed_structure: serde_json::from_str(&proposed_structure_json).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                3,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        time_stability_state: CandidateReviewState::parse(&time_stability_state).ok_or_else(
            || {
                rusqlite::Error::FromSqlConversionFailure(
                    4,
                    rusqlite::types::Type::Text,
                    "invalid time stability state".into(),
                )
            },
        )?,
        agent_reproducibility_state: CandidateReviewState::parse(&agent_reproducibility_state)
            .ok_or_else(|| {
                rusqlite::Error::FromSqlConversionFailure(
                    5,
                    rusqlite::types::Type::Text,
                    "invalid agent reproducibility state".into(),
                )
            })?,
        context_invariance_state: CandidateReviewState::parse(&context_invariance_state)
            .ok_or_else(|| {
                rusqlite::Error::FromSqlConversionFailure(
                    6,
                    rusqlite::types::Type::Text,
                    "invalid context invariance state".into(),
                )
            })?,
        predictive_utility_state: CandidateReviewState::parse(&predictive_utility_state)
            .ok_or_else(|| {
                rusqlite::Error::FromSqlConversionFailure(
                    7,
                    rusqlite::types::Type::Text,
                    "invalid predictive utility state".into(),
                )
            })?,
        structural_review_state: CandidateReviewState::parse(&structural_review_state).ok_or_else(
            || {
                rusqlite::Error::FromSqlConversionFailure(
                    8,
                    rusqlite::types::Type::Text,
                    "invalid structural review state".into(),
                )
            },
        )?,
        candidate_state: OntologyCandidateState::parse(&candidate_state).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                9,
                rusqlite::types::Type::Text,
                "invalid ontology candidate state".into(),
            )
        })?,
        decided_at: row.get(10)?,
        created_at: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn map_rumination_queue_row(
    row: &rusqlite::Row<'_>,
    queue_tier: &str,
) -> Result<PersistedRuminationQueueItem, rusqlite::Error> {
    let status = row.get::<_, String>(2)?;
    let payload_json = row.get::<_, String>(12)?;
    let evidence_refs_json = row.get::<_, Option<String>>(13)?;
    let source_report_json = row.get::<_, Option<String>>(14)?;

    Ok(PersistedRuminationQueueItem {
        queue_tier: queue_tier.to_string(),
        item_id: row.get(0)?,
        trigger_kind: row.get(1)?,
        status: RuminationQueueStatus::parse(&status).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                "invalid rumination queue status".into(),
            )
        })?,
        subject_ref: row.get(3)?,
        dedupe_key: row.get(4)?,
        cooldown_key: row.get(5)?,
        budget_bucket: row.get(6)?,
        priority: row.get(7)?,
        budget_cost: row.get(8)?,
        attempt_count: row.get(9)?,
        cooldown_until: row.get(10)?,
        next_eligible_at: row.get(11)?,
        payload_json: serde_json::from_str(&payload_json).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                12,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        evidence_refs_json: evidence_refs_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()
            .map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    13,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?,
        source_report_json: source_report_json
            .as_deref()
            .map(serde_json::from_str)
            .transpose()
            .map_err(|error| {
                rusqlite::Error::FromSqlConversionFailure(
                    14,
                    rusqlite::types::Type::Text,
                    Box::new(error),
                )
            })?,
        last_error: row.get(15)?,
        created_at: row.get(16)?,
        updated_at: row.get(17)?,
        processed_at: row.get(18)?,
    })
}

fn map_rumination_trigger_state_row(
    row: &rusqlite::Row<'_>,
) -> Result<PersistedRuminationTriggerState, rusqlite::Error> {
    Ok(PersistedRuminationTriggerState {
        queue_tier: row.get(0)?,
        trigger_kind: row.get(1)?,
        dedupe_key: row.get(2)?,
        cooldown_key: row.get(3)?,
        budget_bucket: row.get(4)?,
        budget_window_started_at: row.get(5)?,
        budget_spent: row.get(6)?,
        cooldown_until: row.get(7)?,
        last_enqueued_at: row.get(8)?,
        last_seen_at: row.get(9)?,
        last_decision: row.get(10)?,
        last_item_id: row.get(11)?,
        updated_at: row.get(12)?,
    })
}

fn map_local_adaptation_row(
    row: &rusqlite::Row<'_>,
) -> Result<LocalAdaptationEntry, rusqlite::Error> {
    let target_kind = row.get::<_, String>(2)?;
    let payload_json = row.get::<_, String>(4)?;
    let payload = serde_json::from_str(&payload_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(error))
    })?;

    Ok(LocalAdaptationEntry {
        entry_id: row.get(0)?,
        subject_ref: row.get(1)?,
        target_kind: LocalAdaptationTargetKind::parse(&target_kind).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                "invalid local adaptation target kind".into(),
            )
        })?,
        key: row.get(3)?,
        payload,
        source_queue_item_id: row.get(5)?,
        created_at: row.get(6)?,
        updated_at: row.get(7)?,
    })
}

fn map_rumination_candidate_row(
    row: &rusqlite::Row<'_>,
) -> Result<RuminationCandidate, rusqlite::Error> {
    let candidate_kind = row.get::<_, String>(2)?;
    let status = row.get::<_, String>(6)?;
    let payload_json = row.get::<_, String>(4)?;
    let evidence_refs_json = row.get::<_, String>(5)?;
    let mut payload: Value = serde_json::from_str(&payload_json).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(4, rusqlite::types::Type::Text, Box::new(error))
    })?;
    let governance_ref_id = payload
        .get("governance_ref_id")
        .and_then(Value::as_str)
        .map(ToString::to_string);

    if let Some(object) = payload.as_object_mut() {
        object.remove("governance_ref_id");
    }

    Ok(RuminationCandidate {
        candidate_id: row.get(0)?,
        source_queue_item_id: row.get(1)?,
        candidate_kind: RuminationCandidateKind::parse(&candidate_kind).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                2,
                rusqlite::types::Type::Text,
                "invalid rumination candidate kind".into(),
            )
        })?,
        subject_ref: row.get(3)?,
        payload,
        evidence_refs: serde_json::from_str(&evidence_refs_json).map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(
                5,
                rusqlite::types::Type::Text,
                Box::new(error),
            )
        })?,
        governance_ref_id,
        status: RuminationCandidateStatus::parse(&status).ok_or_else(|| {
            rusqlite::Error::FromSqlConversionFailure(
                6,
                rusqlite::types::Type::Text,
                "invalid rumination candidate status".into(),
            )
        })?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn queue_table(queue_tier: &str) -> Result<&'static str, RepositoryError> {
    match queue_tier {
        "spq" => Ok("spq_queue_items"),
        "lpq" => Ok("lpq_queue_items"),
        other => Err(RepositoryError::InvalidEnum {
            field: "queue_tier",
            value: other.to_string(),
        }),
    }
}

fn parse_source_kind(value: &str) -> Result<SourceKind, RepositoryError> {
    SourceKind::parse(value).ok_or_else(|| RepositoryError::InvalidEnum {
        field: "source_kind",
        value: value.to_string(),
    })
}

fn parse_scope(value: &str) -> Result<Scope, RepositoryError> {
    Scope::parse(value).ok_or_else(|| RepositoryError::InvalidEnum {
        field: "scope",
        value: value.to_string(),
    })
}

fn parse_record_type(value: &str) -> Result<RecordType, RepositoryError> {
    RecordType::parse(value).ok_or_else(|| RepositoryError::InvalidEnum {
        field: "record_type",
        value: value.to_string(),
    })
}

fn parse_truth_layer(value: &str) -> Result<TruthLayer, RepositoryError> {
    TruthLayer::parse(value).ok_or_else(|| RepositoryError::InvalidEnum {
        field: "truth_layer",
        value: value.to_string(),
    })
}

fn map_fact_dsl_row(row: &rusqlite::Row<'_>) -> Result<PersistedFactDslRecordV1, RepositoryError> {
    Ok(PersistedFactDslRecordV1 {
        record_id: row.get(0)?,
        payload: crate::memory::dsl::FlatFactDslRecordV1 {
            domain: row.get(1)?,
            topic: row.get(2)?,
            aspect: row.get(3)?,
            kind: row.get(4)?,
            claim: row.get(5)?,
            truth_layer: row.get(6)?,
            source_ref: row.get(7)?,
            why: row.get(8)?,
            time: row.get(9)?,
            cond: row.get(10)?,
            impact: row.get(11)?,
            conf: row.get(12)?,
            rel: row
                .get::<_, Option<String>>(13)?
                .map(|value| serde_json::from_str(&value))
                .transpose()?,
        },
        classification_confidence: row.get(14)?,
        needs_review: row.get::<_, i64>(15)? != 0,
    })
}

fn parse_embedding_backend(value: &str) -> Result<EmbeddingBackend, RepositoryError> {
    match value {
        "disabled" => Ok(EmbeddingBackend::Disabled),
        "reserved" => Ok(EmbeddingBackend::Reserved),
        "builtin" => Ok(EmbeddingBackend::Builtin),
        other => Err(RepositoryError::InvalidEnum {
            field: "embedding_backend",
            value: other.to_string(),
        }),
    }
}

fn serialize_rumination_candidate_payload(
    candidate: &RuminationCandidate,
) -> Result<String, RepositoryError> {
    let mut payload = candidate.payload.clone();
    if let Some(governance_ref_id) = &candidate.governance_ref_id {
        match payload.as_object_mut() {
            Some(object) => {
                object.insert(
                    "governance_ref_id".to_string(),
                    Value::String(governance_ref_id.clone()),
                );
            }
            None => {
                payload = serde_json::json!({
                    "value": payload,
                    "governance_ref_id": governance_ref_id,
                });
            }
        }
    }

    serde_json::to_string(&payload).map_err(Into::into)
}
