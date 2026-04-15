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
        config::Config,
        db::Database,
        doctor::{CommandPath, DoctorReport},
        status::StatusReport,
    },
    ingest::{IngestRequest, IngestService},
    memory::record::{RecordType, Scope, SourceKind, TruthLayer},
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
        #[arg(long = "top-k", default_value_t = SearchRequest::DEFAULT_LIMIT)]
        top_k: usize,
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
        #[arg(long = "follow-up")]
        follow_up_queries: Vec<String>,
        #[arg(long = "top-k", default_value_t = SearchRequest::DEFAULT_LIMIT)]
        top_k: usize,
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
            top_k,
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
                top_k,
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
            follow_up_queries,
            top_k,
            max_steps,
            json,
        } => agent_search_command(
            &app,
            AgentSearchCommand {
                query,
                follow_up_queries,
                top_k,
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
    top_k: usize,
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
    follow_up_queries: Vec<String>,
    top_k: usize,
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
    let ingest = IngestService::new(db.conn());
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
    if let Some(exit_code) = operational_gate(app, CommandPath::Search)? {
        return Ok(exit_code);
    }

    let db = Database::open(app.db_path())?;
    let service = SearchService::new(db.conn());
    let response = service.search(
        &SearchRequest::new(command.query)
            .with_limit(command.top_k)
            .with_filters(SearchFilters {
                scope: command.scope,
                record_type: command.record_type,
                truth_layer: command.truth_layer,
                valid_at: command.valid_at,
                recorded_from: command.from,
                recorded_to: command.to,
            }),
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
                result.citation.validity.valid_to.as_deref().unwrap_or("none")
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
    if let Some(exit_code) = operational_gate(app, CommandPath::AgentSearch)? {
        return Ok(exit_code);
    }

    let db = Database::open(app.db_path())?;
    let mut request = AgentSearchRequest::developer_defaults(command.query)
        .with_working_memory_limit(command.top_k)
        .with_max_steps(command.max_steps);
    for query in command.follow_up_queries {
        request = request.with_follow_up_query(query);
    }

    let orchestrator = AgentSearchOrchestrator::with_services(
        db.conn(),
        MinimalSelfStateProvider,
        ValueConfig::default(),
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
        format!("gate_decision: {}", gate_label(report.decision.gate.decision)),
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
    let valid_at = filters.valid_at.as_deref().unwrap_or("any");
    let recorded_from = filters.recorded_from.as_deref().unwrap_or("any");
    let recorded_to = filters.recorded_to.as_deref().unwrap_or("any");

    format!(
        "scope={scope}, record_type={record_type}, truth_layer={truth_layer}, valid_at={valid_at}, from={recorded_from}, to={recorded_to}"
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

fn gate_label(decision: crate::cognition::metacog::GateDecision) -> &'static str {
    match decision {
        crate::cognition::metacog::GateDecision::Warning => "warning",
        crate::cognition::metacog::GateDecision::SoftVeto => "soft_veto",
        crate::cognition::metacog::GateDecision::HardVeto => "hard_veto",
        crate::cognition::metacog::GateDecision::Escalate => "escalate",
    }
}
