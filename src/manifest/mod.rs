pub mod cargo;

use std::path::Path;
use anyhow::Result;

/// A trait for parsing project manifests
pub trait ManifestParser {
    /// The dependency type for the manifest
    type Dependency;
    
    /// Parse a manifest file at the given path
    fn parse<P: AsRef<Path>>(path: P) -> Result<Vec<Self::Dependency>>;
}

/// Get the appropriate parser for a manifest file
pub fn get_parser<P: AsRef<Path>>(path: P) -> Result<Box<dyn ManifestParser>> {
    // For now, only Cargo.toml is supported
    let path = path.as_ref();
    
    if path.ends_with("Cargo.toml") {
        Ok(Box::new(cargo::CargoParser::default()))
    } else {
        Err(anyhow::anyhow!("Unsupported manifest file: {:?}", path))
    }
} 