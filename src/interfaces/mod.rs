use std::{path::PathBuf, process::ExitCode};

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::{
    core::{
        app::AppContext,
        config::Config,
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
    Status,
    Doctor,
}

pub fn run(cli: Cli, config: Config) -> Result<ExitCode> {
    let app = AppContext::load(config)?;

    match cli.command {
        Commands::Status => status_command(&app),
        Commands::Doctor => doctor_command(&app),
    }
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
