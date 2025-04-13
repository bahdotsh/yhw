use std::path::Path;
use std::fs;
use anyhow::{Result, Context};
use serde::Deserialize;
use serde_json::Value;

use super::ManifestParser;

#[derive(Debug, Default)]
pub struct NodeJsParser;

#[derive(Debug, Clone)]
pub struct NodeJsDependency {
    pub name: String,
    pub version: String,
    pub dependency_type: DependencyType,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DependencyType {
    Normal,
    Development,
}

impl ManifestParser for NodeJsParser {
    type Dependency = NodeJsDependency;
    
    fn parse<P: AsRef<Path>>(path: P) -> Result<Vec<Self::Dependency>> {
        let manifest_path = path.as_ref();
        let content = fs::read_to_string(manifest_path)
            .with_context(|| format!("Failed to read package.json at {:?}", manifest_path))?;
        
        let package_json: Value = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse package.json at {:?}", manifest_path))?;
        
        let mut dependencies = Vec::new();
        
        // Process normal dependencies
        if let Some(deps) = package_json.get("dependencies").and_then(|d| d.as_object()) {
            for (name, version) in deps {
                if let Some(version_str) = version.as_str() {
                    dependencies.push(NodeJsDependency {
                        name: name.clone(),
                        version: version_str.to_string(),
                        dependency_type: DependencyType::Normal,
                    });
                }
            }
        }
        
        // Process dev dependencies
        if let Some(deps) = package_json.get("devDependencies").and_then(|d| d.as_object()) {
            for (name, version) in deps {
                if let Some(version_str) = version.as_str() {
                    dependencies.push(NodeJsDependency {
                        name: name.clone(),
                        version: version_str.to_string(),
                        dependency_type: DependencyType::Development,
                    });
                }
            }
        }
        
        Ok(dependencies)
    }
}

/// Parse a package.json file and return the dependencies
pub fn parse_package_json<P: AsRef<Path>>(project_path: P) -> Result<Vec<NodeJsDependency>> {
    let manifest_path = project_path.as_ref().join("package.json");
    NodeJsParser::parse(manifest_path)
} 