use std::process::ExitCode;

use anyhow::Result;
use clap::Parser;

use agent_memos::{
    core::config::Config,
    interfaces::{self, Cli},
};

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("error: {error}");
            for cause in error.chain().skip(1) {
                eprintln!("  caused by: {cause}");
            }
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<ExitCode> {
    let cli = Cli::parse();
    let config = match cli.config.as_deref() {
        Some(path) => Config::load_from(path)?,
        None => Config::load()?,
    };
    interfaces::run(cli, config)
}
