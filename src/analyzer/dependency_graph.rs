use std::collections::HashMap;
use std::path::Path;
use anyhow::Result;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::algo;
use petgraph::dot::{Dot, Config};

use crate::manifest::cargo::CargoDependency;

/// Represents the dependency graph of a project
#[derive(Debug)]
pub struct DependencyGraph {
    /// The graph structure with dependencies as nodes and relationships as edges
    pub graph: DiGraph<String, ()>,
    /// Maps dependency names to their node indices
    pub node_indices: HashMap<String, NodeIndex>,
}

impl DependencyGraph {
    /// Create a new dependency graph from the list of dependencies
    pub fn new(dependencies: &[CargoDependency]) -> Self {
        let mut graph = DiGraph::new();
        let mut node_indices = HashMap::new();
        
        // Add all dependencies as nodes in the graph
        for dep in dependencies {
            let node_idx = graph.add_node(dep.name.clone());
            node_indices.insert(dep.name.clone(), node_idx);
        }
        
        // For now, we have a simple graph with just nodes (no edges)
        // In a more complete implementation, we would determine dependency relationships
        // This would involve analyzing Cargo.lock or using cargo-metadata
        
        Self {
            graph,
            node_indices,
        }
    }
    
    /// Calculate the transitive dependencies for each dependency
    pub fn calculate_transitive_dependencies(&self) -> HashMap<String, Vec<String>> {
        let mut result = HashMap::new();
        
        for (dep_name, &node_idx) in &self.node_indices {
            // For each dependency, find all nodes reachable from it
            let reachable = algo::has_path_connecting(&self.graph, node_idx, node_idx, None);
            
            // Collect the names of all reachable dependencies
            let deps: Vec<String> = self.node_indices.iter()
                .filter_map(|(name, &idx)| {
                    if idx != node_idx && reachable {
                        Some(name.clone())
                    } else {
                        None
                    }
                })
                .collect();
            
            result.insert(dep_name.clone(), deps);
        }
        
        result
    }
    
    /// Find circular dependencies in the graph
    pub fn find_circular_dependencies(&self) -> Vec<Vec<String>> {
        let sccs = algo::tarjan_scc(&self.graph);
        
        // Filter for strongly connected components with more than one node
        sccs.into_iter()
            .filter(|scc| scc.len() > 1)
            .map(|scc| {
                // Convert node indices back to dependency names
                scc.into_iter()
                    .filter_map(|idx| {
                        let name = self.graph[idx].clone();
                        Some(name)
                    })
                    .collect()
            })
            .collect()
    }
    
    /// Generate a DOT representation of the dependency graph for visualization
    pub fn to_dot(&self) -> String {
        format!("{:?}", Dot::with_config(&self.graph, &[Config::EdgeNoLabel]))
    }
    
    /// Save the graph to a DOT file for visualization
    pub fn save_dot<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let dot = self.to_dot();
        std::fs::write(path, dot)?;
        Ok(())
    }
    
    /// Add an edge representing a dependency relationship
    pub fn add_dependency(&mut self, dependent: &str, dependency: &str) -> Result<()> {
        let dependent_idx = self.node_indices.get(dependent)
            .ok_or_else(|| anyhow::anyhow!("Dependent {} not found in graph", dependent))?;
        
        let dependency_idx = self.node_indices.get(dependency)
            .ok_or_else(|| anyhow::anyhow!("Dependency {} not found in graph", dependency))?;
        
        // Add edge from dependent to dependency
        self.graph.add_edge(*dependent_idx, *dependency_idx, ());
        
        Ok(())
    }
    
    /// Build a dependency graph from Cargo.lock
    pub fn from_cargo_lock<P: AsRef<Path>>(path: P, dependencies: &[CargoDependency]) -> Result<Self> {
        let mut graph = Self::new(dependencies);
        
        // Parse Cargo.lock to extract dependency relationships
        // This is a simplified implementation. In a complete version,
        // we would parse the Cargo.lock file and build the graph
        // based on the dependency relationships specified there.
        
        // For now, we'll simulate by adding some placeholder relationships
        // In a real implementation, this would be determined from Cargo.lock
        
        Ok(graph)
    }
} 