use std::{
    cell::RefCell,
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process::Command,
    rc::Rc,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{SystemTime, UNIX_EPOCH},
};

use agent_memos::{
    agent::{
        orchestration::{
            AgentSearchBranchValue, AgentSearchError, AgentSearchOrchestrator, AgentSearchReport,
            AgentSearchRequest, AgentSearchRunner, AssemblyPort, GatingPort, RetrievalPort,
            RetrievalStepReport, ScoringPort,
        },
        rig_adapter::RigAgentSearchAdapter,
    },
    cognition::{
        action::{ActionBranch, ActionCandidate, ActionKind},
        assembly::{ActionSeed, WorkingMemoryRequest},
        metacog::GateDecision,
        report::{DecisionReport, GateReport, ScoredBranchReport},
        value::{ProjectedScore, ScoredBranch, ValueConfig, ValueVector},
        working_memory::{
            EvidenceFragment, MetacognitiveFlag, PresentFrame, SelfStateSnapshot, TruthContext,
            WorkingMemory,
        },
    },
    core::{
        config::{EmbeddingBackend, EmbeddingConfig, RootRuntimeConfig},
        db::Database,
    },
    ingest::{IngestRequest, IngestService},
    memory::{
        dsl::FlatFactDslRecordV1,
        record::{
            ChunkAnchor, ChunkMetadata, MemoryRecord, Provenance, RecordTimestamp, RecordType,
            Scope, SourceKind, SourceRef, TruthLayer, ValidityWindow,
        },
    },
    search::{
        ChannelContribution, Citation, ResultTrace, ScoreBreakdown, SearchFilters, SearchRequest,
        SearchResponse, SearchResult,
    },
};
use serde_json::Value;

fn unique_temp_dir(name: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock before unix epoch")
        .as_nanos();
    std::env::temp_dir()
        .join("agent-memos-agent-search-tests")
        .join(format!("{name}-{unique}"))
}

fn write_config_with_mode(
    path: &Path,
    db_path: &Path,
    mode: &str,
    backend: &str,
    model: Option<&str>,
    vector_backend: Option<&str>,
) {
    let parent = path.parent().expect("config path should have parent");
    fs::create_dir_all(parent).expect("config parent should exist");
    let model_line = model
        .map(|value| format!("model = \"{value}\"\n"))
        .unwrap_or_default();
    let vector_block = vector_backend
        .map(|backend| {
            format!(
                "\n[vector]\nbackend = \"{backend}\"\ntable = \"object_embeddings_vec\"\nsimilarity = \"cosine\"\n"
            )
        })
        .unwrap_or_default();
    fs::write(
        path,
        format!(
            r#"
db_path = "{}"

[retrieval]
mode = "{mode}"

[embedding]
backend = "{backend}"
{model_line}{vector_block}
"#,
            db_path.display()
        ),
    )
    .expect("config should be written");
}

fn run_cli(config_path: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_agent-memos"))
        .arg("--config")
        .arg(config_path)
        .args(args)
        .output()
        .expect("binary should run")
}

fn stdout(output: &std::process::Output) -> String {
    String::from_utf8(output.stdout.clone()).expect("stdout should be utf-8")
}

fn stderr(output: &std::process::Output) -> String {
    String::from_utf8(output.stderr.clone()).expect("stderr should be utf-8")
}

struct FixtureRecord<'a> {
    source_uri: &'a str,
    source_label: &'a str,
    content: &'a str,
    scope: Scope,
    record_type: RecordType,
    truth_layer: TruthLayer,
    recorded_at: &'a str,
}

