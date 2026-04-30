use agent_memos::cognition::value::{ValueAdjustment, ValueConfig};

fn approx_eq(left: f32, right: f32) {
    assert!(
        (left - right).abs() < 0.001,
        "expected {left} to be approximately {right}"
    );
}

fn assert_normalized(config: &ValueConfig) {
    let sum = config.goal_progress
        + config.information_gain
        + config.risk_avoidance
        + config.resource_efficiency
        + config.agent_robustness;
    approx_eq(sum, 1.0);
}

fn assert_within_bounds(config: &ValueConfig) {
    let floor = ValueConfig::WEIGHT_FLOOR;
    let ceiling = ValueConfig::WEIGHT_CEILING;
    for (label, value) in [
        ("goal_progress", config.goal_progress),
        ("information_gain", config.information_gain),
        ("risk_avoidance", config.risk_avoidance),
        ("resource_efficiency", config.resource_efficiency),
        ("agent_robustness", config.agent_robustness),
    ] {
        assert!(
            value >= floor && value <= ceiling,
            "{label}={value} is outside [{floor}, {ceiling}]"
        );
    }
}

#[test]
fn zero_adjustment_produces_identical_default_config() {
    let base = ValueConfig::default();
    let adjustment = ValueAdjustment::zero();

    let result = base.apply_adjustment(&adjustment, 0.01);

    approx_eq(result.goal_progress, base.goal_progress);
    approx_eq(result.information_gain, base.information_gain);
    approx_eq(result.risk_avoidance, base.risk_avoidance);
    approx_eq(result.resource_efficiency, base.resource_efficiency);
    approx_eq(result.agent_robustness, base.agent_robustness);
}

#[test]
fn empty_adjustments_produces_identical_config() {
    let base = ValueConfig::default();
    let result = ValueConfig::from_persisted_adjustments(&base, &[], 0.01);

    assert_eq!(result, base);
}

#[test]
fn single_positive_adjustment_shifts_weights_toward_adjusted_dimensions() {
    let base = ValueConfig::default();
    let adjustment = ValueAdjustment::zero()
        .with_goal_progress(1.0)
        .with_resource_efficiency(1.0);

    let result = base.apply_adjustment(&adjustment, 0.01);

    assert!(
        result.goal_progress > base.goal_progress,
        "goal_progress should increase after positive adjustment"
    );
    assert!(
        result.resource_efficiency > base.resource_efficiency,
        "resource_efficiency should increase after positive adjustment"
    );
    assert_normalized(&result);
    assert_within_bounds(&result);
}

#[test]
fn single_negative_adjustment_shifts_weights_away_from_adjusted_dimensions() {
    let base = ValueConfig::default();
    let adjustment = ValueAdjustment::zero()
        .with_goal_progress(-1.0)
        .with_resource_efficiency(-1.0);

    let result = base.apply_adjustment(&adjustment, 0.01);

    assert!(
        result.goal_progress < base.goal_progress,
        "goal_progress should decrease after negative adjustment"
    );
    assert!(
        result.resource_efficiency < base.resource_efficiency,
        "resource_efficiency should decrease after negative adjustment"
    );
    assert_normalized(&result);
    assert_within_bounds(&result);
}

#[test]
fn weights_always_sum_to_one_after_adjustment() {
    let base = ValueConfig::default();
    let adjustment = ValueAdjustment {
        goal_progress: 0.5,
        information_gain: -0.3,
        risk_avoidance: 0.2,
        resource_efficiency: -0.1,
        agent_robustness: 0.4,
    };

    let result = base.apply_adjustment(&adjustment, 0.5);
    assert_normalized(&result);
}

#[test]
fn weights_are_clamped_to_floor_on_large_negative_adjustment() {
    let base = ValueConfig::default();
    let adjustment = ValueAdjustment {
        goal_progress: -100.0,
        information_gain: -100.0,
        risk_avoidance: -100.0,
        resource_efficiency: -100.0,
        agent_robustness: -100.0,
    };

    let result = base.apply_adjustment(&adjustment, 1.0);

    assert_within_bounds(&result);
    assert_normalized(&result);

    // After clamping at floor and renormalization, all should be equal since
    // all hit the same floor value.
    let floor = ValueConfig::WEIGHT_FLOOR;
    approx_eq(result.goal_progress, floor / (5.0 * floor));
    approx_eq(result.information_gain, floor / (5.0 * floor));
}

#[test]
fn weights_are_clamped_to_ceiling_on_large_positive_adjustment() {
    let base = ValueConfig::default();
    let adjustment = ValueAdjustment {
        goal_progress: 100.0,
        information_gain: 100.0,
        risk_avoidance: 100.0,
        resource_efficiency: 100.0,
        agent_robustness: 100.0,
    };

    let result = base.apply_adjustment(&adjustment, 1.0);

    assert_within_bounds(&result);
    assert_normalized(&result);

    // After clamping at ceiling and renormalization, all should be equal since
    // all hit the same ceiling value.
    let ceiling = ValueConfig::WEIGHT_CEILING;
    approx_eq(result.goal_progress, ceiling / (5.0 * ceiling));
}

#[test]
fn multiple_adjustments_fold_sequentially() {
    let base = ValueConfig::default();
    let adjustments = vec![
        ValueAdjustment::zero().with_goal_progress(1.0),
        ValueAdjustment::zero().with_information_gain(1.0),
    ];

    let result = ValueConfig::from_persisted_adjustments(&base, &adjustments, 0.01);

    assert_normalized(&result);
    assert_within_bounds(&result);

    // After two sequential adjustments, goal should have increased then
    // normalization adjusts all, then info increases.
    assert!(
        result.goal_progress > base.goal_progress,
        "goal_progress should have drifted upward after two adjustments"
    );
}

#[test]
fn learning_rate_controls_magnitude() {
    let base = ValueConfig::default();
    let adjustment = ValueAdjustment::zero().with_goal_progress(1.0);

    let small_lr = base.apply_adjustment(&adjustment, 0.01);
    let large_lr = base.apply_adjustment(&adjustment, 0.1);

    let small_delta = (small_lr.goal_progress - base.goal_progress).abs();
    let large_delta = (large_lr.goal_progress - base.goal_progress).abs();

    assert!(
        large_delta > small_delta,
        "larger learning rate should produce a bigger weight change"
    );
}

#[test]
fn adjustment_round_trips_through_json() {
    let adjustment = ValueAdjustment {
        goal_progress: 0.1,
        information_gain: -0.2,
        risk_avoidance: 0.3,
        resource_efficiency: -0.4,
        agent_robustness: 0.5,
    };

    let json = serde_json::to_string(&adjustment).expect("should serialize");
    let deserialized: ValueAdjustment = serde_json::from_str(&json).expect("should deserialize");

    assert_eq!(deserialized, adjustment);
}

#[test]
fn floor_and_ceiling_constants_match_spec() {
    assert_eq!(ValueConfig::WEIGHT_FLOOR, 0.05);
    assert_eq!(ValueConfig::WEIGHT_CEILING, 0.60);
}

#[test]
fn default_config_sums_to_one() {
    let base = ValueConfig::default();
    assert_normalized(&base);
}

#[test]
fn single_dimension_boost_stays_within_bounds() {
    let base = ValueConfig::default();
    let adjustment = ValueAdjustment::zero().with_goal_progress(10.0);

    let result = base.apply_adjustment(&adjustment, 0.1);

    assert_within_bounds(&result);
    assert_normalized(&result);
    assert!(
        result.goal_progress <= ValueConfig::WEIGHT_CEILING,
        "goal_progress must not exceed ceiling"
    );
}
