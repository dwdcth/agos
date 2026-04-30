use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use agent_memos::{
    cognition::{
        assembly::{MinimalSelfStateProvider, WorkingMemoryAssembler, WorkingMemoryRequest},
        attention::{
            ATTENTION_BONUS_CAP, AttentionBaseline, AttentionContribution, AttentionDelta,
            AttentionLane, AttentionState,
        },
        working_memory::MetacognitiveFlag,
    },
    core::db::Database,
    ingest::{IngestRequest, IngestService},
    memory::record::{RecordType, Scope, TruthLayer},
    search::{SearchRequest, SearchService},
};

fn fresh_db_path(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-attention-tests")
        .join(format!("{name}-{unique}"))
        .join("attention.sqlite")
}

fn ingest_test_record(service: &IngestService<'_>, uri_slug: &str, content: &str) -> String {
    let report = service
        .ingest(IngestRequest {
            source_uri: format!("memo://project/{uri_slug}"),
            source_label: Some(format!("label-{uri_slug}")),
            source_kind: None,
            content: content.to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-30T00:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");
    report
        .record_ids
        .into_iter()
        .next()
        .expect("ingest should produce at least one record ID")
}

// ---------------------------------------------------------------------------
// No-attention baseline: search without attention produces zero bonus and null trace
// ---------------------------------------------------------------------------

#[test]
fn no_attention_state_produces_zero_bonus_and_null_trace() {
    let path = fresh_db_path("no-attention-baseline");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());
    ingest_test_record(&ingest, "baseline-1", "lexical search remains explainable");

    let search = SearchService::new(db.conn());
    let request = SearchRequest::new("lexical search");
    let response = search.search(&request).expect("search should succeed");

    assert!(
        !response.results.is_empty(),
        "should find at least one result"
    );
    let result = &response.results[0];
    assert_eq!(
        result.score.attention_bonus, 0.0,
        "no attention state means attention_bonus must be 0.0"
    );
    assert!(
        result.trace.attention.is_none(),
        "no attention state means trace.attention must be None"
    );
}

// ---------------------------------------------------------------------------
// Explicit attention with goal cue matching candidate content
// ---------------------------------------------------------------------------

#[test]
fn explicit_goal_attention_adds_bonus_when_content_matches() {
    let path = fresh_db_path("explicit-goal-attention");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());
    let match_id = ingest_test_record(
        &ingest,
        "goal-match",
        "deploy the attention system to production",
    );
    let unrelated_id = ingest_test_record(
        &ingest,
        "goal-unrelated",
        "unrelated memory about weather and deploy procedures",
    );

    let search = SearchService::new(db.conn());
    let attention = AttentionState {
        baseline: AttentionBaseline,
        delta: AttentionDelta {
            total_bonus: 0.0,
            contributions: vec![AttentionContribution {
                lane: AttentionLane::Goal,
                source: "active_goal".to_string(),
                cue: "attention system".to_string(),
                matched_fields: Vec::new(),
                bonus: 0.0,
            }],
        },
    };
    let request = SearchRequest::new("deploy").with_attention_state(attention);
    let response = search.search(&request).expect("search should succeed");

    assert!(
        response.results.len() >= 2,
        "should find both records, found {}",
        response.results.len()
    );

    // The matching record should have a higher score
    let matched = response
        .results
        .iter()
        .find(|r| r.record.id == match_id)
        .expect("goal-match record should be present");
    assert!(
        matched.score.attention_bonus > 0.0,
        "goal-matching record should receive a positive attention_bonus"
    );
    assert!(
        matched.trace.attention.is_some(),
        "goal-matching record should have an attention trace"
    );

    let trace = matched.trace.attention.as_ref().unwrap();
    assert!(
        trace.total_bonus > 0.0,
        "trace total_bonus should be positive"
    );
    assert!(
        !trace.contributions.is_empty(),
        "trace should have at least one contribution"
    );
    assert_eq!(trace.contributions[0].lane, AttentionLane::Goal);

    // The unrelated record should have zero attention bonus
    let unrelated = response
        .results
        .iter()
        .find(|r| r.record.id == unrelated_id)
        .expect("unrelated record should be present");
    assert_eq!(
        unrelated.score.attention_bonus, 0.0,
        "unrelated record should have zero attention_bonus"
    );
    assert!(
        unrelated.trace.attention.is_none(),
        "unrelated record should have no attention trace"
    );
}

