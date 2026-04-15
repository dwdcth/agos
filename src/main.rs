use anyhow::Result;
use clap::Parser;

use agent_memos::interfaces::Cli;

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
    let _cli = Cli::parse();

    Ok(())
}
