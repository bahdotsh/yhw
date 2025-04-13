use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

/// A CLI tool to analyze project dependencies and explain why they are needed
#[derive(Parser, Debug)]
#[command(name = "why")]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Path to the configuration file
    #[arg(short, long, global = true)]
    pub config: Option<PathBuf>,
    
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, ValueEnum)]
pub enum ExportFormat {
    /// Export as JSON format
    Json,
    /// Export as CSV format
    Csv,
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
        
        /// Enable dependency graph visualization
        #[arg(long)]
        deps: bool,
    },
    
    /// Export dependency analysis to a file
    Export {
        /// Path to the project directory (defaults to current directory)
        #[arg(short, long)]
        path: Option<PathBuf>,
        
        /// Output file path
        #[arg(short, long, required = true)]
        output: PathBuf,
        
        /// Export format
        #[arg(short, long, value_enum, default_value_t = ExportFormat::Json)]
        format: ExportFormat,
        
        /// Filter export to a specific dependency
        #[arg(short, long)]
        dep: Option<String>,
    },
    
    /// Generate a default configuration file
    Config {
        /// Path to save the configuration file (defaults to .why.toml in current directory)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
} 