use std::{path::PathBuf, process::ExitCode};

use anyhow::Result;
use clap::{Parser, Subcommand};
use serde_json::json;

use crate::{
    agent::{
        orchestration::{AgentSearchOrchestrator, AgentSearchReport, AgentSearchRequest},
        rig_adapter::RigAgentSearchAdapter,
    },
    cognition::{assembly::MinimalSelfStateProvider, value::ValueConfig},
    core::{
        app::AppContext,
        config::{Config, RetrievalConfig, RetrievalMode},
        db::Database,
        doctor::{CommandPath, DoctorReport},
        status::StatusReport,
    },
    ingest::{IngestRequest, IngestService},
    memory::{
        record::{RecordType, Scope, SourceKind, TruthLayer},
        taxonomy::{AspectV1, DomainV1, KindV1, TopicV1},
    },
    search::{SearchFilters, SearchRequest, SearchService},
};

#[derive(Debug, Clone, Parser)]
#[command(name = "agent-memos", about = "Local-first memory kernel for agents")]
pub struct Cli {
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Clone, Subcommand)]
pub enum Commands {
    Init,
    Status,
    Doctor,
    Ingest {
        #[arg(long, value_name = "URI")]
        source_uri: String,
        #[arg(long, value_name = "LABEL")]
        source_label: Option<String>,
        #[arg(long, value_name = "KIND", value_parser = parse_source_kind_arg)]
        source_kind: Option<SourceKind>,
        #[arg(long, value_name = "PATH", conflicts_with = "content")]
        path: Option<PathBuf>,
        #[arg(long, value_name = "TEXT", conflicts_with = "path")]
        content: Option<String>,
        #[arg(long, value_name = "SCOPE", default_value = "project", value_parser = parse_scope_arg)]
        scope: Scope,
        #[arg(long = "record-type", value_name = "TYPE", default_value = "observation", value_parser = parse_record_type_arg)]
        record_type: RecordType,
        #[arg(long = "truth-layer", value_name = "LAYER", default_value = "t2", value_parser = parse_truth_layer_arg)]
        truth_layer: TruthLayer,
        #[arg(long = "recorded-at", value_name = "RFC3339")]
        recorded_at: String,
        #[arg(long = "valid-from", value_name = "RFC3339")]
        valid_from: Option<String>,
        #[arg(long = "valid-to", value_name = "RFC3339")]
        valid_to: Option<String>,
        #[arg(long)]
        json: bool,
    },
    Search {
        query: String,
        #[arg(long = "mode", value_name = "MODE", value_parser = parse_retrieval_mode_arg)]
        mode: Option<RetrievalMode>,
        #[arg(long = "top-k", default_value_t = SearchRequest::DEFAULT_LIMIT)]
        top_k: usize,
        #[arg(long = "domain", value_name = "DOMAIN", value_parser = parse_domain_arg)]
        domain: Option<String>,
        #[arg(long = "topic", value_name = "TOPIC", value_parser = parse_topic_arg)]
        topic: Option<String>,
        #[arg(long = "aspect", value_name = "ASPECT", value_parser = parse_aspect_arg)]
        aspect: Option<String>,
        #[arg(long = "kind", value_name = "KIND", value_parser = parse_kind_arg)]
        kind: Option<String>,
        #[arg(long, value_name = "SCOPE", value_parser = parse_scope_arg)]
        scope: Option<Scope>,
        #[arg(long = "record-type", value_name = "TYPE", value_parser = parse_record_type_arg)]
        record_type: Option<RecordType>,
        #[arg(long = "truth-layer", value_name = "LAYER", value_parser = parse_truth_layer_arg)]
        truth_layer: Option<TruthLayer>,
        #[arg(long = "valid-at", value_name = "RFC3339")]
        valid_at: Option<String>,
        #[arg(long = "from", value_name = "RFC3339")]
        from: Option<String>,
        #[arg(long = "to", value_name = "RFC3339")]
        to: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        trace: bool,
    },
    AgentSearch {
        query: String,
        #[arg(long = "mode", value_name = "MODE", value_parser = parse_retrieval_mode_arg)]
        mode: Option<RetrievalMode>,
        #[arg(long = "follow-up")]
        follow_up_queries: Vec<String>,
        #[arg(long = "top-k", default_value_t = SearchRequest::DEFAULT_LIMIT)]
        top_k: usize,
        #[arg(long = "domain", value_name = "DOMAIN", value_parser = parse_domain_arg)]
        domain: Option<String>,
        #[arg(long = "topic", value_name = "TOPIC", value_parser = parse_topic_arg)]
        topic: Option<String>,
        #[arg(long = "aspect", value_name = "ASPECT", value_parser = parse_aspect_arg)]
        aspect: Option<String>,
        #[arg(long = "kind", value_name = "KIND", value_parser = parse_kind_arg)]
        kind: Option<String>,
        #[arg(long = "max-steps", default_value_t = AgentSearchRequest::DEFAULT_MAX_STEPS)]
        max_steps: usize,
        #[arg(long)]
        json: bool,
    },
    Inspect {
        #[command(subcommand)]
        command: InspectCommands,
    },
}