fn ingest_record(service: &IngestService<'_>, record: FixtureRecord<'_>) {
    service
        .ingest(IngestRequest {
            source_uri: record.source_uri.to_string(),
            source_label: Some(record.source_label.to_string()),
            source_kind: None,
            content: record.content.to_string(),
            scope: record.scope,
            record_type: record.record_type,
            truth_layer: record.truth_layer,
            recorded_at: record.recorded_at.to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");
}

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
        dsl: None,
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
            channel_contribution: ChannelContribution::LexicalOnly,
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
        provenance: result.record.provenance,
        truth_context: TruthContext {
            truth_layer: TruthLayer::T2,
            t3_state: None,
            open_review_ids: Vec::new(),
            open_candidate_ids: Vec::new(),
        },
        dsl: Some(FlatFactDslRecordV1 {
            domain: "project".to_string(),
            topic: "agent".to_string(),
            aspect: "structure".to_string(),
            kind: "decision".to_string(),
            claim: "rig stays orchestration only".to_string(),
            truth_layer: "t2".to_string(),
            source_ref: source_uri.to_string(),
            why: Some("preserve thin boundaries".to_string()),
            time: None,
            cond: None,
            impact: Some("keeps search and gates inspectable".to_string()),
            conf: None,
            rel: None,
        }),
        trace: result.trace,
        score: result.score,
    }
}

