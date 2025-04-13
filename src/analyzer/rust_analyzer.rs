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
        
        // Parse the file to an AST
        let syntax = syn::parse_file(&file_content)
            .with_context(|| format!("Failed to parse Rust file: {:?}", file_path))?;
        
        // Create a visitor to find usages
        let mut visitor = DependencyVisitor::new(file_path, dependencies);
        visitor.visit_file(&syntax);
        
        // Update usage data with findings
        for (dep_name, usages) in visitor.usages {
            if let Some(dep_usages) = usage_data.usage_locations.get_mut(&dep_name) {
                dep_usages.extend(usages);
            }
        }
        
        Ok(())
    }
}

/// Visitor for traversing Rust syntax trees and finding dependency usages
struct DependencyVisitor<'a> {
    file_path: &'a Path,
    dependencies: &'a [CargoDependency],
    usages: HashMap<String, Vec<DependencyUsage>>,
}

impl<'a> DependencyVisitor<'a> {
    fn new(file_path: &'a Path, dependencies: &'a [CargoDependency]) -> Self {
        Self {
            file_path,
            dependencies,
            usages: HashMap::new(),
        }
    }
    
    fn add_usage(&mut self, dep_name: String, usage: DependencyUsage) {
        self.usages.entry(dep_name).or_insert_with(Vec::new).push(usage);
    }
    
    fn find_dependency(&self, path: &syn::Path) -> Option<&CargoDependency> {
        if let Some(first_segment) = path.segments.first() {
            let name = first_segment.ident.to_string();
            self.dependencies.iter().find(|dep| dep.name == name)
        } else {
            None
        }
    }
}

impl<'a, 'ast> Visit<'ast> for DependencyVisitor<'a> {
    fn visit_use_path(&mut self, use_path: &'ast syn::UsePath) {
        if let Some(dep) = self.find_dependency(&use_path.path) {
            let line = use_path.path.span().start().line;
            let imported_item = use_path.path.segments.iter()
                .map(|seg| seg.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");
            
            self.add_usage(
                dep.name.clone(),
                DependencyUsage {
                    file: self.file_path.to_path_buf(),
                    line,
                    imported_item,
                    usage_type: UsageType::Import,
                },
            );
        }
        
        // Continue visiting the rest of the tree
        visit::visit_use_path(self, use_path);
    }
    
    fn visit_item_extern_crate(&mut self, extern_crate: &'ast syn::ItemExternCrate) {
        let crate_name = extern_crate.ident.to_string();
        if let Some(dep) = self.dependencies.iter().find(|dep| dep.name == crate_name) {
            let line = extern_crate.span().start().line;
            
            self.add_usage(
                dep.name.clone(),
                DependencyUsage {
                    file: self.file_path.to_path_buf(),
                    line,
                    imported_item: crate_name,
                    usage_type: UsageType::Import,
                },
            );
        }
        
        visit::visit_item_extern_crate(self, extern_crate);
    }
    
    fn visit_macro(&mut self, mac: &'ast syn::Macro) {
        if let Some(path_segment) = mac.path.segments.first() {
            let macro_name = path_segment.ident.to_string();
            
            // Check if this macro comes from one of our dependencies
            // This is a simplification - in practice, tracking macro usage to dependencies is complex
            for dep in self.dependencies {
                // Simple heuristic: if the macro name starts with the dependency name, it's likely from that dependency
                if macro_name.starts_with(&dep.name) || macro_name.contains(&dep.name) {
                    let line = mac.path.span().start().line;
                    
                    self.add_usage(
                        dep.name.clone(),
                        DependencyUsage {
                            file: self.file_path.to_path_buf(),
                            line,
                            imported_item: macro_name,
                            usage_type: UsageType::Macro,
                        },
                    );
                    break;
                }
            }
        }
        
        visit::visit_macro(self, mac);
    }
    
    // Additional visit methods can be implemented to track other types of usage
} 