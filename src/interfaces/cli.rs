use std::{path::PathBuf, process::ExitCode};

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{
    core::{
        app::AppContext,
        config::Config,
        db::Database,
        doctor::{CommandPath, DoctorReport},
        status::StatusReport,
    },
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
        Commands::Inspect { command } => inspect_command(&app, command),
    }
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

    Ok(ExitCode::SUCCESS)
}