// ---------------------------------------------------------------------------
// Attention bonus is capped at ATTENTION_BONUS_CAP
// ---------------------------------------------------------------------------

#[test]
fn attention_bonus_is_capped_at_maximum() {
    let path = fresh_db_path("attention-cap");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());
    ingest_test_record(
        &ingest,
        "cap-test",
        "production deploy risk capability readiness goal",
    );

    let search = SearchService::new(db.conn());
    // Create many contributions to exceed the cap
    let contributions: Vec<AttentionContribution> = (0..20)
        .map(|_| AttentionContribution {
            lane: AttentionLane::Goal,
            source: "active_goal".to_string(),
            cue: "production".to_string(),
            matched_fields: Vec::new(),
            bonus: 0.0,
        })
        .collect();
    let attention = AttentionState {
        baseline: AttentionBaseline,
        delta: AttentionDelta {
            total_bonus: 0.0,
            contributions,
        },
    };
    let request = SearchRequest::new("production").with_attention_state(attention);
    let response = search.search(&request).expect("search should succeed");

    assert!(!response.results.is_empty());
    let result = &response.results[0];
    assert!(
        result.score.attention_bonus <= ATTENTION_BONUS_CAP,
        "attention_bonus should not exceed the cap: got {}",
        result.score.attention_bonus
    );
}

// ---------------------------------------------------------------------------
// Explicit empty AttentionState means "no attention" (suppresses derived bias)
// ---------------------------------------------------------------------------

#[test]
fn explicit_empty_attention_state_suppresses_bonus() {
    let request = WorkingMemoryRequest::new("test query")
        .with_active_goal("important goal")
        .with_active_risk("some risk")
        .with_attention_state(AttentionState {
            baseline: AttentionBaseline,
            delta: AttentionDelta {
                total_bonus: 0.0,
                contributions: Vec::new(),
            },
        });

    let resolved = request.resolved_attention_state();
    assert!(
        resolved.is_some(),
        "explicit empty state should still return Some (suppressing derived)"
    );
    assert!(
        resolved.unwrap().is_empty(),
        "explicit empty state should be empty"
    );
}

// ---------------------------------------------------------------------------
// Derived attention from WorkingMemoryRequest metadata
// ---------------------------------------------------------------------------

#[test]
fn derived_attention_from_working_memory_metadata() {
    let request = WorkingMemoryRequest::new("test query")
        .with_active_goal("ship the feature")
        .with_active_risk("deadline pressure");

    let resolved = request.resolved_attention_state();
    assert!(
        resolved.is_some(),
        "request with goal and risks should derive attention state"
    );
    let state = resolved.unwrap();
    assert!(!state.is_empty(), "derived state should have contributions");
    assert!(
        state
            .delta
            .contributions
            .iter()
            .any(|c| c.lane == AttentionLane::Goal),
        "should contain a Goal contribution"
    );
    assert!(
        state
            .delta
            .contributions
            .iter()
            .any(|c| c.lane == AttentionLane::Risk),
        "should contain a Risk contribution"
    );
}

#[test]
fn derived_attention_includes_metacog_readiness_and_capability_flags() {
    let request = WorkingMemoryRequest::new("test query")
        .with_metacog_flag(MetacognitiveFlag {
            code: "trace_required".to_string(),
            detail: None,
        })
        .with_readiness_flag("search_ready".to_string())
        .with_capability_flag("lexical_search".to_string());

    let resolved = request.resolved_attention_state();
    assert!(resolved.is_some());
    let state = resolved.unwrap();
    assert!(!state.is_empty());

    let lanes: Vec<AttentionLane> = state.delta.contributions.iter().map(|c| c.lane).collect();
    assert!(lanes.contains(&AttentionLane::Metacog));
    assert!(lanes.contains(&AttentionLane::Readiness));
    assert!(lanes.contains(&AttentionLane::Capability));
}

#[test]
fn no_metadata_produces_no_derived_attention() {
    let request = WorkingMemoryRequest::new("test query");
    let resolved = request.resolved_attention_state();
    assert!(
        resolved.is_none(),
        "request with no metadata should produce no derived attention"
    );
}