#[derive(Debug, Clone, Subcommand)]
pub enum InspectCommands {
    Schema,
}

pub fn run(cli: Cli, config: Config) -> Result<ExitCode> {
    let app = AppContext::load(config)?;

    match cli.command {
        Commands::Init => init_command(&app),
        Commands::Status => status_command(&app),
        Commands::Doctor => doctor_command(&app),
        Commands::Ingest {
            source_uri,
            source_label,
            source_kind,
            path,
            content,
            scope,
            record_type,
            truth_layer,
            recorded_at,
            valid_from,
            valid_to,
            json,
        } => ingest_command(
            &app,
            IngestCommand {
                source_uri,
                source_label,
                source_kind,
                path,
                content,
                scope,
                record_type,
                truth_layer,
                recorded_at,
                valid_from,
                valid_to,
                json,
            },
        ),
        Commands::Search {
            query,
            mode,
            top_k,
            domain,
            topic,
            aspect,
            kind,
            scope,
            record_type,
            truth_layer,
            valid_at,
            from,
            to,
            json,
            trace,
        } => search_command(
            &app,
            SearchCommand {
                query,
                mode,
                top_k,
                domain,
                topic,
                aspect,
                kind,
                scope,
                record_type,
                truth_layer,
                valid_at,
                from,
                to,
                json,
                trace,
            },
        ),
        Commands::AgentSearch {
            query,
            mode,
            follow_up_queries,
            top_k,
            domain,
            topic,
            aspect,
            kind,
            max_steps,
            json,
        } => agent_search_command(
            &app,
            AgentSearchCommand {
                query,
                mode,
                follow_up_queries,
                top_k,
                domain,
                topic,
                aspect,
                kind,
                max_steps,
                json,
            },
        ),
        Commands::Inspect { command } => inspect_command(&app, command),
    }
}

#[derive(Debug, Clone)]
struct IngestCommand {
    source_uri: String,
    source_label: Option<String>,
    source_kind: Option<SourceKind>,
    path: Option<PathBuf>,
    content: Option<String>,
    scope: Scope,
    record_type: RecordType,
    truth_layer: TruthLayer,
    recorded_at: String,
    valid_from: Option<String>,
    valid_to: Option<String>,
    json: bool,
}

#[derive(Debug, Clone)]
struct SearchCommand {
    query: String,
    mode: Option<RetrievalMode>,
    top_k: usize,
    domain: Option<String>,
    topic: Option<String>,
    aspect: Option<String>,
    kind: Option<String>,
    scope: Option<Scope>,
    record_type: Option<RecordType>,
    truth_layer: Option<TruthLayer>,
    valid_at: Option<String>,
    from: Option<String>,
    to: Option<String>,
    json: bool,
    trace: bool,
}

#[derive(Debug, Clone)]
struct AgentSearchCommand {
    query: String,
    mode: Option<RetrievalMode>,
    follow_up_queries: Vec<String>,
    top_k: usize,
    domain: Option<String>,
    topic: Option<String>,
    aspect: Option<String>,
    kind: Option<String>,
    max_steps: usize,
    json: bool,
}

