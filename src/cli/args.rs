use clap::{Parser, Subcommand};
use std::path::PathBuf;

/// A CLI tool to analyze project dependencies and explain why they are needed
#[derive(Parser, Debug)]
#[command(name = "why")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Analyze dependencies in a project
    Analyze {
        /// Path to the project directory (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        
        /// Filter analysis to a specific dependency
        #[arg(short, long)]
        dep: Option<String>,
    },
} 