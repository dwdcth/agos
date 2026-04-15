use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
};

use agent_memos::{
    agent::orchestration::{
        AgentSearchBranchValue, AgentSearchOrchestrator, AgentSearchRequest, AssemblyPort,
        GatingPort, RetrievalPort, RetrievalStepReport, ScoringPort,
    },
    cognition::{
        action::{ActionBranch, ActionCandidate, ActionKind},
        assembly::WorkingMemoryRequest,
        metacog::GateDecision,
        report::{DecisionReport, GateReport, ScoredBranchReport},
        value::{ProjectedScore, ScoredBranch, ValueConfig, ValueVector},
        working_memory::{
            EvidenceFragment, MetacognitiveFlag, PresentFrame, SelfStateSnapshot, TruthContext,
            WorkingMemory,
        },
    },
    memory::{
        record::{
            ChunkAnchor, ChunkMetadata, MemoryRecord, Provenance, RecordTimestamp, RecordType,
            Scope, SourceKind, SourceRef, TruthLayer, ValidityWindow,
        },
    },
    search::{
        Citation, ResultTrace, ScoreBreakdown, SearchFilters, SearchRequest, SearchResponse,
        SearchResult,
    },
};

fn sample_record(id: &str, source_uri: &str) -> MemoryRecord {
    MemoryRecord {
        id: id.to_string(),
        source: SourceRef {
            uri: source_uri.to_string(),
            kind: SourceKind::Note,
            label: Some(format!("label-{id}")),
        },
        timestamp: RecordTimestamp {
            recorded_at: "2026-04-16T00:00:00Z".to_string(),
            created_at: "2026-04-16T00:00:00Z".to_string(),
            updated_at: "2026-04-16T00:00:00Z".to_string(),
        },
        scope: Scope::Project,
        record_type: RecordType::Decision,
        truth_layer: TruthLayer::T2,
        provenance: Provenance {
            origin: "test".to_string(),
            imported_via: None,
            derived_from: vec!["seed".to_string()],
        },
        content_text: format!("content for {id}"),
        chunk: Some(ChunkMetadata {
            chunk_index: 0,
            chunk_count: 1,
            anchor: ChunkAnchor::LineRange {
                start_line: 1,
                end_line: 3,
            },
            content_hash: format!("hash-{id}"),
        }),
        validity: ValidityWindow {
            valid_from: Some("2026-04-15T00:00:00Z".to_string()),
            valid_to: None,
        },
    }
}

fn sample_result(record: MemoryRecord, query: &str, snippet: &str) -> SearchResult {
    SearchResult {
        citation: Citation::from_record(&record).expect("chunk metadata should exist"),
        record,
        snippet: snippet.to_string(),
        score: ScoreBreakdown {
            lexical_raw: -2.0,
            lexical_base: 0.33,
            keyword_bonus: 0.02,
            importance_bonus: 0.08,
            recency_bonus: 0.03,
            emotion_bonus: 0.0,
            final_score: 0.46,
        },
        trace: ResultTrace {
            matched_query: query.to_string(),
            query_strategies: Vec::new(),
            applied_filters: SearchFilters::default(),
        },
    }
}

fn sample_fragment(record_id: &str, source_uri: &str) -> EvidenceFragment {
    let record = sample_record(record_id, source_uri);
    let result = sample_result(record, "rig boundary", "rig must stay thin");
    EvidenceFragment {
        record_id: result.record.id,
        snippet: result.snippet,
        citation: result.citation,
        truth_context: TruthContext {
            truth_layer: TruthLayer::T2,
            t3_state: None,
            open_review_ids: Vec::new(),
            open_candidate_ids: Vec::new(),
        },
        trace: result.trace,
        score: result.score,
    }
}

fn sample_working_memory() -> WorkingMemory {
    let fragment = sample_fragment("record-primary", "memo://project/rig-boundary");
    let branch = ActionBranch::new(
        ActionCandidate::new(ActionKind::Epistemic, "collect more evidence")
            .with_intent("retrieve more support before acting"),
    )
    .with_supporting_evidence(vec![fragment.clone()]);

    WorkingMemory {
        present: PresentFrame {
            world_fragments: vec![fragment],
            self_state: SelfStateSnapshot {
                task_context: Some("protect the thin rig boundary".to_string()),
                capability_flags: vec!["lexical_search_ready".to_string()],
                readiness_flags: vec!["agent_search_ready".to_string()],
                facts: Vec::new(),
            },
            active_goal: None,
            active_risks: vec!["ungated output".to_string()],
            metacog_flags: vec![MetacognitiveFlag {
                code: "trace_required".to_string(),
                detail: Some("all agent output needs citations".to_string()),
            }],
        },
        branches: vec![branch],
    }
}

#[derive(Clone)]
struct ScriptedRetriever {
    calls: Rc<RefCell<Vec<String>>>,
    responses: HashMap<String, SearchResponse>,
}

impl RetrievalPort for ScriptedRetriever {
    fn search(&self, request: &SearchRequest) -> anyhow::Result<SearchResponse> {
        self.calls
            .borrow_mut()
            .push(format!("search:{}", request.query));
        self.responses
            .get(&request.query)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("missing scripted query {}", request.query))
    }
}

#[derive(Clone)]
struct ScriptedAssembler {
    calls: Rc<RefCell<Vec<String>>>,
    working_memory: WorkingMemory,
}

