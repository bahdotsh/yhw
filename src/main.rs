use anyhow::Result;
use clap::Parser;
use std::path::{Path, PathBuf};

mod cli;
mod manifest;
mod analyzer;
mod tui;
mod utils;

use cli::args::{Args, Command, ExportFormat};
use utils::config::Config;

fn main() -> Result<()> {
    let args = Args::parse();
    
    // Load config if specified or use default
    let config = match &args.config {
        Some(config_path) => Config::load(config_path)?,
        None => {
            // Try to load from default locations
            let default_paths = [
                PathBuf::from(".why.toml"),
                dirs::config_dir().map(|p| p.join("why/config.toml")).unwrap_or_default(),
            ];
            
            let mut config = Config::default();
            for path in &default_paths {
                if path.exists() {
                    config = Config::load(path)?;
                    break;
                }
            }
            config
        }
    };
    
    match args.command {
        Command::Analyze { path, dep } => {
            let path = path.or(config.general.project_dir.clone())
                .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));
            
            // Start the TUI application
            tui::app::run(path, dep)?;
        },
        Command::Export { path, output, format, dep } => {
            let path = path.or(config.general.project_dir.clone())
                .unwrap_or_else(|| std::env::current_dir().expect("Failed to get current directory"));
            
            // Perform the analysis
            let analysis = perform_analysis(&path, dep.as_deref())?;
            
            // Export the results
            export_analysis(&analysis, &output, format)?;
            println!("Analysis exported to {}", output.display());
        },
        Command::Config { output } => {
            let output_path = output.unwrap_or_else(|| PathBuf::from(".why.toml"));
            
            // Create a default configuration file
            Config::create_default(&output_path)?;
            println!("Created default configuration file at {}", output_path.display());
        }
    }
    
    Ok(())
}

fn perform_analysis(project_path: &Path, filter_dep: Option<&str>) -> Result<analyzer::Analysis> {
    // Parse manifest
    let manifest = manifest::cargo::parse_cargo_toml(project_path)?;
    
    // Analyze code
    let mut analysis = analyzer::analyze(project_path, &manifest)?;
    
    // Apply filter if specified
    if let Some(dep_name) = filter_dep {
        analysis.filter_dependency(dep_name);
    }
    
    Ok(analysis)
}

fn export_analysis(analysis: &analyzer::Analysis, output_path: &Path, format: ExportFormat) -> Result<()> {
    use std::fs::File;
    use std::io::Write;
    
    let mut file = File::create(output_path)?;
    
    match format {
        ExportFormat::Json => {
            let json = serde_json::to_string_pretty(analysis)?;
            file.write_all(json.as_bytes())?;
        },
        ExportFormat::Csv => {
            let mut wtr = csv::Writer::from_writer(file);
            
            // Write header
            wtr.write_record(&["Dependency", "Version", "Usage Count", "Importance Score", "Removable"])?;
            
            // Write data for each dependency
            for dep in &analysis.dependencies {
                wtr.write_record(&[
                    &dep.name,
                    &dep.version,
                    &dep.usage_count.to_string(),
                    &dep.importance_score.to_string(),
                    &dep.removable.to_string(),
                ])?;
            }
            
            wtr.flush()?;
        }
    }
    
    Ok(())
}