fn init_command(app: &AppContext) -> Result<ExitCode> {
    let preflight_status = StatusReport::collect(app)?;
    let doctor = DoctorReport::evaluate(&preflight_status, CommandPath::Init);

    if !doctor.ready {
        println!("{}", doctor.render_text());
        return Ok(ExitCode::FAILURE);
    }

    let db = Database::open(app.db_path())?;
    let post_init_status = StatusReport::collect(app)?;
    let post_init_doctor = DoctorReport::evaluate(&post_init_status, CommandPath::Init);
    println!("initialized: true");
    println!("database_path: {}", db.path().display());
    println!("schema_version: {}", db.schema_version()?);

    if !post_init_doctor.warnings.is_empty() {
        println!("warnings:");
        for warning in post_init_doctor.warnings {
            println!("  - {warning}");
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn status_command(app: &AppContext) -> Result<ExitCode> {
    let report = StatusReport::collect(app)?;
    println!("{}", report.render_text());
    Ok(ExitCode::SUCCESS)
}

fn doctor_command(app: &AppContext) -> Result<ExitCode> {
    let status = StatusReport::collect(app)?;
    let report = DoctorReport::evaluate(&status, CommandPath::Doctor);
    println!("{}", report.render_text());

    Ok(if report.ready {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    })
}

fn ingest_command(app: &AppContext, command: IngestCommand) -> Result<ExitCode> {
    if let Some(exit_code) = operational_gate(app, CommandPath::Ingest)? {
        return Ok(exit_code);
    }

    let db = Database::open(app.db_path())?;
    let ingest = IngestService::with_config(db.conn(), &app.config);
    let content = match (command.path.as_ref(), command.content) {
        (Some(path), None) => std::fs::read_to_string(path)?,
        (None, Some(content)) => content,
        (None, None) => anyhow::bail!("either --path or --content must be provided"),
        (Some(_), Some(_)) => anyhow::bail!("--path and --content cannot be used together"),
    };
    let report = ingest.ingest(IngestRequest {
        source_uri: command.source_uri,
        source_label: command.source_label,
        source_kind: command.source_kind,
        content,
        scope: command.scope,
        record_type: command.record_type,
        truth_layer: command.truth_layer,
        recorded_at: command.recorded_at,
        valid_from: command.valid_from,
        valid_to: command.valid_to,
    })?;

    if command.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!(
            "ingested {} chunk(s) from {}",
            report.chunk_count, report.normalized_source.canonical_uri
        );
    }

    Ok(ExitCode::SUCCESS)
}

fn search_command(app: &AppContext, command: SearchCommand) -> Result<ExitCode> {
    let gate_app = override_mode_app(app, command.mode)?;
    let filters = SearchFilters {
        domain: command.domain,
        topic: command.topic,
        aspect: command.aspect,
        kind: command.kind,
        scope: command.scope,
        record_type: command.record_type,
        truth_layer: command.truth_layer,
        valid_at: command.valid_at,
        recorded_from: command.from,
        recorded_to: command.to,
    };
    validate_taxonomy_filter_combination(&filters)?;
    if let Some(exit_code) = operational_gate(&gate_app, CommandPath::Search)? {
        return Ok(exit_code);
    }

    let db = Database::open(app.db_path())?;
    let service = SearchService::with_runtime_config(db.conn(), &gate_app.config, None);
    let response = service.search(
        &SearchRequest::new(command.query)
            .with_limit(command.top_k)
            .with_filters(filters),
    )?;

    if command.json {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        println!("results: {}", response.results.len());
        println!("filters: {}", format_filters(&response.applied_filters));

        for (index, result) in response.results.iter().enumerate() {
            println!("{}. {}", index + 1, result.record.source.uri);
            println!("   snippet: {}", result.snippet);
            println!(
                "   record: id={} kind={} label={} scope={} type={} truth_layer={}",
                result.record.id,
                result.record.source.kind.as_str(),
                result.record.source.label.as_deref().unwrap_or("none"),
                result.record.scope.as_str(),
                result.record.record_type.as_str(),
                result.record.truth_layer.as_str()
            );
            println!(
                "   provenance: origin={} imported_via={} derived_from={}",
                result.record.provenance.origin,
                result
                    .record
                    .provenance
                    .imported_via
                    .as_deref()
                    .unwrap_or("none"),
                if result.record.provenance.derived_from.is_empty() {
                    "none".to_string()
                } else {
                    result.record.provenance.derived_from.join(",")
                }
            );
            if let Some(dsl) = result.dsl.as_ref() {
                let mut dsl_summary = format!(
                    "{}/{}/{}/{} | {}",
                    dsl.domain, dsl.topic, dsl.aspect, dsl.kind, dsl.claim
                );
                dsl_summary.push_str(&format!(" | SRC: {}", dsl.source_ref));
                if let Some(time) = dsl.time.as_deref() {
                    dsl_summary.push_str(&format!(" | TIME: {time}"));
                }
                if let Some(cond) = dsl.cond.as_deref() {
                    dsl_summary.push_str(&format!(" | COND: {cond}"));
                }
                if let Some(why) = dsl.why.as_deref() {
                    dsl_summary.push_str(&format!(" | WHY: {why}"));
                }
                if let Some(impact) = dsl.impact.as_deref() {
                    dsl_summary.push_str(&format!(" | IMPACT: {impact}"));
                }
                println!("   dsl: {}", dsl_summary);
            }
            println!(
                "   channel: {} strategies={}",
                match result.trace.channel_contribution {
                    crate::search::ChannelContribution::LexicalOnly => "lexical_only",
                    crate::search::ChannelContribution::EmbeddingOnly => "embedding_only",
                    crate::search::ChannelContribution::Hybrid => "hybrid",
                },
                result
                    .trace
                    .query_strategies
                    .iter()
                    .map(|strategy| match strategy {
                        crate::search::QueryStrategy::Jieba => "jieba",
                        crate::search::QueryStrategy::Simple => "simple",
                        crate::search::QueryStrategy::Structured => "structured",
                        crate::search::QueryStrategy::Embedding => "embedding",
                    })
                    .collect::<Vec<_>>()
                    .join(",")
            );
            println!(
                "   citation: chunk {}/{} recorded_at={} valid_from={} valid_to={}",
                result.citation.anchor.chunk_index + 1,
                result.citation.anchor.chunk_count,
                result.citation.recorded_at,
                result
                    .citation
                    .validity
                    .valid_from
                    .as_deref()
                    .unwrap_or("none"),
                result
                    .citation
                    .validity
                    .valid_to
                    .as_deref()
                    .unwrap_or("none")
            );
            println!("   final_score: {:.3}", result.score.final_score);
            if command.trace {
                println!(
                    "   lexical_base: {:.3} keyword_bonus: {:.3} importance_bonus: {:.3} recency_bonus: {:.3} emotion_bonus: {:.3}",
                    result.score.lexical_base,
                    result.score.keyword_bonus,
                    result.score.importance_bonus,
                    result.score.recency_bonus,
                    result.score.emotion_bonus
                );
                println!(
                    "   trace: {}",
                    serde_json::to_string(&json!({
                        "matched_query": result.trace.matched_query,
                        "query_strategies": result.trace.query_strategies,
                        "applied_filters": result.trace.applied_filters,
                    }))?
                );
            }
        }
    }

    Ok(ExitCode::SUCCESS)
}

fn agent_search_command(app: &AppContext, command: AgentSearchCommand) -> Result<ExitCode> {
    let gate_app = override_mode_app(app, command.mode)?;
    let mut request = AgentSearchRequest::developer_defaults(command.query)
        .with_working_memory_limit(command.top_k)
        .with_max_steps(command.max_steps);
    let existing_filters = request.working_memory.filters.clone();
    let filters = SearchFilters {
        domain: command.domain,
        topic: command.topic,
        aspect: command.aspect,
        kind: command.kind,
        ..existing_filters
    };
    validate_taxonomy_filter_combination(&filters)?;
    if let Some(exit_code) = operational_gate(&gate_app, CommandPath::AgentSearch)? {
        return Ok(exit_code);
    }

    let db = Database::open(app.db_path())?;
    request.working_memory = request.working_memory.with_filters(filters);
    for query in command.follow_up_queries {
        request = request.with_follow_up_query(query);
    }

    let orchestrator = AgentSearchOrchestrator::with_runtime_config(
        db.conn(),
        MinimalSelfStateProvider,
        ValueConfig::default(),
        &gate_app.config,
    );
    let adapter = RigAgentSearchAdapter::new(orchestrator);
    let runtime = tokio::runtime::Runtime::new()?;
    let report = runtime.block_on(adapter.run(&request))?;

    println!("{}", render_agent_search_report(&report, command.json)?);
    Ok(ExitCode::SUCCESS)
}

fn operational_gate(app: &AppContext, command_path: CommandPath) -> Result<Option<ExitCode>> {
    let status = StatusReport::collect(app)?;
    let doctor = DoctorReport::evaluate(&status, command_path);

    if doctor.ready {
        Ok(None)
    } else {
        println!("{}", doctor.render_text());
        Ok(Some(ExitCode::FAILURE))
    }
}

fn override_mode_app(app: &AppContext, mode: Option<RetrievalMode>) -> Result<AppContext> {
    match mode {
        Some(mode) => {
            let mut config = app.config.clone();
            config.retrieval = RetrievalConfig { mode };
            AppContext::load(config)
        }
        None => Ok(app.clone()),
    }
}

fn inspect_command(app: &AppContext, command: InspectCommands) -> Result<ExitCode> {
    match command {
        InspectCommands::Schema => inspect_schema_command(app),
    }
}

fn inspect_schema_command(app: &AppContext) -> Result<ExitCode> {
    let report = StatusReport::collect(app)?;
    println!("schema:");
    println!("  path: {}", report.db_path.display());
    println!("  database_exists: {}", report.db_path.exists());
    println!("  schema_state: {}", report.schema_state);
    println!(
        "  schema_version: {}",
        report
            .schema_version
            .map(|value| value.to_string())
            .unwrap_or_else(|| "none".to_string())
    );
    println!("  base_table_state: {}", report.base_table_state);
    println!("  index_readiness: {}", report.index_readiness);
    println!(
        "  embedding_index_readiness: {}",
        report.embedding_index_readiness
    );

    Ok(ExitCode::SUCCESS)
}

pub fn render_agent_search_report(report: &AgentSearchReport, json: bool) -> Result<String> {
    if json {
        return Ok(serde_json::to_string_pretty(report)?);
    }

    let selected_branch = report
        .decision
        .selected_branch
        .as_ref()
        .map(|branch| branch.branch.candidate.summary.as_str())
        .unwrap_or("none");
    let mut output = vec![
        format!("executed_steps: {}", report.executed_steps),
        format!("step_limit: {}", report.step_limit),
        format!(
            "gate_decision: {}",
            gate_label(report.decision.gate.decision)
        ),
        format!("selected_branch: {selected_branch}"),
        format!("citations: {}", report.citations.len()),
    ];

    for citation in &report.citations {
        output.push(format!(
            "  - {} [{}:{}]",
            citation.source_uri,
            citation.anchor.chunk_index + 1,
            citation.anchor.chunk_count
        ));
    }

    let dsl_summaries = report
        .working_memory
        .present
        .world_fragments
        .iter()
        .filter_map(|fragment| fragment.dsl.as_ref())
        .map(|dsl| {
            format!(
                "  - {}/{}/{}/{} | {}",
                dsl.domain, dsl.topic, dsl.aspect, dsl.kind, dsl.claim
            )
        })
        .collect::<std::collections::BTreeSet<_>>();
    if !dsl_summaries.is_empty() {
        output.push("memory:".to_string());
        output.extend(dsl_summaries);
    }

    if !report.decision.gate.diagnostics.is_empty() {
        output.push("diagnostics:".to_string());
        for diagnostic in &report.decision.gate.diagnostics {
            output.push(format!("  - {diagnostic}"));
        }
    }

    Ok(output.join("\n"))
}

fn format_filters(filters: &SearchFilters) -> String {
    let scope = filters.scope.map(Scope::as_str).unwrap_or("any");
    let record_type = filters.record_type.map(RecordType::as_str).unwrap_or("any");
    let truth_layer = filters.truth_layer.map(TruthLayer::as_str).unwrap_or("any");
    let domain = filters.domain.as_deref().unwrap_or("any");
    let topic = filters.topic.as_deref().unwrap_or("any");
    let aspect = filters.aspect.as_deref().unwrap_or("any");
    let kind = filters.kind.as_deref().unwrap_or("any");
    let valid_at = filters.valid_at.as_deref().unwrap_or("any");
    let recorded_from = filters.recorded_from.as_deref().unwrap_or("any");
    let recorded_to = filters.recorded_to.as_deref().unwrap_or("any");

    format!(
        "scope={scope}, record_type={record_type}, truth_layer={truth_layer}, domain={domain}, topic={topic}, aspect={aspect}, kind={kind}, valid_at={valid_at}, from={recorded_from}, to={recorded_to}"
    )
}

fn parse_scope_arg(value: &str) -> std::result::Result<Scope, String> {
    Scope::parse(value).ok_or_else(|| format!("unsupported scope: {value}"))
}

fn parse_record_type_arg(value: &str) -> std::result::Result<RecordType, String> {
    RecordType::parse(value).ok_or_else(|| format!("unsupported record type: {value}"))
}

fn parse_truth_layer_arg(value: &str) -> std::result::Result<TruthLayer, String> {
    TruthLayer::parse(value).ok_or_else(|| format!("unsupported truth layer: {value}"))
}

fn parse_source_kind_arg(value: &str) -> std::result::Result<SourceKind, String> {
    SourceKind::parse(value).ok_or_else(|| format!("unsupported source kind: {value}"))
}

fn parse_retrieval_mode_arg(value: &str) -> std::result::Result<RetrievalMode, String> {
    match value {
        "lexical_only" => Ok(RetrievalMode::LexicalOnly),
        "embedding_only" => Ok(RetrievalMode::EmbeddingOnly),
        "hybrid" => Ok(RetrievalMode::Hybrid),
        other => Err(format!("unsupported retrieval mode: {other}")),
    }
}

fn parse_domain_arg(value: &str) -> std::result::Result<String, String> {
    DomainV1::parse(value)
        .map(|value| value.as_str().to_string())
        .ok_or_else(|| format!("unsupported taxonomy domain: {value}"))
}

fn parse_topic_arg(value: &str) -> std::result::Result<String, String> {
    TopicV1::parse(value)
        .map(|value| value.as_str().to_string())
        .ok_or_else(|| format!("unsupported taxonomy topic: {value}"))
}

fn parse_aspect_arg(value: &str) -> std::result::Result<String, String> {
    AspectV1::parse(value)
        .map(|value| value.as_str().to_string())
        .ok_or_else(|| format!("unsupported taxonomy aspect: {value}"))
}

fn parse_kind_arg(value: &str) -> std::result::Result<String, String> {
    KindV1::parse(value)
        .map(|value| value.as_str().to_string())
        .ok_or_else(|| format!("unsupported taxonomy kind: {value}"))
}

fn validate_taxonomy_filter_combination(filters: &SearchFilters) -> Result<()> {
    let Some(domain) = filters.domain.as_deref() else {
        return Ok(());
    };
    let Some(topic) = filters.topic.as_deref() else {
        return Ok(());
    };

    let domain = DomainV1::parse(domain)
        .ok_or_else(|| anyhow::anyhow!("unsupported taxonomy domain: {domain}"))?;
    let topic = TopicV1::parse(topic)
        .ok_or_else(|| anyhow::anyhow!("unsupported taxonomy topic: {topic}"))?;

    if !TopicV1::allowed_for(domain).contains(&topic) {
        anyhow::bail!(
            "unsupported taxonomy combination: domain={} does not allow topic={}",
            domain.as_str(),
            topic.as_str()
        );
    }

    Ok(())
}

fn gate_label(decision: crate::cognition::metacog::GateDecision) -> &'static str {
    match decision {
        crate::cognition::metacog::GateDecision::Warning => "warning",
        crate::cognition::metacog::GateDecision::SoftVeto => "soft_veto",
        crate::cognition::metacog::GateDecision::HardVeto => "hard_veto",
        crate::cognition::metacog::GateDecision::Escalate => "escalate",
    }
}
