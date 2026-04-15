use std::collections::BTreeSet;

use rusqlite::Connection;
use serde::Serialize;
use serde_json::{Value, json};
use thiserror::Error;

use crate::{
    agent::orchestration::AgentSearchReport,
    cognition::{metacog::GateDecision, report::DecisionReport},
    memory::repository::{
        MemoryRepository, PersistedRuminationQueueItem, PersistedRuminationTriggerState,
        RepositoryError, RuminationQueueStatus,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuminationQueueTier {
    Spq,
    Lpq,
}

impl RuminationQueueTier {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Spq => "spq",
            Self::Lpq => "lpq",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RuminationTriggerKind {
    ActionFailure,
    UserCorrection,
    MetacogVeto,
    SessionBoundary,
    EvidenceAccumulation,
    IdleWindow,
    AbnormalPatternAccumulation,
}

impl RuminationTriggerKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ActionFailure => "action_failure",
            Self::UserCorrection => "user_correction",
            Self::MetacogVeto => "metacog_veto",
            Self::SessionBoundary => "session_boundary",
            Self::EvidenceAccumulation => "evidence_accumulation",
            Self::IdleWindow => "idle_window",
            Self::AbnormalPatternAccumulation => "abnormal_pattern_accumulation",
        }
    }

    pub fn queue_tier(self) -> RuminationQueueTier {
        match self {
            Self::ActionFailure | Self::UserCorrection | Self::MetacogVeto => {
                RuminationQueueTier::Spq
            }
            Self::SessionBoundary
            | Self::EvidenceAccumulation
            | Self::IdleWindow
            | Self::AbnormalPatternAccumulation => RuminationQueueTier::Lpq,
        }
    }

    fn default_priority(self) -> i64 {
        match self {
            Self::MetacogVeto => 100,
            Self::UserCorrection => 90,
            Self::ActionFailure => 80,
            Self::AbnormalPatternAccumulation => 70,
            Self::EvidenceAccumulation => 65,
            Self::SessionBoundary => 60,
            Self::IdleWindow => 50,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RuminationTriggerEvent {
    pub kind: RuminationTriggerKind,
    pub subject_ref: String,
    pub occurred_at: String,
    pub dedupe_key: String,
    pub cooldown_key: String,
    pub budget_bucket: String,
    pub cooldown_until: Option<String>,
    pub budget_cost: u32,
    pub payload: Value,
    pub evidence_refs: Vec<String>,
    pub source_report_ref: Option<String>,
    pub source_report: Option<Value>,
}

impl RuminationTriggerEvent {
    pub fn new(
        kind: RuminationTriggerKind,
        subject_ref: impl Into<String>,
        occurred_at: impl Into<String>,
        budget_bucket: impl Into<String>,
        source_report_ref: Option<String>,
    ) -> Self {
        let subject_ref = subject_ref.into();
        let dedupe_key = make_dedupe_key(kind, &subject_ref, source_report_ref.as_deref());

        Self {
            kind,
            subject_ref,
            occurred_at: occurred_at.into(),
            dedupe_key: dedupe_key.clone(),
            cooldown_key: dedupe_key,
            budget_bucket: budget_bucket.into(),
            cooldown_until: None,
            budget_cost: 1,
            payload: json!({}),
            evidence_refs: Vec::new(),
            source_report_ref,
            source_report: None,
        }
    }

    pub fn with_cooldown_until(mut self, cooldown_until: impl Into<String>) -> Self {
        self.cooldown_until = Some(cooldown_until.into());
        self
    }

    pub fn with_dedupe_key(mut self, dedupe_key: impl Into<String>) -> Self {
        self.dedupe_key = dedupe_key.into();
        self
    }

    pub fn with_cooldown_key(mut self, cooldown_key: impl Into<String>) -> Self {
        self.cooldown_key = cooldown_key.into();
        self
    }

    pub fn with_budget_cost(mut self, budget_cost: u32) -> Self {
        self.budget_cost = budget_cost.max(1);
        self
    }

    pub fn with_payload(mut self, payload: Value) -> Self {
        self.payload = payload;
        self
    }

    pub fn with_evidence_refs(mut self, evidence_refs: Vec<String>) -> Self {
        self.evidence_refs = evidence_refs;
        self
    }

    pub fn with_source_report(mut self, source_report: Value) -> Self {
        self.source_report = Some(source_report);
        self
    }

    pub fn from_decision_report(
        kind: RuminationTriggerKind,
        subject_ref: impl Into<String>,
        report: &DecisionReport,
        occurred_at: impl Into<String>,
        budget_bucket: impl Into<String>,
        cooldown_until: Option<String>,
        source_report_ref: Option<String>,
    ) -> Result<Self, RuminationServiceError> {
        let source_report = serde_json::to_value(report)?;
        let evidence_refs = unique_strings(
            report
                .selected_branch
                .iter()
                .flat_map(|branch| branch.branch.supporting_evidence.iter())
                .map(|fragment| fragment.record_id.clone()),
        );
        let payload = json!({
            "gate_decision": report.gate.decision,
            "diagnostics": report.gate.diagnostics,
            "active_risks": report.active_risks,
            "metacog_flags": report.metacog_flags,
        });

        let mut event = Self::new(
            kind,
            subject_ref,
            occurred_at,
            budget_bucket,
            source_report_ref,
        )
        .with_payload(payload)
        .with_evidence_refs(evidence_refs)
        .with_source_report(source_report);

        if let Some(cooldown_until) = cooldown_until {
            event = event.with_cooldown_until(cooldown_until);
        }

        Ok(event)
    }

    pub fn from_agent_search_report(
        kind: RuminationTriggerKind,
        subject_ref: impl Into<String>,
        report: &AgentSearchReport,
        occurred_at: impl Into<String>,
        budget_bucket: impl Into<String>,
        cooldown_until: Option<String>,
        source_report_ref: Option<String>,
    ) -> Result<Self, RuminationServiceError> {
        let source_report = serde_json::to_value(report)?;
        let evidence_refs = unique_strings(
            report
                .citations
                .iter()
                .map(|citation| citation.record_id.clone()),
        );
        let payload = json!({
            "executed_steps": report.executed_steps,
            "step_limit": report.step_limit,
            "gate_decision": report.decision.gate.decision,
            "active_risks": report.decision.active_risks,
            "metacog_flags": report.decision.metacog_flags,
            "citation_count": report.citations.len(),
        });

        let mut event = Self::new(
            kind,
            subject_ref,
            occurred_at,
            budget_bucket,
            source_report_ref,
        )
        .with_payload(payload)
        .with_evidence_refs(evidence_refs)
        .with_source_report(source_report);

        if let Some(cooldown_until) = cooldown_until {
            event = event.with_cooldown_until(cooldown_until);
        }

        Ok(event)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct RuminationQueueItem {
    pub queue_tier: RuminationQueueTier,
    pub item_id: String,
    pub trigger_kind: RuminationTriggerKind,
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
    pub payload: Value,
    pub evidence_refs: Vec<String>,
    pub source_report: Option<Value>,
    pub last_error: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    pub processed_at: Option<String>,
}

impl RuminationQueueItem {
    fn from_event(event: RuminationTriggerEvent) -> Self {
        let queue_tier = event.kind.queue_tier();
        Self {
            queue_tier,
            item_id: make_item_id(queue_tier, &event.dedupe_key, &event.occurred_at),
            trigger_kind: event.kind,
            status: RuminationQueueStatus::Queued,
            subject_ref: event.subject_ref,
            dedupe_key: event.dedupe_key,
            cooldown_key: event.cooldown_key,
            budget_bucket: event.budget_bucket,
            priority: event.kind.default_priority(),
            budget_cost: event.budget_cost,
            attempt_count: 0,
            cooldown_until: event.cooldown_until,
            next_eligible_at: event.occurred_at.clone(),
            payload: event.payload,
            evidence_refs: event.evidence_refs,
            source_report: event.source_report,
            last_error: None,
            created_at: event.occurred_at.clone(),
            updated_at: event.occurred_at,
            processed_at: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RuminationTriggerDecision {
    Enqueued {
        tier: RuminationQueueTier,
        item_id: String,
    },
    Deduped {
        tier: RuminationQueueTier,
        dedupe_key: String,
    },
    CooldownBlocked {
        tier: RuminationQueueTier,
        cooldown_until: String,
    },
    BudgetBlocked {
        tier: RuminationQueueTier,
        budget_bucket: String,
        spent: u32,
        limit: u32,
    },
}

impl RuminationTriggerDecision {
    fn as_storage_str(&self) -> &'static str {
        match self {
            Self::Enqueued { .. } => "enqueued",
            Self::Deduped { .. } => "deduped",
            Self::CooldownBlocked { .. } => "cooldown_blocked",
            Self::BudgetBlocked { .. } => "budget_blocked",
        }
    }
}

#[derive(Debug, Error)]
pub enum RuminationServiceError {
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    #[error(transparent)]
    Json(#[from] serde_json::Error),
    #[error("unsupported gate decision for rumination routing: {0:?}")]
    UnsupportedGateDecision(GateDecision),
}

pub struct RuminationService<'db> {
    repository: MemoryRepository<'db>,
    spq_budget_limit: u32,
    lpq_budget_limit: u32,
}

impl<'db> RuminationService<'db> {
    pub fn new(conn: &'db Connection) -> Self {
        Self::with_budget_limits(conn, 3, 3)
    }

    pub fn with_budget_limits(
        conn: &'db Connection,
        spq_budget_limit: u32,
        lpq_budget_limit: u32,
    ) -> Self {
        Self {
            repository: MemoryRepository::new(conn),
            spq_budget_limit: spq_budget_limit.max(1),
            lpq_budget_limit: lpq_budget_limit.max(1),
        }
    }

    pub fn schedule(
        &self,
        event: RuminationTriggerEvent,
    ) -> Result<RuminationTriggerDecision, RuminationServiceError> {
        let queue_tier = event.kind.queue_tier();
        let queue_tier_str = queue_tier.as_str();

        if let Some(active_item) = self
            .repository
            .find_active_rumination_item(queue_tier_str, &event.dedupe_key)?
        {
            let decision = RuminationTriggerDecision::Deduped {
                tier: queue_tier,
                dedupe_key: event.dedupe_key.clone(),
            };
            self.persist_trigger_state(
                &event,
                &decision,
                Some(active_item.item_id),
                self.repository
                    .get_rumination_trigger_state(queue_tier_str, &event.dedupe_key)?
                    .map_or(0, |state| state.budget_spent),
            )?;
            return Ok(decision);
        }

        if let Some(cooldown_until) = self
            .repository
            .get_latest_rumination_cooldown(queue_tier_str, &event.cooldown_key)?
            .filter(|cooldown_until| cooldown_until > &event.occurred_at)
        {
            let decision = RuminationTriggerDecision::CooldownBlocked {
                tier: queue_tier,
                cooldown_until,
            };
            self.persist_trigger_state(&event, &decision, None, 0)?;
            return Ok(decision);
        }

        let budget_spent = self
            .repository
            .total_rumination_budget_spent(queue_tier_str, &event.budget_bucket)?;
        let budget_limit = self.budget_limit(queue_tier);
        if budget_spent.saturating_add(event.budget_cost) > budget_limit {
            let decision = RuminationTriggerDecision::BudgetBlocked {
                tier: queue_tier,
                budget_bucket: event.budget_bucket.clone(),
                spent: budget_spent,
                limit: budget_limit,
            };
            self.persist_trigger_state(&event, &decision, None, budget_spent)?;
            return Ok(decision);
        }

        let item = RuminationQueueItem::from_event(event.clone());
        self.repository
            .insert_rumination_queue_item(&to_persisted_item(&item))?;

        let decision = RuminationTriggerDecision::Enqueued {
            tier: queue_tier,
            item_id: item.item_id.clone(),
        };
        self.persist_trigger_state(
            &event,
            &decision,
            Some(item.item_id.clone()),
            budget_spent.saturating_add(item.budget_cost),
        )?;

        Ok(decision)
    }

    pub fn claim_next_ready(
        &self,
        now: &str,
    ) -> Result<Option<RuminationQueueItem>, RuminationServiceError> {
        self.repository
            .claim_next_rumination_item(now)?
            .map(from_persisted_item)
            .transpose()
    }

    pub fn complete(
        &self,
        item: &RuminationQueueItem,
        processed_at: &str,
    ) -> Result<(), RuminationServiceError> {
        self.repository.complete_rumination_queue_item(
            item.queue_tier.as_str(),
            &item.item_id,
            processed_at,
        )?;

        Ok(())
    }

    pub fn retry(
        &self,
        item: &RuminationQueueItem,
        next_eligible_at: &str,
        last_error: &str,
        updated_at: &str,
    ) -> Result<(), RuminationServiceError> {
        self.repository.retry_rumination_queue_item(
            item.queue_tier.as_str(),
            &item.item_id,
            next_eligible_at,
            last_error,
            updated_at,
        )?;

        Ok(())
    }

    fn budget_limit(&self, tier: RuminationQueueTier) -> u32 {
        match tier {
            RuminationQueueTier::Spq => self.spq_budget_limit,
            RuminationQueueTier::Lpq => self.lpq_budget_limit,
        }
    }

    fn persist_trigger_state(
        &self,
        event: &RuminationTriggerEvent,
        decision: &RuminationTriggerDecision,
        last_item_id: Option<String>,
        budget_spent: u32,
    ) -> Result<(), RuminationServiceError> {
        let state = PersistedRuminationTriggerState {
            queue_tier: event.kind.queue_tier().as_str().to_string(),
            trigger_kind: event.kind.as_str().to_string(),
            dedupe_key: event.dedupe_key.clone(),
            cooldown_key: event.cooldown_key.clone(),
            budget_bucket: event.budget_bucket.clone(),
            budget_window_started_at: Some(event.budget_bucket.clone()),
            budget_spent,
            cooldown_until: event.cooldown_until.clone(),
            last_enqueued_at: match decision {
                RuminationTriggerDecision::Enqueued { .. } => Some(event.occurred_at.clone()),
                _ => None,
            },
            last_seen_at: event.occurred_at.clone(),
            last_decision: decision.as_storage_str().to_string(),
            last_item_id,
            updated_at: event.occurred_at.clone(),
        };
        self.repository.upsert_rumination_trigger_state(&state)?;

        Ok(())
    }
}

fn to_persisted_item(item: &RuminationQueueItem) -> PersistedRuminationQueueItem {
    PersistedRuminationQueueItem {
        queue_tier: item.queue_tier.as_str().to_string(),
        item_id: item.item_id.clone(),
        trigger_kind: item.trigger_kind.as_str().to_string(),
        status: item.status,
        subject_ref: item.subject_ref.clone(),
        dedupe_key: item.dedupe_key.clone(),
        cooldown_key: item.cooldown_key.clone(),
        budget_bucket: item.budget_bucket.clone(),
        priority: item.priority,
        budget_cost: item.budget_cost,
        attempt_count: item.attempt_count,
        cooldown_until: item.cooldown_until.clone(),
        next_eligible_at: item.next_eligible_at.clone(),
        payload_json: item.payload.clone(),
        evidence_refs_json: Some(item.evidence_refs.clone()),
        source_report_json: item.source_report.clone(),
        last_error: item.last_error.clone(),
        created_at: item.created_at.clone(),
        updated_at: item.updated_at.clone(),
        processed_at: item.processed_at.clone(),
    }
}

fn from_persisted_item(
    item: PersistedRuminationQueueItem,
) -> Result<RuminationQueueItem, RuminationServiceError> {
    Ok(RuminationQueueItem {
        queue_tier: match item.queue_tier.as_str() {
            "spq" => RuminationQueueTier::Spq,
            "lpq" => RuminationQueueTier::Lpq,
            other => {
                return Err(RuminationServiceError::Repository(
                    RepositoryError::InvalidEnum {
                        field: "queue_tier",
                        value: other.to_string(),
                    },
                ));
            }
        },
        item_id: item.item_id,
        trigger_kind: match item.trigger_kind.as_str() {
            "action_failure" => RuminationTriggerKind::ActionFailure,
            "user_correction" => RuminationTriggerKind::UserCorrection,
            "metacog_veto" => RuminationTriggerKind::MetacogVeto,
            "session_boundary" => RuminationTriggerKind::SessionBoundary,
            "evidence_accumulation" => RuminationTriggerKind::EvidenceAccumulation,
            "idle_window" => RuminationTriggerKind::IdleWindow,
            "abnormal_pattern_accumulation" => {
                RuminationTriggerKind::AbnormalPatternAccumulation
            }
            other => {
                return Err(RuminationServiceError::Repository(
                    RepositoryError::InvalidEnum {
                        field: "trigger_kind",
                        value: other.to_string(),
                    },
                ));
            }
        },
        status: item.status,
        subject_ref: item.subject_ref,
        dedupe_key: item.dedupe_key,
        cooldown_key: item.cooldown_key,
        budget_bucket: item.budget_bucket,
        priority: item.priority,
        budget_cost: item.budget_cost,
        attempt_count: item.attempt_count,
        cooldown_until: item.cooldown_until,
        next_eligible_at: item.next_eligible_at,
        payload: item.payload_json,
        evidence_refs: item.evidence_refs_json.unwrap_or_default(),
        source_report: item.source_report_json,
        last_error: item.last_error,
        created_at: item.created_at,
        updated_at: item.updated_at,
        processed_at: item.processed_at,
    })
}

fn make_dedupe_key(
    kind: RuminationTriggerKind,
    subject_ref: &str,
    source_report_ref: Option<&str>,
) -> String {
    match source_report_ref {
        Some(source_report_ref) => {
            format!("{}:{subject_ref}:{source_report_ref}", kind.as_str())
        }
        None => format!("{}:{subject_ref}", kind.as_str()),
    }
}

fn make_item_id(queue_tier: RuminationQueueTier, dedupe_key: &str, occurred_at: &str) -> String {
    format!("{}:{dedupe_key}:{occurred_at}", queue_tier.as_str())
}

fn unique_strings<I>(values: I) -> Vec<String>
where
    I: IntoIterator<Item = String>,
{
    values
        .into_iter()
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
