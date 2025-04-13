use std::collections::HashMap;
use std::path::Path;
use std::fs;
use anyhow::{Result, Context};
use serde::Deserialize;
use toml::Table;

use super::ManifestParser;

#[derive(Debug, Default)]
pub struct CargoParser;

#[derive(Debug, Clone, Deserialize)]
pub struct CargoDependency {
    pub name: String,
    pub version: Option<String>,
    pub features: Vec<String>,
    pub optional: bool,
    pub dependency_type: DependencyType,
    pub source: String,
}

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub enum DependencyType {
    Normal,
    Development,
    Build,
}

/// Parse a Cargo.toml file and return the dependencies
pub fn parse_cargo_toml<P: AsRef<Path>>(project_path: P) -> Result<Vec<CargoDependency>> {
    let manifest_path = project_path.as_ref().join("Cargo.toml");
    CargoParser::parse(manifest_path)
}

impl ManifestParser for CargoParser {
    type Dependency = CargoDependency;
    
    fn parse<P: AsRef<Path>>(path: P) -> Result<Vec<Self::Dependency>> {
        let manifest_path = path.as_ref();
        let content = fs::read_to_string(manifest_path)
            .with_context(|| format!("Failed to read Cargo.toml at {:?}", manifest_path))?;
        
        let cargo_toml: Table = toml::from_str(&content)
            .with_context(|| format!("Failed to parse Cargo.toml at {:?}", manifest_path))?;
        
        let mut dependencies = Vec::new();
        
        // Process normal dependencies
        if let Some(deps) = cargo_toml.get("dependencies").and_then(|d| d.as_table()) {
            Self::extract_dependencies(deps, &mut dependencies, DependencyType::Normal);
        }
        
        // Process dev-dependencies
        if let Some(deps) = cargo_toml.get("dev-dependencies").and_then(|d| d.as_table()) {
            Self::extract_dependencies(deps, &mut dependencies, DependencyType::Development);
        }
        
        // Process build-dependencies
        if let Some(deps) = cargo_toml.get("build-dependencies").and_then(|d| d.as_table()) {
            Self::extract_dependencies(deps, &mut dependencies, DependencyType::Build);
        }
        
        Ok(dependencies)
    }
}

impl CargoParser {
    fn extract_dependencies(
        deps_table: &Table,
        dependencies: &mut Vec<CargoDependency>,
        dep_type: DependencyType,
    ) {
        for (name, value) in deps_table {
            let mut dep = CargoDependency {
                name: name.clone(),
                version: None,
                features: Vec::new(),
                optional: false,
                dependency_type: dep_type.clone(),
                source: "Cargo.toml".to_string(),
            };
            
            match value {
                toml::Value::String(version) => {
                    dep.version = Some(version.clone());
                }
                toml::Value::Table(table) => {
                    // Handle inline table specification
                    if let Some(version) = table.get("version").and_then(|v| v.as_str()) {
                        dep.version = Some(version.to_string());
                    }
                    
                    if let Some(features) = table.get("features").and_then(|f| f.as_array()) {
                        dep.features = features.iter()
                            .filter_map(|f| f.as_str().map(|s| s.to_string()))
                            .collect();
                    }
                    
                    if let Some(optional) = table.get("optional").and_then(|o| o.as_bool()) {
                        dep.optional = optional;
                    }
                }
                _ => {
                    // Skip other value types
                    continue;
                }
            }
            
            dependencies.push(dep);
        }
    }
} 