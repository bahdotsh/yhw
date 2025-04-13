use std::collections::{HashMap, HashSet};
use anyhow::Result;

use crate::manifest::cargo::CargoDependency;
use crate::analyzer::{DependencyUsageData, DependencyMetrics, UsageType};

/// Calculate metrics for dependencies based on usage data
pub fn calculate_metrics(
    dependencies: &[CargoDependency],
    usage_data: &DependencyUsageData,
) -> Result<DependencyMetrics> {
    let mut metrics = DependencyMetrics::default();
    
    // Calculate usage metrics
    for dep in dependencies {
        let empty_vec = Vec::new();
        let usages = usage_data.usage_locations.get(&dep.name).unwrap_or(&empty_vec);
        let is_used = !usages.is_empty();
        
        // Count unique files where the dependency is used
        let unique_files: HashSet<_> = usages.iter().map(|usage| &usage.file).collect();
        let usage_count = unique_files.len();
        
        // Count usage types
        let usage_types = count_usage_types(usages);
        
        // Calculate feature usage
        let feature_usage = calculate_feature_usage(dep, usages);
        
        // Calculate importance score (enhanced version)
        let importance_score = calculate_importance_score(dep, usages, usage_count, &usage_types);
        
        // Determine if dependency is partially used
        let is_partially_used = determine_if_partially_used(dep, &feature_usage);
        
        // Store metrics
        metrics.is_used.insert(dep.name.clone(), is_used);
        metrics.usage_count.insert(dep.name.clone(), usage_count);
        metrics.importance_scores.insert(dep.name.clone(), importance_score);
        metrics.usage_types.insert(dep.name.clone(), usage_types);
        metrics.feature_usage.insert(dep.name.clone(), feature_usage);
        metrics.is_partially_used.insert(dep.name.clone(), is_partially_used);
    }
    
    metrics.removable_dependencies = find_removable_dependencies(&metrics);
    
    Ok(metrics)
}

/// Count the different types of usage for a dependency
fn count_usage_types(usages: &[crate::analyzer::DependencyUsage]) -> HashMap<UsageType, usize> {
    let mut counts = HashMap::new();
    
    for usage in usages {
        *counts.entry(usage.usage_type.clone()).or_insert(0) += 1;
    }
    
    counts
}

/// Calculate which features of a dependency are used
fn calculate_feature_usage(
    dep: &CargoDependency, 
    usages: &[crate::analyzer::DependencyUsage]
) -> HashMap<String, bool> {
    let mut feature_usage = HashMap::new();
    
    // Initialize all features as unused
    for feature in &dep.features {
        feature_usage.insert(feature.clone(), false);
    }
    
    // If we have no features or the dependency isn't used, return early
    if dep.features.is_empty() || usages.is_empty() {
        return feature_usage;
    }
    
    // This is a simplified heuristic and would need to be expanded for real feature detection
    // A more accurate approach would require deeper analysis of the crate's API and what features enable what items
    
    // For now, we'll mark features as used based on some simple heuristics
    for usage in usages {
        let imported_item = &usage.imported_item;
        
        for feature in &dep.features {
            // Simple heuristic: if the imported item contains the feature name, mark it as used
            // This is not accurate but serves as a placeholder for more sophisticated detection
            if imported_item.contains(feature) {
                feature_usage.insert(feature.clone(), true);
            }
            
            // This could be expanded with crate-specific knowledge about what each feature enables
        }
    }
    
    feature_usage
}

/// Determine if a dependency is partially used (not all features are used)
fn determine_if_partially_used(
    dep: &CargoDependency,
    feature_usage: &HashMap<String, bool>
) -> bool {
    // If there are no features, it's not partially used
    if dep.features.is_empty() {
        return false;
    }
    
    // If any feature is not used, it's partially used
    feature_usage.values().any(|&used| !used)
}

/// Calculate an enhanced importance score for a dependency
fn calculate_importance_score(
    dep: &CargoDependency,
    usages: &[crate::analyzer::DependencyUsage],
    usage_count: usize,
    usage_types: &HashMap<UsageType, usize>,
) -> f64 {
    // Factors that influence importance:
    // 1. Number of files using the dependency (coverage)
    // 2. Variety of usage types (function calls, types, macros)
    // 3. Whether it's a dev or normal dependency
    // 4. Whether it's optional
    // 5. Frequency and diversity of usage
    
    if usages.is_empty() {
        return 0.0;
    }
    
    // Calculate base score from usage count
    // More files using the dependency = higher importance
    let base_score = (usage_count as f64).min(20.0) / 20.0;
    
    // Variety of usage types increases importance
    let variety_factor = (usage_types.len() as f64) / 5.0;
    
    // Depth of usage - check how extensively the dependency is used
    let usage_depth = calculate_usage_depth(usage_types);
    
    // Dependency type factors
    let type_factor = match dep.dependency_type {
        crate::manifest::cargo::DependencyType::Normal => 1.0,
        crate::manifest::cargo::DependencyType::Development => 0.5,
        crate::manifest::cargo::DependencyType::Build => 0.7,
    };
    
    // Optional dependencies are less important
    let optional_factor = if dep.optional { 0.7 } else { 1.0 };
    
    // Calculate final score (capped at 1.0)
    let score = base_score * (1.0 + variety_factor) * (1.0 + usage_depth) * type_factor * optional_factor;
    score.min(1.0)
}

/// Calculate the depth of usage based on usage types
fn calculate_usage_depth(usage_types: &HashMap<UsageType, usize>) -> f64 {
    let mut depth = 0.0;
    
    // Imports are basic usage
    let import_count = usage_types.get(&UsageType::Import).unwrap_or(&0);
    if *import_count > 0 {
        depth += 0.1;
    }
    
    // Function calls indicate more significant usage
    let function_count = usage_types.get(&UsageType::Function).unwrap_or(&0);
    if *function_count > 0 {
        depth += 0.3 * (*function_count as f64).min(10.0) / 10.0;
    }
    
    // Type usage indicates structural dependency
    let type_count = usage_types.get(&UsageType::Type).unwrap_or(&0);
    if *type_count > 0 {
        depth += 0.3 * (*type_count as f64).min(10.0) / 10.0;
    }
    
    // Trait usage indicates deep integration
    let trait_count = usage_types.get(&UsageType::Trait).unwrap_or(&0);
    if *trait_count > 0 {
        depth += 0.3 * (*trait_count as f64).min(5.0) / 5.0;
    }
    
    // Macro usage can indicate either shallow or deep usage depending on the macro
    let macro_count = usage_types.get(&UsageType::Macro).unwrap_or(&0);
    if *macro_count > 0 {
        depth += 0.2 * (*macro_count as f64).min(10.0) / 10.0;
    }
    
    depth.min(1.0)
}

/// Find dependencies that can potentially be removed
pub fn find_removable_dependencies(metrics: &DependencyMetrics) -> Vec<String> {
    let mut removable = Vec::new();
    
    for (dep_name, is_used) in &metrics.is_used {
        if !is_used {
            // Unused dependencies are definitely removable
            removable.push(dep_name.clone());
        } else {
            // For used dependencies, check if they're minimally used
            let score = metrics.importance_scores.get(dep_name).unwrap_or(&1.0);
            
            if *score < 0.1 {
                // Very low importance score suggests it might be removable
                removable.push(dep_name.clone());
            } else if let Some(true) = metrics.is_partially_used.get(dep_name) {
                // If it's partially used and the importance score is still low,
                // suggest it as potentially removable, but with lower confidence
                if *score < 0.3 {
                    removable.push(dep_name.clone());
                }
            }
        }
    }
    
    removable
} 