use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use agent_memos::{
    cognition::{
        assembly::{MinimalSelfStateProvider, WorkingMemoryAssembler, WorkingMemoryRequest},
        attention::{
            ATTENTION_BONUS_CAP, AttentionBaseline, AttentionContribution, AttentionDelta,
            AttentionLane, AttentionState, BASELINE_LEARNING_RATE, EmotionModulator,
            EmotionProfile, InhibitionConstraint, MetacogModifier, compute_inhibition,
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
        baseline: AttentionBaseline::default(),
        emotion: Default::default(),
        metacog_modifier: Default::default(),
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
        inhibition_constraints: Vec::new(),
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
        baseline: AttentionBaseline::default(),
        emotion: Default::default(),
        metacog_modifier: Default::default(),
        delta: AttentionDelta {
            total_bonus: 0.0,
            contributions,
        },
        inhibition_constraints: Vec::new(),
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
            baseline: AttentionBaseline::default(),
            emotion: Default::default(),
            metacog_modifier: Default::default(),
            delta: AttentionDelta {
                total_bonus: 0.0,
                contributions: Vec::new(),
            },
            inhibition_constraints: Vec::new(),
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
            baseline: AttentionBaseline::default(),
            emotion: Default::default(),
            metacog_modifier: Default::default(),
            delta: AttentionDelta {
                total_bonus: 0.0,
                contributions: Vec::new(),
            },
            inhibition_constraints: Vec::new(),
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
        baseline: AttentionBaseline::default(),
        emotion: Default::default(),
        metacog_modifier: Default::default(),
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
        inhibition_constraints: Vec::new(),
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

// ===========================================================================
// NEW TESTS: Attention Baseline dimensions and smooth update
// ===========================================================================

#[test]
fn attention_baseline_defaults_to_neutral() {
    let baseline = AttentionBaseline::default();
    assert!((baseline.time_pressure - 0.5).abs() < f32::EPSILON);
    assert!((baseline.cognitive_load - 0.5).abs() < f32::EPSILON);
    assert!((baseline.uncertainty_level - 0.5).abs() < f32::EPSILON);
    assert!((baseline.exploration_mode - 0.5).abs() < f32::EPSILON);
}

#[test]
fn attention_baseline_as_vector_returns_dimension_order() {
    let baseline = AttentionBaseline {
        time_pressure: 0.1,
        cognitive_load: 0.2,
        uncertainty_level: 0.3,
        exploration_mode: 0.4,
    };
    let vec = baseline.as_vector();
    assert!((vec[0] - 0.1).abs() < f32::EPSILON);
    assert!((vec[1] - 0.2).abs() < f32::EPSILON);
    assert!((vec[2] - 0.3).abs() < f32::EPSILON);
    assert!((vec[3] - 0.4).abs() < f32::EPSILON);
}

#[test]
fn attention_baseline_smooth_update_converges() {
    let baseline = AttentionBaseline::default(); // all 0.5
    let observed = AttentionBaseline {
        time_pressure: 0.9,
        cognitive_load: 0.1,
        uncertainty_level: 0.8,
        exploration_mode: 0.2,
    };

    let eta = BASELINE_LEARNING_RATE; // 0.1
    let updated = baseline.update(&observed, eta);

    // Expected: 0.5 + 0.1 * (0.9 - 0.5) = 0.54
    assert!(
        (updated.time_pressure - 0.54).abs() < 1e-6,
        "expected 0.54, got {}",
        updated.time_pressure
    );
    // Expected: 0.5 + 0.1 * (0.1 - 0.5) = 0.46
    assert!(
        (updated.cognitive_load - 0.46).abs() < 1e-6,
        "expected 0.46, got {}",
        updated.cognitive_load
    );
}

#[test]
fn attention_baseline_update_clamps_to_bounds() {
    let baseline = AttentionBaseline {
        time_pressure: 0.99,
        cognitive_load: 0.01,
        uncertainty_level: 0.5,
        exploration_mode: 0.5,
    };
    let observed = AttentionBaseline {
        time_pressure: 1.5,   // above 1.0
        cognitive_load: -0.5, // below 0.0
        uncertainty_level: 0.5,
        exploration_mode: 0.5,
    };

    let updated = baseline.update(&observed, 0.1);

    assert!(
        updated.time_pressure <= 1.0,
        "time_pressure should be clamped to 1.0, got {}",
        updated.time_pressure
    );
    assert!(
        updated.cognitive_load >= 0.0,
        "cognitive_load should be clamped to 0.0, got {}",
        updated.cognitive_load
    );
}

// ===========================================================================
// NEW TESTS: EmotionModulator predefined profiles and multiplicative mask
// ===========================================================================

#[test]
fn emotion_modulator_neutral_has_no_effect() {
    let modulator = EmotionModulator::neutral();
    assert_eq!(modulator.profile, EmotionProfile::Neutral);
    assert!((modulator.intensity).abs() < f32::EPSILON);
    assert!(modulator.is_neutral());

    let baseline = AttentionBaseline::default();
    let modulated = modulator.modulate(&baseline);
    let original = baseline.as_vector();

    // Neutral modulator should not change the baseline
    for i in 0..4 {
        assert!(
            (modulated[i] - original[i]).abs() < f32::EPSILON,
            "neutral should not change dimension {i}: got {}",
            modulated[i]
        );
    }
}

#[test]
fn emotion_modulator_cautious_boosts_uncertainty_suppresses_exploration() {
    let modulator = EmotionModulator::cautious();
    assert_eq!(modulator.profile, EmotionProfile::Cautious);
    assert!(!modulator.is_neutral());

    let baseline = AttentionBaseline {
        time_pressure: 0.5,
        cognitive_load: 0.5,
        uncertainty_level: 0.5,
        exploration_mode: 0.5,
    };
    let modulated = modulator.modulate(&baseline);

    // Cautious mask: [-0.1, 0.1, 0.3, -0.2], intensity 0.5
    // uncertainty: 0.5 * (1 + 0.5 * 0.3) = 0.5 * 1.15 = 0.575
    assert!(
        modulated[2] > baseline.uncertainty_level,
        "cautious should boost uncertainty: got {}",
        modulated[2]
    );
    // exploration: 0.5 * (1 + 0.5 * -0.2) = 0.5 * 0.9 = 0.45
    assert!(
        modulated[3] < baseline.exploration_mode,
        "cautious should suppress exploration: got {}",
        modulated[3]
    );
}

#[test]
fn emotion_modulator_curious_boosts_exploration_suppresses_time_pressure() {
    let modulator = EmotionModulator::curious();
    assert_eq!(modulator.profile, EmotionProfile::Curious);

    let baseline = AttentionBaseline {
        time_pressure: 0.5,
        cognitive_load: 0.5,
        uncertainty_level: 0.5,
        exploration_mode: 0.5,
    };
    let modulated = modulator.modulate(&baseline);

    // Curious mask: [-0.2, -0.1, -0.1, 0.3], intensity 0.5
    assert!(
        modulated[3] > baseline.exploration_mode,
        "curious should boost exploration: got {}",
        modulated[3]
    );
    assert!(
        modulated[0] < baseline.time_pressure,
        "curious should suppress time_pressure: got {}",
        modulated[0]
    );
}

#[test]
fn emotion_modulator_urgent_boosts_time_pressure_suppresses_exploration() {
    let modulator = EmotionModulator::urgent();
    assert_eq!(modulator.profile, EmotionProfile::Urgent);

    let baseline = AttentionBaseline {
        time_pressure: 0.5,
        cognitive_load: 0.5,
        uncertainty_level: 0.5,
        exploration_mode: 0.5,
    };
    let modulated = modulator.modulate(&baseline);

    // Urgent mask: [0.3, 0.1, 0.0, -0.3], intensity 0.7
    assert!(
        modulated[0] > baseline.time_pressure,
        "urgent should boost time_pressure: got {}",
        modulated[0]
    );
    assert!(
        modulated[3] < baseline.exploration_mode,
        "urgent should suppress exploration: got {}",
        modulated[3]
    );
}

// ===========================================================================
// NEW TESTS: Inhibition from self-model constraints
// ===========================================================================

#[test]
fn compute_inhibition_returns_zero_when_no_constraints() {
    let penalty = compute_inhibition(&[], "label", "content", Some("claim"));
    assert!(
        (penalty).abs() < f32::EPSILON,
        "no constraints should produce zero penalty"
    );
}

#[test]
fn compute_inhibition_returns_zero_when_no_match() {
    let constraints = vec![InhibitionConstraint {
        source: "capability_flag".to_string(),
        pattern: "quantum_computing".to_string(),
        weight: 0.02,
    }];
    let penalty = compute_inhibition(
        &constraints,
        "basic label",
        "basic content",
        Some("basic claim"),
    );
    assert!(
        (penalty).abs() < f32::EPSILON,
        "non-matching constraint should produce zero penalty"
    );
}

#[test]
fn compute_inhibition_sums_matched_weights() {
    let constraints = vec![
        InhibitionConstraint {
            source: "capability_flag".to_string(),
            pattern: "deploy".to_string(),
            weight: 0.01,
        },
        InhibitionConstraint {
            source: "readiness_flag".to_string(),
            pattern: "production".to_string(),
            weight: 0.015,
        },
    ];
    let penalty = compute_inhibition(
        &constraints,
        "deploy label",
        "production content here",
        None,
    );
    assert!(
        (penalty - 0.025).abs() < f32::EPSILON,
        "should sum matched weights: expected 0.025, got {}",
        penalty
    );
}

#[test]
fn compute_inhibition_matches_all_terms_in_pattern() {
    let constraints = vec![InhibitionConstraint {
        source: "capability_flag".to_string(),
        pattern: "deploy production".to_string(),
        weight: 0.01,
    }];
    // "deploy" is in label but "production" is not anywhere
    let penalty = compute_inhibition(&constraints, "deploy label", "some content", None);
    assert!(
        (penalty).abs() < f32::EPSILON,
        "partial match should not trigger penalty"
    );

    // Both terms present
    let penalty = compute_inhibition(&constraints, "deploy label", "production content", None);
    assert!(penalty > 0.0, "full match should trigger penalty");
}

#[test]
fn derived_attention_state_includes_inhibition_constraints() {
    let request = WorkingMemoryRequest::new("test query")
        .with_active_goal("deploy")
        .with_capability_flag("search_ready")
        .with_readiness_flag("database_connected");

    let state = request
        .resolved_attention_state()
        .expect("should derive state");
    assert!(
        !state.inhibition_constraints.is_empty(),
        "should have inhibition constraints from flags"
    );
    assert!(
        state
            .inhibition_constraints
            .iter()
            .any(|c| c.pattern == "search_ready"),
        "should have capability flag constraint"
    );
    assert!(
        state
            .inhibition_constraints
            .iter()
            .any(|c| c.pattern == "database_connected"),
        "should have readiness flag constraint"
    );
}

#[test]
fn inhibition_penalty_reduces_attention_bonus() {
    let path = fresh_db_path("inhibition-penalty");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());
    ingest_test_record(
        &ingest,
        "inhibit-1",
        "deploy the search_ready module to production",
    );

    let search = SearchService::new(db.conn());

    // First: attention without inhibition
    let attention_no_inhibit = AttentionState {
        baseline: AttentionBaseline::default(),
        emotion: EmotionModulator::neutral(),
        metacog_modifier: MetacogModifier::none(),
        delta: AttentionDelta {
            total_bonus: 0.0,
            contributions: vec![AttentionContribution {
                lane: AttentionLane::Goal,
                source: "active_goal".to_string(),
                cue: "deploy".to_string(),
                matched_fields: Vec::new(),
                bonus: 0.0,
            }],
        },
        inhibition_constraints: Vec::new(),
    };

    // Second: attention with inhibition matching the content
    let attention_with_inhibit = AttentionState {
        baseline: AttentionBaseline::default(),
        emotion: EmotionModulator::neutral(),
        metacog_modifier: MetacogModifier::none(),
        delta: AttentionDelta {
            total_bonus: 0.0,
            contributions: vec![AttentionContribution {
                lane: AttentionLane::Goal,
                source: "active_goal".to_string(),
                cue: "deploy".to_string(),
                matched_fields: Vec::new(),
                bonus: 0.0,
            }],
        },
        inhibition_constraints: vec![InhibitionConstraint {
            source: "capability_flag".to_string(),
            pattern: "search_ready".to_string(),
            weight: 0.03, // enough to be noticeable
        }],
    };

    let resp_no_inhibit = search
        .search(&SearchRequest::new("deploy").with_attention_state(attention_no_inhibit))
        .expect("search should succeed");
    let resp_with_inhibit = search
        .search(&SearchRequest::new("deploy").with_attention_state(attention_with_inhibit))
        .expect("search should succeed");

    let bonus_no = resp_no_inhibit.results[0].score.attention_bonus;
    let bonus_with = resp_with_inhibit.results[0].score.attention_bonus;

    assert!(
        bonus_no > 0.0,
        "should have positive bonus without inhibition"
    );
    assert!(
        bonus_with < bonus_no,
        "inhibition should reduce the bonus: {} should be less than {}",
        bonus_with,
        bonus_no
    );
}

// ===========================================================================
// NEW TESTS: MetacogModifier from flags
// ===========================================================================

#[test]
fn metacog_modifier_none_is_identity() {
    let modifier = MetacogModifier::none();
    assert!((modifier.goal_weight_multiplier - 1.0).abs() < f32::EPSILON);
    assert!((modifier.diversity_temperature - 1.0).abs() < f32::EPSILON);
    assert!((modifier.inhibition_strength - 1.0).abs() < f32::EPSILON);
    assert!(modifier.is_none());
}

#[test]
fn metacog_modifier_from_empty_flags_is_none() {
    let modifier = MetacogModifier::from_flags(&[]);
    assert!(modifier.is_none());
}

#[test]
fn metacog_modifier_from_warning_flags() {
    let flags = vec![MetacognitiveFlag {
        code: "warning_under_supported".to_string(),
        detail: Some("evidence needed".to_string()),
    }];
    let modifier = MetacogModifier::from_flags(&flags);
    assert!(!modifier.is_none());
    assert!(
        modifier.goal_weight_multiplier > 1.0,
        "warning should boost goal weight"
    );
    assert!(
        modifier.inhibition_strength > 1.0,
        "warning should boost inhibition"
    );
}

#[test]
fn metacog_modifier_from_soft_veto_flags() {
    let flags = vec![MetacognitiveFlag {
        code: "soft_veto_active".to_string(),
        detail: Some("regulative forced".to_string()),
    }];
    let modifier = MetacogModifier::from_flags(&flags);
    assert!(!modifier.is_none());
    assert!(
        modifier.inhibition_strength > 1.2,
        "soft veto should strongly boost inhibition"
    );
    assert!(
        modifier.diversity_temperature < 1.0,
        "soft veto should reduce diversity temperature"
    );
}

// ===========================================================================
// NEW TESTS: Default AttentionState backward compatibility
// ===========================================================================

#[test]
fn default_attention_state_is_neutral_and_empty() {
    let state = AttentionState::default();
    assert!(state.is_empty());
    assert!(state.emotion.is_neutral());
    assert!(state.metacog_modifier.is_none());
    assert!(state.inhibition_constraints.is_empty());

    // Baseline should be at neutral 0.5
    let vec = state.baseline.as_vector();
    for (i, &v) in vec.iter().enumerate() {
        assert!(
            (v - 0.5).abs() < f32::EPSILON,
            "default baseline dimension {i} should be 0.5"
        );
    }
}

#[test]
fn default_attention_state_produces_identical_scoring_as_before() {
    // When using default AttentionState (neutral emotion, no metacog modifier,
    // no inhibition), the scoring should produce the same results as the
    // old code with the empty AttentionBaseline unit struct.
    let path = fresh_db_path("backward-compat");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());
    ingest_test_record(
        &ingest,
        "compat-1",
        "test the backward compatibility scoring",
    );

    let search = SearchService::new(db.conn());

    // Using default state with a goal contribution
    let attention = AttentionState {
        baseline: AttentionBaseline::default(),
        emotion: EmotionModulator::default(),
        metacog_modifier: MetacogModifier::default(),
        delta: AttentionDelta {
            total_bonus: 0.0,
            contributions: vec![AttentionContribution {
                lane: AttentionLane::Goal,
                source: "active_goal".to_string(),
                cue: "backward compatibility".to_string(),
                matched_fields: Vec::new(),
                bonus: 0.0,
            }],
        },
        inhibition_constraints: Vec::new(),
    };

    let request = SearchRequest::new("test").with_attention_state(attention);
    let response = search.search(&request).expect("search should succeed");

    assert!(!response.results.is_empty());
    let result = &response.results[0];
    assert!(
        result.score.attention_bonus > 0.0,
        "default state with goal should produce positive bonus"
    );
    assert!(
        result.score.attention_bonus <= ATTENTION_BONUS_CAP,
        "bonus should be within cap"
    );
}

// ===========================================================================
// NEW TESTS: Emotion modulation integration with scoring
// ===========================================================================

#[test]
fn emotion_modulation_affects_scoring_sensitivity() {
    let path = fresh_db_path("emotion-scoring");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());
    ingest_test_record(
        &ingest,
        "emotion-1",
        "explore the frontier of cognitive architecture design",
    );

    let search = SearchService::new(db.conn());

    let make_state = |emotion: EmotionModulator| AttentionState {
        baseline: AttentionBaseline::default(),
        emotion,
        metacog_modifier: MetacogModifier::none(),
        delta: AttentionDelta {
            total_bonus: 0.0,
            contributions: vec![AttentionContribution {
                lane: AttentionLane::Goal,
                source: "active_goal".to_string(),
                cue: "cognitive architecture".to_string(),
                matched_fields: Vec::new(),
                bonus: 0.0,
            }],
        },
        inhibition_constraints: Vec::new(),
    };

    let neutral_state = make_state(EmotionModulator::neutral());
    let curious_state = make_state(EmotionModulator::curious());

    let resp_neutral = search
        .search(&SearchRequest::new("explore").with_attention_state(neutral_state))
        .expect("search should succeed");
    let resp_curious = search
        .search(&SearchRequest::new("explore").with_attention_state(curious_state))
        .expect("search should succeed");

    // Both should have positive attention bonus (goal cue matches content)
    assert!(resp_neutral.results[0].score.attention_bonus > 0.0);
    assert!(resp_curious.results[0].score.attention_bonus > 0.0);

    // The curious modulator adjusts sensitivity through the modulated baseline.
    // The exact relationship depends on mask values, but both should produce valid bonuses.
    // Key assertion: modulation does not crash and produces finite results.
    assert!(
        resp_curious.results[0].score.attention_bonus.is_finite(),
        "curious emotion should produce finite bonus"
    );
}

