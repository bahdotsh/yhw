pub mod rust_analyzer;
pub mod metrics;

use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::manifest::cargo::CargoDependency;

/// Main analyzer that orchestrates the analysis process
pub struct DependencyAnalyzer {
    project_path: PathBuf,
}

impl DependencyAnalyzer {
    /// Create a new analyzer for the given project path
    pub fn new<P: AsRef<Path>>(project_path: P) -> Self {
        Self {
            project_path: project_path.as_ref().to_path_buf(),
        }
    }
    
    /// Analyze a project to find dependency usage
    pub fn analyze(&self) -> Result<AnalysisResult> {
        // Find manifest file
        let manifest_path = self.find_manifest_file()?;
        
        // Parse manifest file
        let dependencies = self.parse_manifest(&manifest_path)?;
        
        // Analyze code
        let usage_data = self.analyze_code(&dependencies)?;
        
        // Calculate metrics
        let metrics = self.calculate_metrics(&dependencies, &usage_data)?;
        
        Ok(AnalysisResult {
            dependencies,
            usage_data,
            metrics,
        })
    }
    
    fn find_manifest_file(&self) -> Result<PathBuf> {
        let cargo_toml = self.project_path.join("Cargo.toml");
        if cargo_toml.exists() {
            return Ok(cargo_toml);
        }
        
        Err(anyhow::anyhow!("No supported manifest file found in {:?}", self.project_path))
    }
    
    fn parse_manifest(&self, manifest_path: &Path) -> Result<Vec<CargoDependency>> {
        use crate::manifest::{get_parser, ManifestParser};
        
        let parser = get_parser(manifest_path)?;
        // We need to cast the returned dependencies to the concrete type
        // This is a limitation of using trait objects with associated types
        let deps = parser.parse(manifest_path)?;
        
        Ok(deps.into_iter()
            .map(|dep| dep)
            .collect())
    }
    
    fn analyze_code(&self, dependencies: &[CargoDependency]) -> Result<DependencyUsageData> {
        let analyzer = rust_analyzer::RustAnalyzer::new(&self.project_path);
        analyzer.analyze(dependencies)
    }
    
    fn calculate_metrics(&self, 
                        dependencies: &[CargoDependency], 
                        usage_data: &DependencyUsageData) -> Result<DependencyMetrics> {
        metrics::calculate_metrics(dependencies, usage_data)
    }
}

/// Result of the dependency analysis
#[derive(Debug)]
pub struct AnalysisResult {
    pub dependencies: Vec<CargoDependency>,
    pub usage_data: DependencyUsageData,
    pub metrics: DependencyMetrics,
}

/// Data about how dependencies are used in the project
#[derive(Debug, Default)]
pub struct DependencyUsageData {
    /// Maps dependency name to a map of files where it's used
    pub usage_locations: std::collections::HashMap<String, Vec<DependencyUsage>>,
}

/// A specific usage of a dependency in the code
#[derive(Debug, Clone)]
pub struct DependencyUsage {
    pub file: PathBuf,
    pub line: usize,
    pub imported_item: String,
    pub usage_type: UsageType,
}

/// Type of dependency usage
#[derive(Debug, Clone, PartialEq)]
pub enum UsageType {
    Import,
    Function,
    Type,
    Trait,
    Macro,
    Other,
}

/// Metrics calculated for dependencies
#[derive(Debug, Default)]
pub struct DependencyMetrics {
    /// Maps dependency name to its importance score (0.0 to 1.0)
    pub importance_scores: std::collections::HashMap<String, f64>,
    /// Maps dependency name to whether it's used or not
    pub is_used: std::collections::HashMap<String, bool>,
    /// Maps dependency name to how many files it's used in
    pub usage_count: std::collections::HashMap<String, usize>,
} 