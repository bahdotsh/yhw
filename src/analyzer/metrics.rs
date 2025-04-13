use std::collections::{HashMap, HashSet};
use anyhow::Result;

use crate::manifest::cargo::CargoDependency;
use crate::analyzer::{DependencyUsageData, DependencyMetrics};

/// Calculate metrics for dependencies based on usage data
pub fn calculate_metrics(
    dependencies: &[CargoDependency],
    usage_data: &DependencyUsageData,
) -> Result<DependencyMetrics> {
    let mut metrics = DependencyMetrics::default();
    
    // Calculate usage metrics
    for dep in dependencies {
        let usages = usage_data.usage_locations.get(&dep.name).unwrap_or(&Vec::new());
        let is_used = !usages.is_empty();
        
        // Count unique files where the dependency is used
        let unique_files: HashSet<_> = usages.iter().map(|usage| &usage.file).collect();
        let usage_count = unique_files.len();
        
        // Calculate importance score
        let importance_score = calculate_importance_score(dep, usages, usage_count);
        
        // Store metrics
        metrics.is_used.insert(dep.name.clone(), is_used);
        metrics.usage_count.insert(dep.name.clone(), usage_count);
        metrics.importance_scores.insert(dep.name.clone(), importance_score);
    }
    
    Ok(metrics)
}

/// Calculate an importance score for a dependency
fn calculate_importance_score(
    dep: &CargoDependency,
    usages: &[crate::analyzer::DependencyUsage],
    usage_count: usize,
) -> f64 {
    // Factors that influence importance:
    // 1. Number of files using the dependency (coverage)
    // 2. Variety of usage types (function calls, types, macros)
    // 3. Whether it's a dev or normal dependency
    // 4. Whether it's optional
    
    if usages.is_empty() {
        return 0.0;
    }
    
    // Calculate base score from usage count
    // More files using the dependency = higher importance
    let base_score = (usage_count as f64).min(20.0) / 20.0;
    
    // Variety of usage types increases importance
    let unique_usage_types: HashSet<_> = usages.iter().map(|usage| &usage.usage_type).collect();
    let variety_factor = (unique_usage_types.len() as f64) / 5.0;
    
    // Dependency type factors
    let type_factor = match dep.dependency_type {
        crate::manifest::cargo::DependencyType::Normal => 1.0,
        crate::manifest::cargo::DependencyType::Development => 0.5,
        crate::manifest::cargo::DependencyType::Build => 0.7,
    };
    
    // Optional dependencies are less important
    let optional_factor = if dep.optional { 0.7 } else { 1.0 };
    
    // Calculate final score (capped at 1.0)
    let score = base_score * (1.0 + variety_factor) * type_factor * optional_factor;
    score.min(1.0)
}

/// Find dependencies that can potentially be removed
pub fn find_removable_dependencies(metrics: &DependencyMetrics) -> Vec<String> {
    metrics.is_used.iter()
        .filter_map(|(dep_name, is_used)| {
            if !is_used {
                Some(dep_name.clone())
            } else {
                // Rarely used dependencies (low importance score)
                let score = metrics.importance_scores.get(dep_name).unwrap_or(&1.0);
                if *score < 0.1 {
                    Some(dep_name.clone())
                } else {
                    None
                }
            }
        })
        .collect()
} 