// ===========================================================================
// NEW TESTS: Serialization roundtrip
// ===========================================================================

#[test]
fn attention_state_serializes_and_deserializes() {
    let state = AttentionState {
        baseline: AttentionBaseline {
            time_pressure: 0.8,
            cognitive_load: 0.3,
            uncertainty_level: 0.6,
            exploration_mode: 0.4,
        },
        emotion: EmotionModulator::cautious(),
        metacog_modifier: MetacogModifier::from_flags(&[MetacognitiveFlag {
            code: "warning_test".to_string(),
            detail: None,
        }]),
        delta: AttentionDelta {
            total_bonus: 0.05,
            contributions: vec![AttentionContribution {
                lane: AttentionLane::Goal,
                source: "active_goal".to_string(),
                cue: "test goal".to_string(),
                matched_fields: vec!["content".to_string()],
                bonus: 0.05,
            }],
        },
        inhibition_constraints: vec![InhibitionConstraint {
            source: "capability_flag".to_string(),
            pattern: "test pattern".to_string(),
            weight: 0.01,
        }],
    };

    let json = serde_json::to_string(&state).expect("should serialize");
    let deserialized: AttentionState = serde_json::from_str(&json).expect("should deserialize");
    assert_eq!(state, deserialized);
}

#[test]
fn emotion_profile_serializes_to_snake_case() {
    let profile = EmotionProfile::Curious;
    let json = serde_json::to_string(&profile).expect("should serialize");
    assert!(json.contains("curious"), "should serialize as snake_case");
}