impl AssemblyPort for ScriptedAssembler {
    fn assemble(&self, request: &WorkingMemoryRequest) -> anyhow::Result<WorkingMemory> {
        self.calls
            .borrow_mut()
            .push(format!("assemble:{}", request.query));
        Ok(self.working_memory.clone())
    }
}

#[derive(Clone)]
struct ScriptedScorer {
    calls: Rc<RefCell<Vec<String>>>,
}

impl ScoringPort for ScriptedScorer {
    fn score(
        &self,
        working_memory: &WorkingMemory,
        branch_values: &[AgentSearchBranchValue],
    ) -> anyhow::Result<Vec<ScoredBranch>> {
        self.calls
            .borrow_mut()
            .push(format!("score:{}", working_memory.branches.len()));

        Ok(vec![ScoredBranch {
            branch: working_memory.branches[0].clone(),
            value: branch_values[0].value.clone(),
            projected: ProjectedScore {
                final_score: 0.71,
                weight_snapshot: ValueConfig::default(),
            },
        }])
    }
}

#[derive(Clone)]
struct ScriptedGate {
    calls: Rc<RefCell<Vec<String>>>,
}

impl GatingPort for ScriptedGate {
    fn evaluate(
        &self,
        working_memory: &WorkingMemory,
        scored_branches: Vec<ScoredBranch>,
    ) -> anyhow::Result<DecisionReport> {
        self.calls
            .borrow_mut()
            .push(format!("gate:{}", scored_branches.len()));

        let selected = ScoredBranchReport::from(scored_branches[0].clone());
        Ok(DecisionReport {
            scored_branches: scored_branches
                .into_iter()
                .map(ScoredBranchReport::from)
                .collect(),
            selected_branch: Some(selected),
            gate: GateReport {
                decision: GateDecision::Warning,
                diagnostics: vec!["bounded local orchestration".to_string()],
                rejected_branch: None,
                regulative_branch: None,
                safe_response: None,
                autonomy_paused: false,
            },
            active_risks: working_memory.present.active_risks.clone(),
            metacog_flags: working_memory.present.metacog_flags.clone(),
        })
    }
}

#[test]
fn orchestrator_reuses_internal_services_and_returns_structured_report() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let retriever = ScriptedRetriever {
        calls: Rc::clone(&calls),
        responses: HashMap::from([
            (
                "rig boundary".to_string(),
                SearchResponse {
                    applied_filters: SearchFilters::default(),
                    results: vec![sample_result(
                        sample_record("record-primary", "memo://project/rig-boundary"),
                        "rig boundary",
                        "rig stays orchestration only",
                    )],
                },
            ),
            (
                "gate diagnostics".to_string(),
                SearchResponse {
                    applied_filters: SearchFilters::default(),
                    results: vec![sample_result(
                        sample_record("record-secondary", "memo://project/gate-diagnostics"),
                        "gate diagnostics",
                        "gate diagnostics must remain typed",
                    )],
                },
            ),
        ]),
    };
    let assembler = ScriptedAssembler {
        calls: Rc::clone(&calls),
        working_memory: sample_working_memory(),
    };
    let scorer = ScriptedScorer {
        calls: Rc::clone(&calls),
    };
    let gate = ScriptedGate {
        calls: Rc::clone(&calls),
    };

    let orchestrator = AgentSearchOrchestrator::new(retriever, assembler, scorer, gate);
    let request = AgentSearchRequest::new(WorkingMemoryRequest::new("rig boundary"))
        .with_follow_up_query("gate diagnostics")
        .with_follow_up_query("ignored extra step")
        .with_max_steps(2)
        .with_step_limit(3)
        .with_branch_value(AgentSearchBranchValue::new(
            ActionKind::Epistemic,
            "collect more evidence",
            ValueVector {
                goal_progress: 0.40,
                information_gain: 0.95,
                risk_avoidance: 0.60,
                resource_efficiency: 0.50,
                agent_robustness: 0.75,
            },
        ));

    let report = orchestrator
        .run(&request)
        .expect("scripted orchestration should succeed");

    assert_eq!(
        calls.borrow().as_slice(),
        &[
            "search:rig boundary".to_string(),
            "search:gate diagnostics".to_string(),
            "assemble:rig boundary".to_string(),
            "score:1".to_string(),
            "gate:1".to_string(),
        ],
        "orchestration should stay bounded and reuse internal ports in order",
    );
    assert_eq!(report.retrieval_steps.len(), 2);
    assert_eq!(report.retrieval_steps[0].query, "rig boundary");
    assert_eq!(report.retrieval_steps[1].query, "gate diagnostics");
    assert_eq!(report.citations.len(), 2);
    assert_eq!(
        report
            .decision
            .selected_branch
            .as_ref()
            .expect("selected branch should stay structured")
            .branch
            .candidate
            .summary,
        "collect more evidence"
    );
    assert_eq!(report.decision.gate.decision, GateDecision::Warning);
    assert!(
        report
            .citations
            .iter()
            .all(|citation| citation.source_uri.starts_with("memo://project/")),
        "structured report should preserve retrieval citations",
    );
    assert!(
        calls.borrow().iter().all(|entry| {
            !entry.contains("semantic") && !entry.contains("rumination") && !entry.contains("write")
        }),
        "task 1 must not introduce semantic retrieval, rumination, or truth writes",
    );

    let RetrievalStepReport { citations, .. } = &report.retrieval_steps[0];
    assert_eq!(citations[0].record_id, "record-primary");
}
