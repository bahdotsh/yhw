use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use walkdir::WalkDir;
use syn::{self, visit::{Visit, self}};

use crate::manifest::cargo::CargoDependency;
use crate::analyzer::{DependencyUsageData, DependencyUsage, UsageType};

/// Analyzer for Rust code files
pub struct RustAnalyzer {
    project_path: PathBuf,
}

impl RustAnalyzer {
    /// Create a new Rust analyzer for the given project path
    pub fn new<P: AsRef<Path>>(project_path: P) -> Self {
        Self {
            project_path: project_path.as_ref().to_path_buf(),
        }
    }
    
    /// Analyze Rust code to detect dependency usage
    pub fn analyze(&self, dependencies: &[CargoDependency]) -> Result<DependencyUsageData> {
        let mut usage_data = DependencyUsageData::default();
        
        // Initialize usage locations for all dependencies
        for dep in dependencies {
            usage_data.usage_locations.insert(dep.name.clone(), Vec::new());
        }
        
        // Find all Rust files in the project
        for entry in WalkDir::new(&self.project_path)
            .into_iter()
            .filter_entry(|e| !Self::is_excluded(e.path()))
            .filter_map(Result::ok)
            .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
        {
            let file_path = entry.path();
            self.analyze_file(file_path, dependencies, &mut usage_data)?;
        }
        
        Ok(usage_data)
    }
    
    /// Determine if a path should be excluded from analysis
    fn is_excluded(path: &Path) -> bool {
        let path_str = path.to_string_lossy();
        path_str.contains("/target/") || path_str.contains("/.git/")
    }
    
    /// Analyze a single Rust file for dependency usage
    fn analyze_file(
        &self,
        file_path: &Path,
        dependencies: &[CargoDependency],
        usage_data: &mut DependencyUsageData,
    ) -> Result<()> {
        let file_content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read file: {:?}", file_path))?;
        
        // Simple approach: look for use statements and extern crate statements in the file
        // For a more complete solution, we would parse the file to an AST and use a visitor
        // But for Phase 1, this simplified approach should be sufficient
        
        // Track line numbers
        for (line_number, line) in file_content.lines().enumerate() {
            // Check for use statements
            if line.trim().starts_with("use ") {
                self.process_use_statement(line, line_number + 1, file_path, dependencies, usage_data);
            }
            
            // Check for extern crate statements
            if line.trim().starts_with("extern crate ") {
                self.process_extern_crate(line, line_number + 1, file_path, dependencies, usage_data);
            }
        }
        
        Ok(())
    }
    
    /// Process a use statement to find dependency usages
    fn process_use_statement(
        &self,
        line: &str, 
        line_number: usize, 
        file_path: &Path,
        dependencies: &[CargoDependency],
        usage_data: &mut DependencyUsageData,
    ) {
        // Extract the first part of the use statement
        let line = line.trim().trim_start_matches("use ");
        let first_part = line.split("::").next().unwrap_or("");
        
        // Check if this matches a dependency
        for dep in dependencies {
            if first_part == dep.name {
                if let Some(usages) = usage_data.usage_locations.get_mut(&dep.name) {
                    usages.push(DependencyUsage {
                        file: file_path.to_path_buf(),
                        line: line_number,
                        imported_item: line.trim_end_matches(';').to_owned(),
                        usage_type: UsageType::Import,
                    });
                }
            }
        }
    }
    
    /// Process an extern crate statement to find dependency usages
    fn process_extern_crate(
        &self,
        line: &str, 
        line_number: usize, 
        file_path: &Path,
        dependencies: &[CargoDependency],
        usage_data: &mut DependencyUsageData,
    ) {
        // Extract the crate name
        let line = line.trim().trim_start_matches("extern crate ");
        let crate_name = line.split_whitespace().next().unwrap_or("").trim_end_matches(';');
        
        // Check if this matches a dependency
        for dep in dependencies {
            if crate_name == dep.name {
                if let Some(usages) = usage_data.usage_locations.get_mut(&dep.name) {
                    usages.push(DependencyUsage {
                        file: file_path.to_path_buf(),
                        line: line_number,
                        imported_item: crate_name.to_owned(),
                        usage_type: UsageType::Import,
                    });
                }
            }
        }
    }
} 