use std::{cmp::Ordering, collections::BTreeMap};

use crate::{
    cognition::working_memory::{SelfStateFact, SelfStateSnapshot},
    memory::repository::{
        LocalAdaptationEntry, LocalAdaptationTargetKind, MemoryRepository,
        PersistedSelfModelSnapshot, PersistedSelfModelSnapshotEntry, RepositoryError,
        SelfModelGovernanceMetadata, SelfModelResolutionState,
    },
};

pub use crate::memory::repository::{
    PersistedSelfModelSnapshot as SelfModelSnapshot,
    PersistedSelfModelSnapshotEntry as SelfModelSnapshotEntry,
};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StableSelfKnowledge {
    pub capability_flags: Vec<String>,
    pub facts: Vec<SelfStateFact>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RuntimeSelfState {
    pub task_context: Option<String>,
    pub readiness_flags: Vec<String>,
    pub facts: Vec<SelfStateFact>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProjectedSelfModel {
    pub stable: StableSelfKnowledge,
    pub runtime: RuntimeSelfState,
}

impl ProjectedSelfModel {
    pub fn new(stable: StableSelfKnowledge, runtime: RuntimeSelfState) -> Self {
        Self { stable, runtime }
    }

    pub fn project_snapshot(&self) -> SelfStateSnapshot {
        let mut facts = Vec::with_capacity(self.stable.facts.len() + self.runtime.facts.len());
        facts.extend(self.stable.facts.iter().cloned());
        facts.extend(self.runtime.facts.iter().cloned());

        SelfStateSnapshot {
            task_context: self.runtime.task_context.clone(),
            capability_flags: self.stable.capability_flags.clone(),
            readiness_flags: self.runtime.readiness_flags.clone(),
            facts,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedSelfModelEntry {
    pub target_kind: LocalAdaptationTargetKind,
    pub key: String,
    pub value: String,
    pub active: bool,
    pub governance: Option<SelfModelGovernanceMetadata>,
    pub source_queue_item_id: Option<String>,
    pub updated_at: String,
    pub entry_id: String,
}

impl ResolvedSelfModelEntry {
    pub fn governance_resolution(&self) -> Option<SelfModelResolutionState> {
        self.governance
            .as_ref()
            .map(|governance| governance.resolution)
    }

    fn fact_key(&self) -> String {
        match self.target_kind {
            LocalAdaptationTargetKind::SelfState => format!("self_state:{}", self.key),
            LocalAdaptationTargetKind::RiskBoundary => format!("risk_boundary:{}", self.key),
            LocalAdaptationTargetKind::PrivateT3 => format!("private_t3:{}", self.key),
        }
    }

    fn to_fact(&self) -> SelfStateFact {
        SelfStateFact {
            key: self.fact_key(),
            value: self.value.clone(),
            source_record_id: None,
        }
    }

    fn surfaces_in_runtime(&self) -> bool {
        self.active
            && !matches!(
                self.governance_resolution(),
                Some(SelfModelResolutionState::Unresolved | SelfModelResolutionState::Rejected)
            )
    }

    fn cursor(&self) -> SelfModelCompactionCursor {
        SelfModelCompactionCursor {
            updated_at: self.updated_at.clone(),
            entry_id: self.entry_id.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SelfModelCompactionCursor {
    pub updated_at: String,
    pub entry_id: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SelfModelReadModel {
    entries: Vec<ResolvedSelfModelEntry>,
}

impl SelfModelReadModel {
    pub fn from_entries(
        persisted_entries: &[LocalAdaptationEntry],
        overlay_entries: &[LocalAdaptationEntry],
    ) -> Self {
        Self::from_persisted_state(None, persisted_entries, overlay_entries)
    }

    pub fn from_persisted_state(
        snapshot: Option<&SelfModelSnapshot>,
        persisted_entries: &[LocalAdaptationEntry],
        overlay_entries: &[LocalAdaptationEntry],
    ) -> Self {
        let mut candidates = BTreeMap::<(u8, String), Vec<RankedSelfModelEntry>>::new();

        if let Some(snapshot) = snapshot {
            for (sequence, entry) in snapshot.entries.iter().enumerate() {
                push_snapshot_entry(&mut candidates, entry, sequence);
            }
        }

        for (sequence, entry) in persisted_entries.iter().enumerate() {
            push_entry(&mut candidates, entry, EntrySource::Persisted, sequence);
        }

        for (sequence, entry) in overlay_entries.iter().enumerate() {
            push_entry(&mut candidates, entry, EntrySource::Overlay, sequence);
        }

        let mut entries = candidates
            .into_values()
            .filter_map(resolve_logical_key)
            .collect::<Vec<_>>();
        entries.sort_by(|left, right| {
            target_kind_rank(left.target_kind)
                .cmp(&target_kind_rank(right.target_kind))
                .then_with(|| left.key.cmp(&right.key))
                .then_with(|| left.updated_at.cmp(&right.updated_at))
                .then_with(|| left.entry_id.cmp(&right.entry_id))
        });

        Self { entries }
    }

    pub fn from_overlay_entries(overlay_entries: &[LocalAdaptationEntry]) -> Self {
        Self::from_entries(&[], overlay_entries)
    }

    pub fn entries(&self) -> &[ResolvedSelfModelEntry] {
        &self.entries
    }

    pub fn active_facts(&self) -> Vec<SelfStateFact> {
        self.entries
            .iter()
            .filter(|entry| entry.surfaces_in_runtime())
            .map(ResolvedSelfModelEntry::to_fact)
            .collect()
    }

    pub fn to_snapshot(
        &self,
        subject_ref: impl Into<String>,
        snapshot_id: impl Into<String>,
        compacted_at: impl Into<String>,
        cursor: SelfModelCompactionCursor,
    ) -> SelfModelSnapshot {
        let compacted_at = compacted_at.into();

        PersistedSelfModelSnapshot {
            subject_ref: subject_ref.into(),
            snapshot_id: snapshot_id.into(),
            entries: self
                .entries
                .iter()
                .map(|entry| PersistedSelfModelSnapshotEntry {
                    target_kind: entry.target_kind,
                    key: entry.key.clone(),
                    value: entry.value.clone(),
                    active: entry.active,
                    governance: entry.governance.clone(),
                    source_queue_item_id: entry.source_queue_item_id.clone(),
                    updated_at: entry.updated_at.clone(),
                    entry_id: entry.entry_id.clone(),
                })
                .collect(),
            compacted_through_updated_at: cursor.updated_at,
            compacted_through_entry_id: cursor.entry_id,
            created_at: compacted_at.clone(),
            updated_at: compacted_at,
        }
    }
}

pub fn compact_self_model_subject(
    repository: &MemoryRepository<'_>,
    subject_ref: &str,
    snapshot_id: &str,
    compacted_at: &str,
) -> Result<Option<SelfModelSnapshot>, RepositoryError> {
    let persisted_state = repository.load_self_model_state(subject_ref)?;
    if persisted_state.tail_entries.is_empty() {
        return Ok(None);
    }

    let read_model = SelfModelReadModel::from_persisted_state(
        persisted_state.snapshot.as_ref(),
        &persisted_state.tail_entries,
        &[],
    );
    let cursor = persisted_state
        .snapshot
        .as_ref()
        .map(snapshot_cursor)
        .into_iter()
        .chain(
            persisted_state
                .tail_entries
                .iter()
                .map(resolved_entry_from_local_adaptation)
                .map(|entry| entry.cursor()),
        )
        .max()
        .expect("tail_entries should guarantee a compaction cursor");
    let snapshot = read_model.to_snapshot(subject_ref, snapshot_id, compacted_at, cursor);

    repository.replace_self_model_snapshot(&snapshot)?;
    repository.prune_local_adaptation_entries_through(
        subject_ref,
        &snapshot.compacted_through_updated_at,
        &snapshot.compacted_through_entry_id,
    )?;

    Ok(Some(snapshot))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum EntrySource {
    Persisted,
    Overlay,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct RankedSelfModelEntry {
    resolved: ResolvedSelfModelEntry,
    source: EntrySource,
    sequence: usize,
}

fn push_snapshot_entry(
    candidates: &mut BTreeMap<(u8, String), Vec<RankedSelfModelEntry>>,
    entry: &SelfModelSnapshotEntry,
    sequence: usize,
) {
    let logical_key = (target_kind_rank(entry.target_kind), entry.key.clone());
    candidates
        .entry(logical_key)
        .or_default()
        .push(RankedSelfModelEntry {
            resolved: ResolvedSelfModelEntry {
                target_kind: entry.target_kind,
                key: entry.key.clone(),
                value: entry.value.clone(),
                active: entry.active,
                governance: entry.governance.clone(),
                source_queue_item_id: entry.source_queue_item_id.clone(),
                updated_at: entry.updated_at.clone(),
                entry_id: entry.entry_id.clone(),
            },
            source: EntrySource::Persisted,
            sequence,
        });
}

fn push_entry(
    candidates: &mut BTreeMap<(u8, String), Vec<RankedSelfModelEntry>>,
    entry: &LocalAdaptationEntry,
    source: EntrySource,
    sequence: usize,
) {
    let logical_key = (target_kind_rank(entry.target_kind), entry.key.clone());
    candidates
        .entry(logical_key)
        .or_default()
        .push(RankedSelfModelEntry {
            resolved: resolved_entry_from_local_adaptation(entry),
            source,
            sequence,
        });
}

fn resolved_entry_from_local_adaptation(entry: &LocalAdaptationEntry) -> ResolvedSelfModelEntry {
    let parsed_payload = parse_local_adaptation_payload(entry);
    ResolvedSelfModelEntry {
        target_kind: entry.target_kind,
        key: entry.key.clone(),
        value: display_value(&parsed_payload.effective_value),
        active: value_is_active(&parsed_payload.effective_value),
        governance: parsed_payload.governance,
        source_queue_item_id: entry.source_queue_item_id.clone(),
        updated_at: entry.updated_at.clone(),
        entry_id: entry.entry_id.clone(),
    }
}

fn resolve_logical_key(
    mut candidates: Vec<RankedSelfModelEntry>,
) -> Option<ResolvedSelfModelEntry> {
    candidates.sort_by(|left, right| entry_precedence(right, left));

    let mut resolved = candidates.first()?.resolved.clone();
    let conflicting_candidates = candidates
        .iter()
        .skip(1)
        .filter(|candidate| entry_conflicts_materially(&resolved, &candidate.resolved))
        .collect::<Vec<_>>();
    let conflicting_entry_ids = conflicting_candidates
        .iter()
        .map(|candidate| candidate.resolved.entry_id.clone())
        .collect::<Vec<_>>();
    let governed_conflict_present = conflicting_candidates
        .iter()
        .any(|candidate| candidate.resolved.governance.is_some());
    if !conflicting_entry_ids.is_empty() {
        match resolved.governance.as_mut() {
            Some(governance) if governance.conflicting_entry_ids.is_empty() => {
                governance.conflicting_entry_ids = conflicting_entry_ids;
            }
            Some(_) => {}
            None if governed_conflict_present => {
                resolved.governance = Some(SelfModelGovernanceMetadata {
                    resolution: SelfModelResolutionState::Unresolved,
                    conflicting_entry_ids,
                    review_reason: None,
                });
            }
            None => {}
        }
    }

    Some(resolved)
}

#[derive(Debug)]
struct ParsedLocalAdaptationPayload {
    effective_value: serde_json::Value,
    governance: Option<SelfModelGovernanceMetadata>,
}

fn parse_local_adaptation_payload(entry: &LocalAdaptationEntry) -> ParsedLocalAdaptationPayload {
    let Some(object) = entry.payload.value.as_object() else {
        return ParsedLocalAdaptationPayload {
            effective_value: entry.payload.value.clone(),
            governance: None,
        };
    };

    let Some(governance_value) = object.get("governance") else {
        return ParsedLocalAdaptationPayload {
            effective_value: entry.payload.value.clone(),
            governance: None,
        };
    };

    let governance = parse_governance_metadata(governance_value);
    if let Some(value) = object.get("value") {
        return ParsedLocalAdaptationPayload {
            effective_value: value.clone(),
            governance,
        };
    }

    let mut effective_object = object.clone();
    effective_object.remove("governance");
    ParsedLocalAdaptationPayload {
        effective_value: serde_json::Value::Object(effective_object),
        governance,
    }
}

fn parse_governance_metadata(
    governance_value: &serde_json::Value,
) -> Option<SelfModelGovernanceMetadata> {
    match serde_json::from_value(governance_value.clone()) {
        Ok(governance) => Some(governance),
        Err(_) => Some(SelfModelGovernanceMetadata {
            resolution: SelfModelResolutionState::Unresolved,
            conflicting_entry_ids: Vec::new(),
            review_reason: Some("invalid governance metadata".to_string()),
        }),
    }
}

fn entry_precedence(left: &RankedSelfModelEntry, right: &RankedSelfModelEntry) -> Ordering {
    left.source
        .cmp(&right.source)
        .then_with(|| left.resolved.updated_at.cmp(&right.resolved.updated_at))
        .then_with(|| match (left.source, right.source) {
            (EntrySource::Overlay, EntrySource::Overlay) => left.sequence.cmp(&right.sequence),
            _ => Ordering::Equal,
        })
        .then_with(|| left.resolved.entry_id.cmp(&right.resolved.entry_id))
}

fn entry_conflicts_materially(
    left: &ResolvedSelfModelEntry,
    right: &ResolvedSelfModelEntry,
) -> bool {
    left.value != right.value || left.active != right.active
}

fn display_value(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(value) => value.clone(),
        other => other.to_string(),
    }
}

fn value_is_active(value: &serde_json::Value) -> bool {
    if value.is_null() {
        return false;
    }

    let Some(object) = value.as_object() else {
        return true;
    };

    if matches!(object.get("active"), Some(serde_json::Value::Bool(false))) {
        return false;
    }

    !matches!(
        object
            .get("status")
            .or_else(|| object.get("state"))
            .and_then(serde_json::Value::as_str),
        Some("inactive")
    )
}

fn snapshot_cursor(snapshot: &SelfModelSnapshot) -> SelfModelCompactionCursor {
    SelfModelCompactionCursor {
        updated_at: snapshot.compacted_through_updated_at.clone(),
        entry_id: snapshot.compacted_through_entry_id.clone(),
    }
}

fn target_kind_rank(target_kind: LocalAdaptationTargetKind) -> u8 {
    match target_kind {
        LocalAdaptationTargetKind::SelfState => 0,
        LocalAdaptationTargetKind::RiskBoundary => 1,
        LocalAdaptationTargetKind::PrivateT3 => 2,
    }
}
