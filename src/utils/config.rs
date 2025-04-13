use std::path::{Path, PathBuf};
use std::fs;
use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use toml;

/// Configuration options for the Why CLI tool
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Config {
    /// General configuration options
    #[serde(default)]
    pub general: GeneralConfig,
    
    /// Analysis configuration options
    #[serde(default)]
    pub analysis: AnalysisConfig,
    
    /// Export configuration options
    #[serde(default)]
    pub export: ExportConfig,
    
    /// TUI configuration options
    #[serde(default)]
    pub tui: TuiConfig,
}

/// General configuration options
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GeneralConfig {
    /// Path to the project directory
    pub project_dir: Option<PathBuf>,
    
    /// Whether to include dev dependencies in the analysis
    pub include_dev_dependencies: bool,
    
    /// Whether to include build dependencies in the analysis
    pub include_build_dependencies: bool,
    
    /// Maximum depth to search for project files
    pub max_search_depth: usize,
}

impl Default for GeneralConfig {
    fn default() -> Self {
        Self {
            project_dir: None,
            include_dev_dependencies: true,
            include_build_dependencies: true,
            max_search_depth: 5,
        }
    }
}

/// Analysis configuration options
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AnalysisConfig {
    /// Threshold for considering a dependency as removable
    pub removal_threshold: f64,
    
    /// Number of threads to use for analysis
    pub threads: Option<usize>,
    
    /// Whether to follow symlinks during analysis
    pub follow_symlinks: bool,
    
    /// List of globs to exclude from analysis
    pub exclude_patterns: Vec<String>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            removal_threshold: 0.1,
            threads: None,
            follow_symlinks: false,
            exclude_patterns: vec![
                "**/target/**".to_string(),
                "**/node_modules/**".to_string(),
                "**/.git/**".to_string(),
            ],
        }
    }
}

/// Export configuration options
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ExportConfig {
    /// Default export format
    pub default_format: ExportFormat,
    
    /// Default output directory
    pub output_dir: PathBuf,
}

impl Default for ExportConfig {
    fn default() -> Self {
        Self {
            default_format: ExportFormat::Json,
            output_dir: PathBuf::from("."),
        }
    }
}

/// Export format options
#[derive(Debug, Clone, Copy, Deserialize, Serialize, PartialEq)]
pub enum ExportFormat {
    /// JSON format
    Json,
    /// CSV format
    Csv,
}

/// TUI configuration options
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct TuiConfig {
    /// Whether to use unicode symbols
    pub use_unicode: bool,
    
    /// Whether to show the help bar
    pub show_help_bar: bool,
    
    /// Color scheme
    pub color_scheme: ColorScheme,
}

impl Default for TuiConfig {
    fn default() -> Self {
        Self {
            use_unicode: true,
            show_help_bar: true,
            color_scheme: ColorScheme::default(),
        }
    }
}

/// Color scheme configuration
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ColorScheme {
    /// Primary color
    pub primary: String,
    
    /// Secondary color
    pub secondary: String,
    
    /// Background color
    pub background: String,
    
    /// Highlight color
    pub highlight: String,
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self {
            primary: "#268bd2".to_string(),
            secondary: "#2aa198".to_string(),
            background: "#073642".to_string(),
            highlight: "#d33682".to_string(),
        }
    }
}

impl Config {
    /// Load the configuration from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        // If the file doesn't exist, return the default config
        if !path.exists() {
            return Ok(Self::default());
        }
        
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read config file at {:?}", path))?;
        
        toml::from_str(&content)
            .with_context(|| format!("Failed to parse config file at {:?}", path))
    }
    
    /// Save the configuration to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let path = path.as_ref();
        
        let content = toml::to_string_pretty(self)
            .with_context(|| format!("Failed to serialize config"))?;
        
        fs::write(path, content)
            .with_context(|| format!("Failed to write config file to {:?}", path))?;
        
        Ok(())
    }
    
    /// Create a new configuration with default values
    pub fn create_default<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = Self::default();
        config.save(path)?;
        Ok(config)
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            general: GeneralConfig::default(),
            analysis: AnalysisConfig::default(),
            export: ExportConfig::default(),
            tui: TuiConfig::default(),
        }
    }
} 