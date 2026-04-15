use anyhow::Result;
use clap::Parser;

use agent_memos::{
    core::{app::AppContext, config::Config},
    interfaces::Cli,
};

fn main() {
    if let Err(error) = run() {
        eprintln!("error: {error}");
        for cause in error.chain().skip(1) {
            eprintln!("  caused by: {cause}");
        }
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();
    let config = match cli.config.as_deref() {
        Some(path) => Config::load_from(path)?,
        None => Config::load()?,
    };
    let _app = AppContext::load(config)?;

    Ok(())
}
