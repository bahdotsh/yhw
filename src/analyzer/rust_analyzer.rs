use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use anyhow::{Result, Context};
use walkdir::WalkDir;
use syn::{self, visit::{Visit, self}, parse_file, ItemUse, UseTree, UsePath, UseGroup, UseName};
use syn::spanned::Spanned;

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
        
        // Advanced approach: parse the file to an AST and use a visitor to analyze dependency usage
        match parse_file(&file_content) {
            Ok(file) => {
                let mut visitor = RustDependencyVisitor {
                    file_path: file_path.to_path_buf(),
                    dependencies,
                    usage_data,
                    current_imports: HashMap::new(),
                };
                visitor.visit_file(&file);
            }
            Err(err) => {
                // Fall back to simple text-based parsing if AST parsing fails
                eprintln!("Warning: Failed to parse file {:?}: {}", file_path, err);
                self.analyze_file_simple(&file_content, file_path, dependencies, usage_data);
            }
        }
        
        Ok(())
    }
    
    /// Simple text-based analysis fallback
    fn analyze_file_simple(
        &self,
        file_content: &str,
        file_path: &Path,
        dependencies: &[CargoDependency],
        usage_data: &mut DependencyUsageData,
    ) {
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

/// AST visitor to extract dependency usage information
struct RustDependencyVisitor<'a> {
    file_path: PathBuf,
    dependencies: &'a [CargoDependency],
    usage_data: &'a mut DependencyUsageData,
    current_imports: HashMap<String, String>, // Maps local name to fully qualified name
}

impl<'a, 'ast> Visit<'ast> for RustDependencyVisitor<'a> {
    fn visit_item_use(&mut self, node: &'ast ItemUse) {
        // Process imports and update current_imports map
        // For simplicity, we're using line 0 as we can't easily get the line number
        // In a real implementation, you would extract the line number properly
        let line = 0;
        self.process_use_tree(&node.tree, "", line);
        
        // Continue visiting
        visit::visit_item_use(self, node);
    }
    
    fn visit_macro(&mut self, node: &'ast syn::Macro) {
        // Detect macro usage
        if let Some(segment) = node.path.segments.first() {
            let macro_name = segment.ident.to_string();
            
            // Check if this macro is from a tracked dependency
            for (local_name, full_path) in &self.current_imports {
                if macro_name == *local_name {
                    // Extract the crate name from the full path
                    let crate_name = full_path.split("::").next().unwrap_or("");
                    
                    for dep in self.dependencies {
                        if crate_name == dep.name {
                            if let Some(usages) = self.usage_data.usage_locations.get_mut(&dep.name) {
                                // Get span info, defaulting to line 0 if unavailable
                                let line = 0; // In a real implementation, extract the correct line number
                                
                                usages.push(DependencyUsage {
                                    file: self.file_path.clone(),
                                    line,
                                    imported_item: format!("{}!", macro_name),
                                    usage_type: UsageType::Macro,
                                });
                            }
                        }
                    }
                }
            }
        }
        
        // Continue visiting
        visit::visit_macro(self, node);
    }
    
    fn visit_path(&mut self, node: &'ast syn::Path) {
        // Check if this path refers to a tracked dependency
        if let Some(segment) = node.segments.first() {
            let name = segment.ident.to_string();
            
            // Direct usage of dependencies (e.g., some_crate::func())
            for dep in self.dependencies {
                if name == dep.name {
                    if let Some(usages) = self.usage_data.usage_locations.get_mut(&dep.name) {
                        let path_str = path_to_string(node);
                        let usage_type = determine_usage_type(node);
                        
                        // Use 0 as placeholder for line number
                        let line = 0; // In a real implementation, extract the correct line number
                        
                        usages.push(DependencyUsage {
                            file: self.file_path.clone(),
                            line,
                            imported_item: path_str,
                            usage_type,
                        });
                    }
                }
            }
            
            // Usage through imports (e.g., use some_crate::Thing; ... Thing::new())
            if let Some(full_path) = self.current_imports.get(&name) {
                let crate_name = full_path.split("::").next().unwrap_or("");
                
                for dep in self.dependencies {
                    if crate_name == dep.name {
                        if let Some(usages) = self.usage_data.usage_locations.get_mut(&dep.name) {
                            let path_str = path_to_string(node);
                            let usage_type = determine_usage_type(node);
                            
                            // Use 0 as placeholder for line number
                            let line = 0; // In a real implementation, extract the correct line number
                            
                            usages.push(DependencyUsage {
                                file: self.file_path.clone(),
                                line,
                                imported_item: format!("{} (from {})", path_str, full_path),
                                usage_type,
                            });
                        }
                    }
                }
            }
        }
        
        // Continue visiting
        visit::visit_path(self, node);
    }
}