fn sample_fragment_with_query(record_id: &str, source_uri: &str, query: &str) -> EvidenceFragment {
    let record = sample_record(record_id, source_uri);
    let result = sample_result(record, query, "integrated follow-up evidence");
    EvidenceFragment {
        record_id: result.record.id,
        snippet: result.snippet,
        citation: result.citation,
        provenance: result.record.provenance,
        truth_context: TruthContext {
            truth_layer: TruthLayer::T2,
            t3_state: None,
            open_review_ids: Vec::new(),
            open_candidate_ids: Vec::new(),
        },
        dsl: Some(FlatFactDslRecordV1 {
            domain: "project".to_string(),
            topic: "agent".to_string(),
            aspect: "structure".to_string(),
            kind: "observation".to_string(),
            claim: "integrated follow-up evidence".to_string(),
            truth_layer: "t2".to_string(),
            source_ref: source_uri.to_string(),
            why: None,
            time: None,
            cond: None,
            impact: None,
            conf: None,
            rel: None,
        }),
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

fn sample_agent_search_report() -> AgentSearchReport {
    let working_memory = sample_working_memory();
    let selected_branch = ScoredBranchReport {
        branch: working_memory.branches[0].clone(),
        value: ValueVector {
            goal_progress: 0.40,
            information_gain: 0.95,
            risk_avoidance: 0.60,
            resource_efficiency: 0.50,
            agent_robustness: 0.75,
        },
        projected: ProjectedScore {
            final_score: 0.71,
            weight_snapshot: ValueConfig::default(),
        },
    };

    AgentSearchReport {
        working_memory,
        decision: DecisionReport {
            scored_branches: vec![selected_branch.clone()],
            selected_branch: Some(selected_branch),
            gate: GateReport {
                decision: GateDecision::Warning,
                diagnostics: vec!["bounded local orchestration".to_string()],
                rejected_branch: None,
                regulative_branch: None,
                safe_response: None,
                autonomy_paused: false,
            },
            active_risks: vec!["ungated output".to_string()],
            metacog_flags: vec![MetacognitiveFlag {
                code: "trace_required".to_string(),
                detail: Some("all agent output needs citations".to_string()),
            }],
        },
        retrieval_steps: vec![
            RetrievalStepReport {
                query: "rig boundary".to_string(),
                applied_filters: SearchFilters::default(),
                result_count: 1,
                citations: vec![
                    sample_result(
                        sample_record("record-primary", "memo://project/rig-boundary"),
                        "rig boundary",
                        "rig stays orchestration only",
                    )
                    .citation,
                ],
            },
            RetrievalStepReport {
                query: "gate diagnostics".to_string(),
                applied_filters: SearchFilters::default(),
                result_count: 1,
                citations: vec![
                    sample_result(
                        sample_record("record-secondary", "memo://project/gate-diagnostics"),
                        "gate diagnostics",
                        "gate diagnostics must remain typed",
                    )
                    .citation,
                ],
            },
        ],
        citations: vec![
            sample_result(
                sample_record("record-primary", "memo://project/rig-boundary"),
                "rig boundary",
                "rig stays orchestration only",
            )
            .citation,
            sample_result(
                sample_record("record-secondary", "memo://project/gate-diagnostics"),
                "gate diagnostics",
                "gate diagnostics must remain typed",
            )
            .citation,
        ],
        executed_steps: 2,
        step_limit: 3,
    }
}

#[derive(Clone)]
struct ScriptedRetriever {
    calls: Rc<RefCell<Vec<String>>>,
    requests: Rc<RefCell<Vec<SearchRequest>>>,
    responses: HashMap<String, SearchResponse>,
}

impl RetrievalPort for ScriptedRetriever {
    fn search(&self, request: &SearchRequest) -> anyhow::Result<SearchResponse> {
        self.calls
            .borrow_mut()
            .push(format!("search:{}", request.query));
        self.requests.borrow_mut().push(request.clone());
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
        let mut working_memory = self.working_memory.clone();
        if !request.integrated_results.is_empty() {
            let merged_fragments = request
                .integrated_results
                .iter()
                .map(|result| EvidenceFragment {
                    record_id: result.record.id.clone(),
                    snippet: result.snippet.clone(),
                    citation: result.citation.clone(),
                    provenance: result.record.provenance.clone(),
                    truth_context: TruthContext {
                        truth_layer: TruthLayer::T2,
                        t3_state: None,
                        open_review_ids: Vec::new(),
                        open_candidate_ids: Vec::new(),
                    },
                    dsl: None,
                    trace: result.trace.clone(),
                    score: result.score.clone(),
                })
                .collect::<Vec<_>>();
            working_memory.present.world_fragments = merged_fragments.clone();
            if let Some(branch) = working_memory.branches.first_mut() {
                branch.supporting_evidence = merged_fragments;
            }
        }
        Ok(working_memory)
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

#[test]
fn agent_search_does_not_hide_missing_manual_branch_values_with_unique_kind_fallback() {
    let path = unique_temp_dir("manual-branch-value-missing").join("agent-memos.sqlite");
    let db = Database::open(&path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());
    ingest
        .ingest(IngestRequest {
            source_uri: "memo://project/manual-branch-value-missing".to_string(),
            source_label: Some("manual-branch-value-missing".to_string()),
            source_kind: None,
            content: "direct-action evidence exists but the manual branch mapping stays distinct"
                .to_string(),
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-27T11:00:00Z".to_string(),
            valid_from: None,
            valid_to: None,
        })
        .expect("ingest should succeed");

    let request = AgentSearchRequest::new(
        WorkingMemoryRequest::new("direct action evidence").with_action_seed(ActionSeed::new(
            ActionCandidate::new(ActionKind::Instrumental, "ship directly"),
        )),
    )
    .with_max_steps(1)
    .with_working_memory_limit(1)
    .with_branch_value(AgentSearchBranchValue::new(
        ActionKind::Instrumental,
        "take the leading action",
        ValueVector {
            goal_progress: 0.90,
            information_gain: 0.35,
            risk_avoidance: 0.50,
            resource_efficiency: 0.85,
            agent_robustness: 0.65,
        },
    ));

    let error = AgentSearchOrchestrator::with_services(
        db.conn(),
        agent_memos::cognition::assembly::MinimalSelfStateProvider,
        ValueConfig::default(),
    )
    .run(&request)
    .expect_err("manual branches should still require an exact value mapping");

    assert!(
        matches!(error, AgentSearchError::Scoring { .. }),
        "missing manual branch mappings should fail in value scoring: {error}"
    );
    assert!(
        error.to_string().contains("value scoring failed"),
        "agent-search should surface the scoring-stage failure: {error}"
    );
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
    let requests = Rc::new(RefCell::new(Vec::new()));
    let retriever = ScriptedRetriever {
        calls: Rc::clone(&calls),
        requests: Rc::clone(&requests),
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

#[test]
fn orchestrator_integrates_follow_up_evidence_into_working_memory_and_report() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let requests = Rc::new(RefCell::new(Vec::new()));
    let retriever = ScriptedRetriever {
        calls: Rc::clone(&calls),
        requests: Rc::clone(&requests),
        responses: HashMap::from([
            (
                "primary".to_string(),
                SearchResponse {
                    applied_filters: SearchFilters::default(),
                    results: vec![sample_result(
                        sample_record("record-primary", "memo://project/primary"),
                        "primary",
                        "primary evidence",
                    )],
                },
            ),
            (
                "follow-up".to_string(),
                SearchResponse {
                    applied_filters: SearchFilters::default(),
                    results: vec![sample_result(
                        sample_record("record-follow-up", "memo://project/follow-up"),
                        "follow-up",
                        "follow-up evidence",
                    )],
                },
            ),
        ]),
    };
    let assembler = ScriptedAssembler {
        calls: Rc::clone(&calls),
        working_memory: sample_working_memory(),
    };
    let orchestrator = AgentSearchOrchestrator::new(
        retriever,
        assembler,
        ScriptedScorer {
            calls: Rc::clone(&calls),
        },
        ScriptedGate {
            calls: Rc::clone(&calls),
        },
    );

    let report = orchestrator
        .run(
            &AgentSearchRequest::developer_defaults("primary")
                .with_follow_up_query("follow-up")
                .with_max_steps(2),
        )
        .expect("scripted orchestration should succeed");

    let world_ids = report
        .working_memory
        .present
        .world_fragments
        .iter()
        .map(|fragment| fragment.record_id.as_str())
        .collect::<Vec<_>>();
    assert!(
        world_ids.contains(&"record-follow-up"),
        "working memory should include follow-up-only evidence after orchestration integration: {world_ids:?}"
    );
    assert_eq!(report.retrieval_steps.len(), 2);
    assert_eq!(report.retrieval_steps[1].query, "follow-up");
    assert!(
        report
            .citations
            .iter()
            .any(|citation| citation.record_id == "record-follow-up"),
        "top-level citations should still retain follow-up evidence"
    );
}

#[test]
fn integrated_follow_up_evidence_influences_decision_surface() {
    let calls = Rc::new(RefCell::new(Vec::new()));
    let requests = Rc::new(RefCell::new(Vec::new()));
    let retriever = ScriptedRetriever {
        calls: Rc::clone(&calls),
        requests: Rc::clone(&requests),
        responses: HashMap::from([
            (
                "primary".to_string(),
                SearchResponse {
                    applied_filters: SearchFilters::default(),
                    results: vec![sample_result(
                        sample_record("record-primary", "memo://project/primary"),
                        "primary",
                        "primary evidence",
                    )],
                },
            ),
            (
                "follow-up".to_string(),
                SearchResponse {
                    applied_filters: SearchFilters::default(),
                    results: vec![sample_result(
                        sample_record("record-follow-up", "memo://project/follow-up"),
                        "follow-up",
                        "follow-up evidence",
                    )],
                },
            ),
        ]),
    };
    let base_working_memory = sample_working_memory();
    let mut merged_working_memory = base_working_memory.clone();
    let follow_up_fragment =
        sample_fragment_with_query("record-follow-up", "memo://project/follow-up", "follow-up");
    merged_working_memory
        .present
        .world_fragments
        .push(follow_up_fragment.clone());
    merged_working_memory.branches[0]
        .supporting_evidence
        .push(follow_up_fragment);
    let assembler = ScriptedAssembler {
        calls: Rc::clone(&calls),
        working_memory: merged_working_memory,
    };
    let orchestrator = AgentSearchOrchestrator::new(
        retriever,
        assembler,
        ScriptedScorer {
            calls: Rc::clone(&calls),
        },
        ScriptedGate {
            calls: Rc::clone(&calls),
        },
    );

    let report = orchestrator
        .run(
            &AgentSearchRequest::developer_defaults("primary")
                .with_follow_up_query("follow-up")
                .with_max_steps(2),
        )
        .expect("scripted orchestration should succeed");

    let selected_branch = report
        .decision
        .selected_branch
        .as_ref()
        .expect("selected branch should exist");
    assert!(
        selected_branch
            .branch
            .supporting_evidence
            .iter()
            .any(|fragment| fragment.record_id == "record-follow-up"),
        "selected branch should be supported by integrated follow-up evidence"
    );
    assert!(
        report
            .working_memory
            .present
            .world_fragments
            .iter()
            .any(|fragment| fragment.trace.matched_query == "follow-up"),
        "integrated working memory should preserve follow-up query provenance"
    );
}

#[test]
fn agent_search_reuses_ordinary_retrieval_under_dual_channel_modes() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("dual-channel-modes");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "lexical_only",
        "builtin",
        Some(&config.embedding.model),
        Some("sqlite_vec"),
    );

    let db = Database::open(&db_path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some(config.embedding.model.clone()),
            endpoint: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/agent-search-modes",
            source_label: "agent search mode memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T10:00:00Z",
        },
    );

    let lexical_output = run_cli(
        &config_path,
        &[
            "agent-search",
            "citations",
            "--mode",
            "lexical_only",
            "--json",
        ],
    );
    let lexical_stdout = stdout(&lexical_output);
    assert!(
        lexical_output.status.success(),
        "lexical agent-search should succeed: stdout={lexical_stdout} stderr={}",
        stderr(&lexical_output)
    );
    let lexical_json: Value =
        serde_json::from_str(&lexical_stdout).expect("lexical agent-search should emit json");
    assert_eq!(
        lexical_json["working_memory"]["present"]["world_fragments"][0]["trace"]["channel_contribution"],
        "lexical_only"
    );

    let embedding_output = run_cli(
        &config_path,
        &[
            "agent-search",
            "retrieval fusion",
            "--mode",
            "embedding_only",
            "--json",
        ],
    );
    let embedding_stdout = stdout(&embedding_output);
    assert!(
        embedding_output.status.success(),
        "embedding agent-search should succeed: stdout={embedding_stdout} stderr={}",
        stderr(&embedding_output)
    );
    let embedding_json: Value =
        serde_json::from_str(&embedding_stdout).expect("embedding agent-search should emit json");
    assert_eq!(
        embedding_json["working_memory"]["present"]["world_fragments"][0]["trace"]["channel_contribution"],
        "embedding_only"
    );

    let hybrid_output = run_cli(
        &config_path,
        &[
            "agent-search",
            "retrieval fusion",
            "--mode",
            "hybrid",
            "--json",
        ],
    );
    let hybrid_stdout = stdout(&hybrid_output);
    assert!(
        hybrid_output.status.success(),
        "hybrid agent-search should succeed: stdout={hybrid_stdout} stderr={}",
        stderr(&hybrid_output)
    );
    let hybrid_json: Value =
        serde_json::from_str(&hybrid_stdout).expect("hybrid agent-search should emit json");
    assert_eq!(
        hybrid_json["working_memory"]["present"]["world_fragments"][0]["trace"]["channel_contribution"],
        "hybrid"
    );
}

#[test]
fn dual_channel_mode_selection_preserves_agent_report_contract() {
    let config = RootRuntimeConfig::load_from(&PathBuf::from("config.toml"))
        .expect("root config should parse");

    let dir = unique_temp_dir("report-contract");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "lexical_only",
        "builtin",
        Some(&config.embedding.model),
        Some("sqlite_vec"),
    );

    let db = Database::open(&db_path).expect("database should bootstrap");
    let ingest = IngestService::with_embedding_config(
        db.conn(),
        Default::default(),
        EmbeddingConfig {
            backend: EmbeddingBackend::Builtin,
            model: Some(config.embedding.model.clone()),
            endpoint: None,
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/agent-search-contract",
            source_label: "agent search contract memo",
            content: "retrieval fusion semantic retrieval fusion citations",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T11:00:00Z",
        },
    );

    let json_output = run_cli(
        &config_path,
        &[
            "agent-search",
            "retrieval fusion",
            "--mode",
            "hybrid",
            "--follow-up",
            "citations",
            "--json",
        ],
    );
    let json_stdout = stdout(&json_output);
    assert!(
        json_output.status.success(),
        "hybrid agent-search json should succeed: stdout={json_stdout} stderr={}",
        stderr(&json_output)
    );
    let json: Value =
        serde_json::from_str(&json_stdout).expect("hybrid agent-search should emit json");
    assert_eq!(json["executed_steps"], 2);
    assert!(
        json["retrieval_steps"]
            .as_array()
            .is_some_and(|steps| steps.len() == 2),
        "report should preserve multi-step retrieval structure: {json_stdout}"
    );
    assert!(
        json["citations"]
            .as_array()
            .is_some_and(|citations| !citations.is_empty()),
        "report should preserve top-level citations: {json_stdout}"
    );
    assert!(
        json["working_memory"]["present"]["world_fragments"]
            .as_array()
            .is_some_and(|fragments| {
                fragments.iter().any(|fragment| {
                    fragment["trace"]["channel_contribution"] == "hybrid"
                        && fragment["citation"]["source_uri"]
                            == Value::String("memo://project/agent-search-contract".to_string())
                })
            }),
        "working memory should preserve ordinary retrieval trace and citation shape: {json_stdout}"
    );

    let text_output = run_cli(
        &config_path,
        &["agent-search", "retrieval fusion", "--mode", "hybrid"],
    );
    let text = stdout(&text_output);
    assert!(
        text_output.status.success(),
        "hybrid agent-search text should succeed: stdout={text} stderr={}",
        stderr(&text_output)
    );
    assert!(
        text.contains("gate_decision:") && text.contains("memo://project/agent-search-contract"),
        "text report should stay structured and cited after mode selection: {text}"
    );
    assert!(
        text.contains("memory:") && text.contains("/retrieval/"),
        "text report should surface the compressed DSL memory summary: {text}"
    );
}

#[test]
fn agent_search_accepts_taxonomy_filter_flags() {
    let dir = unique_temp_dir("agent-search-taxonomy-flags");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "lexical_only",
        "disabled",
        None,
        None,
    );

    let db = Database::open(&db_path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/agent-taxonomy-retrieval",
            source_label: "agent taxonomy retrieval memo",
            content: "retrieval baseline keeps lexical search explainable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T12:00:00Z",
        },
    );
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/agent-taxonomy-config",
            source_label: "agent taxonomy config memo",
            content: "config baseline keeps toml setting review stable",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T12:05:00Z",
        },
    );

    let output = run_cli(
        &config_path,
        &[
            "agent-search",
            "baseline",
            "--topic",
            "retrieval",
            "--kind",
            "decision",
            "--json",
        ],
    );
    let json_stdout = stdout(&output);
    assert!(
        output.status.success(),
        "agent-search should succeed with taxonomy flags: stdout={json_stdout} stderr={}",
        stderr(&output)
    );
    let json: Value = serde_json::from_str(&json_stdout).expect("agent-search should emit json");
    assert_eq!(
        json["retrieval_steps"][0]["applied_filters"]["topic"],
        "retrieval"
    );
    assert_eq!(
        json["retrieval_steps"][0]["applied_filters"]["kind"],
        "decision"
    );
    assert_eq!(
        json["working_memory"]["present"]["world_fragments"][0]["citation"]["source_uri"],
        "memo://project/agent-taxonomy-retrieval"
    );
    assert_eq!(
        json["working_memory"]["present"]["world_fragments"][0]["trace"]["applied_filters"]["topic"],
        "retrieval"
    );
    assert_eq!(
        json["working_memory"]["present"]["world_fragments"][0]["trace"]["applied_filters"]["kind"],
        "decision"
    );
}