// ---------------------------------------------------------------------------
// Integration: WorkingMemoryRequest resolved attention flows through assembly search
// ---------------------------------------------------------------------------

#[test]
fn working_memory_assembly_forwards_resolved_attention_into_search() {
    let path = fresh_db_path("assembly-attention");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());
    ingest_test_record(
        &ingest,
        "assembly-goal",
        "assemble the working memory frame with goal cues",
    );

    let assembler = WorkingMemoryAssembler::new(db.conn(), MinimalSelfStateProvider);
    let request = WorkingMemoryRequest::new("working memory goal")
        .with_active_goal("assemble working memory frame");

    let working_memory = assembler
        .assemble(&request)
        .expect("assembly should succeed");

    let fragment = working_memory
        .present
        .world_fragments
        .first()
        .expect("should have at least one fragment");

    // The fragment should have received attention bonus because goal cue matches content
    assert!(
        fragment.score.attention_bonus > 0.0,
        "assembly should forward resolved attention into search, producing bonus: got {}",
        fragment.score.attention_bonus
    );
    assert!(
        fragment.trace.attention.is_some(),
        "assembly should forward resolved attention, producing a trace"
    );
}

// ---------------------------------------------------------------------------
// Explicit empty AttentionState on WorkingMemoryRequest disables derived fallback
// ---------------------------------------------------------------------------

#[test]
fn assembly_explicit_empty_attention_disables_derived_bias() {
    let path = fresh_db_path("assembly-empty-attention");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());
    ingest_test_record(&ingest, "empty-attn", "some content about goals and risks");

    let assembler = WorkingMemoryAssembler::new(db.conn(), MinimalSelfStateProvider);
    let request = WorkingMemoryRequest::new("goals risks")
        .with_active_goal("important goal")
        .with_active_risk("deadline")
        .with_attention_state(AttentionState {
            baseline: AttentionBaseline,
            delta: AttentionDelta {
                total_bonus: 0.0,
                contributions: Vec::new(),
            },
        });

    let working_memory = assembler
        .assemble(&request)
        .expect("assembly should succeed");
    let fragment = working_memory
        .present
        .world_fragments
        .first()
        .expect("should have at least one fragment");

    assert_eq!(
        fragment.score.attention_bonus, 0.0,
        "explicit empty attention should suppress derived bias"
    );
    assert!(
        fragment.trace.attention.is_none(),
        "explicit empty attention should produce no trace"
    );
}

// ---------------------------------------------------------------------------
// Trace output structure
// ---------------------------------------------------------------------------

#[test]
fn attention_trace_exposes_lane_source_cue_and_matched_fields() {
    let path = fresh_db_path("trace-structure");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());
    let trace_id = ingest_test_record(
        &ingest,
        "trace-1",
        "deploy the attention module for risk mitigation",
    );

    let search = SearchService::new(db.conn());
    let attention = AttentionState {
        baseline: AttentionBaseline,
        delta: AttentionDelta {
            total_bonus: 0.0,
            contributions: vec![AttentionContribution {
                lane: AttentionLane::Risk,
                source: "active_risks".to_string(),
                cue: "risk mitigation".to_string(),
                matched_fields: Vec::new(),
                bonus: 0.0,
            }],
        },
    };
    let request = SearchRequest::new("deploy attention").with_attention_state(attention);
    let response = search.search(&request).expect("search should succeed");

    let result = response
        .results
        .iter()
        .find(|r| r.record.id == trace_id)
        .expect("trace-1 should be present");

    assert!(result.score.attention_bonus > 0.0);
    let trace = result
        .trace
        .attention
        .as_ref()
        .expect("trace should be present");
    assert_eq!(trace.contributions.len(), 1);
    let contribution = &trace.contributions[0];
    assert_eq!(contribution.lane, AttentionLane::Risk);
    assert_eq!(contribution.source, "active_risks");
    assert!(contribution.cue.contains("risk"));
    assert!(!contribution.matched_fields.is_empty());
    assert!(contribution.bonus > 0.0);
    assert_eq!(trace.total_bonus, result.score.attention_bonus);
}
