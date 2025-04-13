pub mod rust_analyzer;
pub mod metrics;
pub mod dependency_graph;

use std::path::{Path, PathBuf};
use anyhow::Result;
use crate::manifest::cargo::CargoDependency;
use serde::Serialize;

/// Main analyzer that orchestrates the analysis process
pub struct DependencyAnalyzer {
    project_path: PathBuf,
}

// Structure to represent an analyzed dependency with all relevant metrics
#[derive(Debug, Clone, Serialize)]
pub struct AnalyzedDependency {
    pub name: String,
    pub version: String,
    pub usage_count: usize,
    pub importance_score: f64,
    pub removable: bool,
    pub used_features: Vec<String>,
    pub unused_features: Vec<String>,
}

/// Analysis result that will be returned to the main function and can be exported
#[derive(Debug, Serialize)]
pub struct Analysis {
    pub dependencies: Vec<AnalyzedDependency>,
}

impl Analysis {
    pub fn filter_dependency(&mut self, dep_name: &str) {
        self.dependencies.retain(|dep| dep.name == dep_name);
    }
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
        
        // Generate dependency graph
        let dependency_graph = self.generate_dependency_graph(&dependencies)?;
        
        Ok(AnalysisResult {
            dependencies,
            usage_data,
            metrics,
            dependency_graph,
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
        use crate::manifest;
        
        manifest::parse_dependencies(manifest_path)
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
    
    fn generate_dependency_graph(&self, dependencies: &[CargoDependency]) -> Result<dependency_graph::DependencyGraph> {
        // Check for Cargo.lock file
        let cargo_lock_path = self.project_path.join("Cargo.lock");
        if cargo_lock_path.exists() {
            // Use Cargo.lock to build a more accurate dependency graph
            dependency_graph::DependencyGraph::from_cargo_lock(&cargo_lock_path, dependencies)
        } else {
            // Create a simple graph without relationship information
            Ok(dependency_graph::DependencyGraph::new(dependencies))
        }
    }
}

/// Result of the dependency analysis
#[derive(Debug)]
pub struct AnalysisResult {
    pub dependencies: Vec<CargoDependency>,
    pub usage_data: DependencyUsageData,
    pub metrics: DependencyMetrics,
    pub dependency_graph: dependency_graph::DependencyGraph,
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
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
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
    /// Maps dependency name to counts of different usage types
    pub usage_types: std::collections::HashMap<String, std::collections::HashMap<UsageType, usize>>,
    /// Maps dependency name to feature usage (feature name -> is used)
    pub feature_usage: std::collections::HashMap<String, std::collections::HashMap<String, bool>>,
    /// Maps dependency name to whether it's partially used
    pub is_partially_used: std::collections::HashMap<String, bool>,
    /// List of dependencies that could potentially be removed
    pub removable_dependencies: Vec<String>,
}

/// Analyze a project and return a simplified representation for export
pub fn analyze<P: AsRef<Path>>(project_path: P, manifest: &[CargoDependency]) -> Result<Analysis> {
    let analyzer = DependencyAnalyzer::new(project_path);
    let analysis_result = analyzer.analyze()?;
    
    // Create analyzed dependencies by combining data from the analysis result
    let dependencies = manifest.iter()
        .map(|dep| {
            let name = &dep.name;
            let version = dep.version.clone().unwrap_or_else(|| "".to_string());
            let usage_count = *analysis_result.metrics.usage_count.get(name).unwrap_or(&0);
            let importance_score = *analysis_result.metrics.importance_scores.get(name).unwrap_or(&0.0);
            let removable = analysis_result.metrics.removable_dependencies.contains(name);
            
            // Extract used and unused features
            let mut used_features = Vec::new();
            let mut unused_features = Vec::new();
            
            if let Some(feature_map) = analysis_result.metrics.feature_usage.get(name) {
                for (feature, is_used) in feature_map {
                    if *is_used {
                        used_features.push(feature.clone());
                    } else {
                        unused_features.push(feature.clone());
                    }
                }
            }
            
            AnalyzedDependency {
                name: name.clone(),
                version,
                usage_count,
                importance_score,
                removable,
                used_features,
                unused_features,
            }
        })
        .collect();
    
    Ok(Analysis { dependencies })
} 