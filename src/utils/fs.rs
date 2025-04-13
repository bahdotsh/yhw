use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::Result;

/// Find a file in a directory tree
pub fn find_file<P: AsRef<Path>>(dir: P, filename: &str) -> Option<PathBuf> {
    WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .find(|entry| entry.file_name().to_string_lossy() == filename)
        .map(|entry| entry.path().to_path_buf())
}

/// Find all Rust source files in a directory
pub fn find_rust_files<P: AsRef<Path>>(dir: P) -> Result<Vec<PathBuf>> {
    let mut rust_files = Vec::new();
    
    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|e| !is_excluded(e.path()))
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "rs"))
    {
        rust_files.push(entry.path().to_path_buf());
    }
    
    Ok(rust_files)
}

/// Check if a path should be excluded from analysis
pub fn is_excluded(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/target/") || path_str.contains("/.git/")
}

/// Find the project's manifest file
pub fn find_manifest_file<P: AsRef<Path>>(dir: P) -> Option<PathBuf> {
    let cargo_toml = dir.as_ref().join("Cargo.toml");
    if cargo_toml.exists() {
        return Some(cargo_toml);
    }
    
    None
} 