#[test]
fn agent_search_preserves_structured_recall_trace_in_working_memory() {
    let dir = unique_temp_dir("agent-search-structured-trace");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "lexical_only",
        "disabled",
        None,
        None,
    );

    let db = Database::open(&db_path).expect("database should bootstrap");
    let ingest = IngestService::new(db.conn());
    ingest_record(
        &ingest,
        FixtureRecord {
            source_uri: "memo://project/agent-structured-trace",
            source_label: "agent structured trace memo",
            content: "use lexical-first as baseline because explainability matters",
            scope: Scope::Project,
            record_type: RecordType::Decision,
            truth_layer: TruthLayer::T2,
            recorded_at: "2026-04-17T12:10:00Z",
        },
    );

    let output = run_cli(
        &config_path,
        &[
            "agent-search",
            "decision",
            "--mode",
            "lexical_only",
            "--json",
        ],
    );
    let json_stdout = stdout(&output);
    assert!(
        output.status.success(),
        "agent-search should succeed for structured trace coverage: stdout={json_stdout} stderr={}",
        stderr(&output)
    );
    let json: Value = serde_json::from_str(&json_stdout).expect("agent-search should emit json");
    assert_eq!(
        json["working_memory"]["present"]["world_fragments"][0]["trace"]["channel_contribution"],
        "lexical_only"
    );
    assert!(
        json["working_memory"]["present"]["world_fragments"][0]["trace"]["query_strategies"]
            .as_array()
            .is_some_and(|strategies| strategies.iter().any(|value| value == "Structured")),
        "working-memory fragments should preserve structured recall provenance: {json_stdout}"
    );
    assert_eq!(
        json["working_memory"]["present"]["world_fragments"][0]["dsl"]["kind"],
        "decision"
    );
}

