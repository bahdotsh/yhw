pub mod cargo;
pub mod nodejs;

use std::path::Path;
use anyhow::Result;
use crate::manifest::cargo::CargoDependency;

/// A trait for parsing project manifests
pub trait ManifestParser {
    /// The dependency type for the manifest
    type Dependency;
    
    /// Parse a manifest file at the given path
    fn parse<P: AsRef<Path>>(path: P) -> Result<Vec<Self::Dependency>>;
}

/// Enum of supported manifest parsers
pub enum ManifestParserType {
    Cargo,
    NodeJs,
}

/// Get the appropriate parser type for a manifest file
pub fn get_parser_type<P: AsRef<Path>>(path: P) -> Result<ManifestParserType> {
    let path = path.as_ref();
    
    if path.ends_with("Cargo.toml") {
        Ok(ManifestParserType::Cargo)
    } else if path.ends_with("package.json") {
        Ok(ManifestParserType::NodeJs)
    } else {
        Err(anyhow::anyhow!("Unsupported manifest file: {:?}", path))
    }
}

/// Parse dependencies from a manifest file
pub fn parse_dependencies<P: AsRef<Path>>(path: P) -> Result<Vec<CargoDependency>> {
    let parser_type = get_parser_type(&path)?;
    
    match parser_type {
        ManifestParserType::Cargo => cargo::CargoParser::parse(path),
        ManifestParserType::NodeJs => {
            // For now, we'll convert NodeJs dependencies to CargoDependency format
            // Later we can use a more generalized Dependency trait/enum
            let node_deps = nodejs::NodeJsParser::parse(path)?;
            
            // Convert NodeJs dependencies to Cargo format
            let cargo_deps = node_deps.into_iter()
                .map(|node_dep| CargoDependency {
                    name: node_dep.name,
                    version: Some(node_dep.version),
                    features: Vec::new(), // Node.js doesn't have features like Cargo
                    optional: false,
                    dependency_type: match node_dep.dependency_type {
                        nodejs::DependencyType::Normal => cargo::DependencyType::Normal,
                        nodejs::DependencyType::Development => cargo::DependencyType::Development,
                    },
                    source: "package.json".to_string(),
                })
                .collect();
            
            Ok(cargo_deps)
        }
    }
} 