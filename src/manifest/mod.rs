pub mod cargo;

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
}

/// Get the appropriate parser type for a manifest file
pub fn get_parser_type<P: AsRef<Path>>(path: P) -> Result<ManifestParserType> {
    let path = path.as_ref();
    
    if path.ends_with("Cargo.toml") {
        Ok(ManifestParserType::Cargo)
    } else {
        Err(anyhow::anyhow!("Unsupported manifest file: {:?}", path))
    }
}

/// Parse dependencies from a manifest file
pub fn parse_dependencies<P: AsRef<Path>>(path: P) -> Result<Vec<CargoDependency>> {
    let parser_type = get_parser_type(&path)?;
    
    match parser_type {
        ManifestParserType::Cargo => cargo::CargoParser::parse(path),
    }
} 