#[test]
fn agent_search_rejects_unknown_taxonomy_filter_values() {
    let dir = unique_temp_dir("agent-search-invalid-taxonomy");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "lexical_only",
        "disabled",
        None,
        None,
    );

    let output = run_cli(
        &config_path,
        &[
            "agent-search",
            "baseline",
            "--kind",
            "unknown_kind",
            "--json",
        ],
    );

    assert!(
        !output.status.success(),
        "agent-search should reject unsupported taxonomy filters"
    );
    assert!(
        stderr(&output).contains("unsupported taxonomy kind: unknown_kind"),
        "stderr should explain the invalid taxonomy value: {}",
        stderr(&output)
    );
}

#[test]
fn agent_search_rejects_invalid_domain_topic_combinations() {
    let dir = unique_temp_dir("agent-search-invalid-taxonomy-combo");
    let db_path = dir.join("agent-memos.sqlite");
    let config_path = dir.join("config.toml");
    write_config_with_mode(
        &config_path,
        &db_path,
        "lexical_only",
        "disabled",
        None,
        None,
    );

    let output = run_cli(
        &config_path,
        &[
            "agent-search",
            "baseline",
            "--domain",
            "project",
            "--topic",
            "storage",
            "--json",
        ],
    );

    assert!(
        !output.status.success(),
        "agent-search should reject invalid domain/topic combinations"
    );
    let combined = format!("{}{}", stdout(&output), stderr(&output));
    assert!(
        combined.contains("domain=project does not allow topic=storage"),
        "output should explain the invalid taxonomy combination: {combined}",
    );
}

