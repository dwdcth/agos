use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Clone, Parser)]
#[command(name = "agent-memos", about = "Local-first memory kernel for agents")]
pub struct Cli {
    #[arg(long, value_name = "PATH")]
    pub config: Option<PathBuf>,
}