impl<'a> RustDependencyVisitor<'a> {
    /// Process a use tree to extract import information
    fn process_use_tree(&mut self, tree: &UseTree, prefix: &str, line: usize) {
        match tree {
            UseTree::Path(UsePath { ident, tree, .. }) => {
                let new_prefix = if prefix.is_empty() {
                    ident.to_string()
                } else {
                    format!("{}::{}", prefix, ident)
                };
                
                // Check if this is a dependency
                for dep in self.dependencies {
                    if ident.to_string() == dep.name && prefix.is_empty() {
                        if let Some(usages) = self.usage_data.usage_locations.get_mut(&dep.name) {
                            usages.push(DependencyUsage {
                                file: self.file_path.clone(),
                                line,
                                imported_item: format!("{}::<rest>", ident),
                                usage_type: UsageType::Import,
                            });
                        }
                    }
                }
                
                self.process_use_tree(tree, &new_prefix, line);
            },
            UseTree::Name(UseName { ident, .. }) => {
                let full_path = if prefix.is_empty() {
                    ident.to_string()
                } else {
                    format!("{}::{}", prefix, ident)
                };
                
                // Add to imports map
                self.current_imports.insert(ident.to_string(), full_path.clone());
                
                // Check if the prefix is a dependency
                let crate_name = prefix.split("::").next().unwrap_or("");
                for dep in self.dependencies {
                    if crate_name == dep.name {
                        if let Some(usages) = self.usage_data.usage_locations.get_mut(&dep.name) {
                            usages.push(DependencyUsage {
                                file: self.file_path.clone(),
                                line,
                                imported_item: full_path.clone(),
                                usage_type: UsageType::Import,
                            });
                        }
                    }
                }
            },
            UseTree::Rename(rename) => {
                let full_path = if prefix.is_empty() {
                    rename.ident.to_string()
                } else {
                    format!("{}::{}", prefix, rename.ident)
                };
                
                // Add to imports map with the renamed identifier
                self.current_imports.insert(rename.rename.to_string(), full_path.clone());
                
                // Check if the prefix is a dependency
                let crate_name = prefix.split("::").next().unwrap_or("");
                for dep in self.dependencies {
                    if crate_name == dep.name {
                        if let Some(usages) = self.usage_data.usage_locations.get_mut(&dep.name) {
                            usages.push(DependencyUsage {
                                file: self.file_path.clone(),
                                line,
                                imported_item: format!("{} as {}", full_path, rename.rename),
                                usage_type: UsageType::Import,
                            });
                        }
                    }
                }
            },
            UseTree::Glob(_) => {
                // For glob imports (e.g., use some_crate::*;)
                let crate_name = prefix.split("::").next().unwrap_or("");
                for dep in self.dependencies {
                    if crate_name == dep.name {
                        if let Some(usages) = self.usage_data.usage_locations.get_mut(&dep.name) {
                            usages.push(DependencyUsage {
                                file: self.file_path.clone(),
                                line,
                                imported_item: format!("{}::*", prefix),
                                usage_type: UsageType::Import,
                            });
                        }
                    }
                }
            },
            UseTree::Group(UseGroup { items, .. }) => {
                // For grouped imports (e.g., use some_crate::{Thing1, Thing2};)
                for item in items {
                    self.process_use_tree(item, prefix, line);
                }
            },
        }
    }
}

/// Convert a path to a string
fn path_to_string(path: &syn::Path) -> String {
    path.segments.iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}

/// Determine the type of usage based on the context of the path
fn determine_usage_type(path: &syn::Path) -> UsageType {
    // This is a simplified heuristic and could be improved
    // For more accuracy, we would need to examine the parent node in the AST
    let last_segment = path.segments.last();
    
    if let Some(segment) = last_segment {
        let name = segment.ident.to_string();
        
        // Check if path is a function call
        if segment.arguments.is_empty() && is_lowercase_first(&name) {
            return UsageType::Function;
        }
        
        // Check if path is likely a type
        if is_uppercase_first(&name) {
            return UsageType::Type;
        }
        
        // Check if path is likely a trait
        if is_uppercase_first(&name) && path.segments.len() > 1 {
            return UsageType::Trait;
        }
    }
    
    UsageType::Other
}

/// Check if a string starts with a lowercase letter
fn is_lowercase_first(s: &str) -> bool {
    s.chars().next().map_or(false, |c| c.is_lowercase())
}

/// Check if a string starts with an uppercase letter
fn is_uppercase_first(s: &str) -> bool {
    s.chars().next().map_or(false, |c| c.is_uppercase())
} 