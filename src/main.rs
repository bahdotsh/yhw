use anyhow::Result;
use clap::Parser;

mod cli;
mod manifest;
mod analyzer;
mod tui;
mod utils;

use cli::args::Args;

fn main() -> Result<()> {
    let args = Args::parse();
    
    match args.command {
        cli::args::Command::Analyze { path, dep } => {
            let path = path.unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));
            
            // Start the TUI application
            tui::app::run(path, dep)?;
        }
    }
    
    Ok(())
}