#[derive(Clone)]
struct CountingRunner {
    calls: Arc<AtomicUsize>,
    report: AgentSearchReport,
}

impl AgentSearchRunner for CountingRunner {
    fn run(
        &self,
        _request: &AgentSearchRequest,
    ) -> Result<AgentSearchReport, agent_memos::agent::orchestration::AgentSearchError> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(self.report.clone())
    }
}

#[tokio::test]
async fn rig_adapter_stays_thin_and_never_bypasses_search_or_truth_gates() {
    let calls = Arc::new(AtomicUsize::new(0));
    let adapter = RigAgentSearchAdapter::new(CountingRunner {
        calls: Arc::clone(&calls),
        report: sample_agent_search_report(),
    });
    let request = AgentSearchRequest::new(WorkingMemoryRequest::new("rig boundary"));

    let report = adapter
        .run(&request)
        .await
        .expect("thin rig adapter should delegate to the internal runner");

    assert_eq!(calls.load(Ordering::SeqCst), 1);
    assert_eq!(adapter.boundary().tool_name, "internal_agent_search");
    assert!(
        !adapter.boundary().allows_truth_write
            && !adapter.boundary().allows_semantic_retrieval
            && !adapter.boundary().allows_rumination,
        "rig adapter must not expose bypass paths around retrieval or governance",
    );

    let rendered_json = agent_memos::interfaces::cli::render_agent_search_report(&report, true)
        .expect("developer surface should render structured json");
    let rendered_text = agent_memos::interfaces::cli::render_agent_search_report(&report, false)
        .expect("developer surface should render structured text");
    let json: Value =
        serde_json::from_str(&rendered_json).expect("rendered report should stay valid json");

    assert_eq!(json["citations"].as_array().map(Vec::len), Some(2));
    assert_eq!(json["decision"]["gate"]["decision"], "warning");
    assert_eq!(json["executed_steps"], 2);
    assert!(
        rendered_text.contains("gate_decision: warning")
            && rendered_text.contains("memo://project/rig-boundary")
            && rendered_text.contains("memory:"),
        "developer-facing surface should stay structured and cited, not freeform only",
    );
}
