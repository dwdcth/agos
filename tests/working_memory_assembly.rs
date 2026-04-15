use agent_memos::cognition::{
    action::{ActionBranch, ActionCandidate, ActionKind},
    working_memory::{
        ActiveGoal, MetacognitiveFlag, PresentFrame, SelfStateFact, SelfStateSnapshot,
        WorkingMemoryBuildError, WorkingMemoryBuilder,
    },
};

fn sample_present_frame() -> PresentFrame {
    PresentFrame {
        world_fragments: Vec::new(),
        self_state: SelfStateSnapshot {
            task_context: Some("stabilize working-memory contracts".to_string()),
            capability_flags: vec!["lexical_search_ready".to_string()],
            readiness_flags: vec!["truth_governance_ready".to_string()],
            facts: vec![SelfStateFact {
                key: "current_focus".to_string(),
                value: "phase_04_plan_01".to_string(),
                source_record_id: None,
            }],
        },
        active_goal: Some(ActiveGoal {
            summary: "assemble an immutable working-memory frame".to_string(),
        }),
        active_risks: vec!["terminology drift".to_string()],
        metacog_flags: vec![MetacognitiveFlag {
            code: "trace_required".to_string(),
            detail: Some("citations must remain attached".to_string()),
        }],
    }
}

#[test]
fn working_memory_builder_requires_present_frame_and_uses_phase4_action_labels() {
    let branch = ActionBranch::new(ActionCandidate::new(
        ActionKind::Instrumental,
        "assemble the present frame",
    ));

    let err = WorkingMemoryBuilder::default()
        .push_branch(branch.clone())
        .build()
        .expect_err("builder should reject incomplete working-memory state");
    assert_eq!(err, WorkingMemoryBuildError::MissingPresentFrame);

    assert_eq!(ActionKind::Epistemic.as_str(), "epistemic");
    assert_eq!(ActionKind::Instrumental.as_str(), "instrumental");
    assert_eq!(ActionKind::Regulative.as_str(), "regulative");
    assert_eq!(ActionKind::parse("operational"), None);
    assert_eq!(ActionKind::parse("regulatory"), None);

    let working_memory = WorkingMemoryBuilder::default()
        .present(sample_present_frame())
        .push_branch(branch)
        .build()
        .expect("builder should produce immutable working memory once present exists");

    assert_eq!(
        working_memory.present.active_goal.unwrap().summary,
        "assemble an immutable working-memory frame"
    );
    assert_eq!(working_memory.branches.len(), 1);
    assert_eq!(
        working_memory.branches[0].candidate.kind,
        ActionKind::Instrumental
    );
}
