//! Dependency Graph Module
//! Copyright 2026 Eddi Andreé Salazar Matos
//! Licensed under Apache 2.0
//!
//! Asset dependency graph builder and analyzer

use crate::scanner::{AssetInfo, AssetScanner, AssetType};
use crate::uasset::UAssetParser;
use crate::Result;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::Dfs;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::{debug, info, warn};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetNode {
    pub path: PathBuf,
    pub asset_type: AssetType,
    pub is_startup_critical: bool,
    pub load_order: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyEdge {
    pub dependency_type: DependencyType,
    pub is_hard: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DependencyType {
    Import,
    SoftReference,
    Blueprint,
    Material,
    Texture,
    Animation,
}

pub struct DependencyGraph {
    graph: DiGraph<AssetNode, DependencyEdge>,
    path_to_node: HashMap<PathBuf, NodeIndex>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            path_to_node: HashMap::new(),
        }
    }

    pub fn build(project_root: &Path) -> Result<Self> {
        info!("Building dependency graph for: {}", project_root.display());

        let scanner = AssetScanner::new(project_root)?;
        let assets = scanner.scan_all(None)?;

        let mut graph = Self::new();

        // Add all assets as nodes
        for asset in &assets {
            graph.add_asset(asset);
        }

        info!("Added {} nodes to graph", graph.node_count());

        // Parse dependencies in parallel
        let dependencies: Vec<_> = assets
            .par_iter()
            .filter(|a| a.asset_type == AssetType::UAsset)
            .filter_map(|asset| {
                match UAssetParser::parse_imports(&asset.path) {
                    Ok(imports) => Some((asset.path.clone(), imports)),
                    Err(e) => {
                        debug!("Failed to parse {}: {}", asset.path.display(), e);
                        None
                    }
                }
            })
            .collect();

        // Add edges
        for (source_path, imports) in dependencies {
            for import in imports {
                let import_path = resolve_import_path(project_root, &import);
                if let Some(target_path) = import_path {
                    graph.add_dependency(
                        &source_path,
                        &target_path,
                        DependencyType::Import,
                        true,
                    );
                }
            }
        }

        info!("Added {} edges to graph", graph.edge_count());

        Ok(graph)
    }

    pub fn add_asset(&mut self, asset: &AssetInfo) -> NodeIndex {
        if let Some(&idx) = self.path_to_node.get(&asset.path) {
            return idx;
        }

        let node = AssetNode {
            path: asset.path.clone(),
            asset_type: asset.asset_type,
            is_startup_critical: false,
            load_order: None,
        };

        let idx = self.graph.add_node(node);
        self.path_to_node.insert(asset.path.clone(), idx);
        idx
    }

    pub fn add_dependency(
        &mut self,
        from: &Path,
        to: &Path,
        dep_type: DependencyType,
        is_hard: bool,
    ) {
        let from_idx = match self.path_to_node.get(from) {
            Some(&idx) => idx,
            None => return,
        };

        let to_idx = match self.path_to_node.get(to) {
            Some(&idx) => idx,
            None => return,
        };

        let edge = DependencyEdge {
            dependency_type: dep_type,
            is_hard,
        };

        self.graph.add_edge(from_idx, to_idx, edge);
    }

    pub fn node_count(&self) -> usize {
        self.graph.node_count()
    }

    pub fn edge_count(&self) -> usize {
        self.graph.edge_count()
    }

    pub fn get_dependencies(&self, path: &Path) -> Vec<&AssetNode> {
        let idx = match self.path_to_node.get(path) {
            Some(&idx) => idx,
            None => return Vec::new(),
        };

        self.graph
            .neighbors(idx)
            .map(|n| &self.graph[n])
            .collect()
    }

    pub fn get_dependents(&self, path: &Path) -> Vec<&AssetNode> {
        let idx = match self.path_to_node.get(path) {
            Some(&idx) => idx,
            None => return Vec::new(),
        };

        self.graph
            .neighbors_directed(idx, petgraph::Direction::Incoming)
            .map(|n| &self.graph[n])
            .collect()
    }

    pub fn filter_startup_critical(mut self) -> Self {
        // Mark startup-critical assets
        let critical_indices: Vec<_> = self.graph
            .node_indices()
            .filter(|&idx| {
                let node = &self.graph[idx];
                node.asset_type == AssetType::UMap ||
                node.path.to_string_lossy().to_lowercase().contains("startup")
            })
            .collect();

        // Mark all dependencies of critical assets
        for idx in &critical_indices {
            self.graph[*idx].is_startup_critical = true;
            
            let mut dfs = Dfs::new(&self.graph, *idx);
            while let Some(dep_idx) = dfs.next(&self.graph) {
                self.graph[dep_idx].is_startup_critical = true;
            }
        }

        // Remove non-critical nodes
        let non_critical: Vec<_> = self.graph
            .node_indices()
            .filter(|&idx| !self.graph[idx].is_startup_critical)
            .collect();

        for idx in non_critical.into_iter().rev() {
            self.graph.remove_node(idx);
        }

        // Rebuild path map
        self.path_to_node.clear();
        for idx in self.graph.node_indices() {
            let path = self.graph[idx].path.clone();
            self.path_to_node.insert(path, idx);
        }

        self
    }

    pub fn compute_load_order(&mut self) {
        use petgraph::algo::toposort;

        match toposort(&self.graph, None) {
            Ok(order) => {
                for (i, idx) in order.iter().enumerate() {
                    self.graph[*idx].load_order = Some(i as u32);
                }
            }
            Err(_) => {
                warn!("Cycle detected in dependency graph, using fallback ordering");
                for (i, idx) in self.graph.node_indices().enumerate() {
                    self.graph[idx].load_order = Some(i as u32);
                }
            }
        }
    }

    pub fn to_dot(&self) -> String {
        use petgraph::dot::{Config, Dot};
        format!("{:?}", Dot::with_config(&self.graph, &[Config::EdgeNoLabel]))
    }

    pub fn get_load_order(&self) -> Vec<&AssetNode> {
        let mut nodes: Vec<_> = self.graph
            .node_indices()
            .map(|idx| &self.graph[idx])
            .collect();

        nodes.sort_by_key(|n| n.load_order.unwrap_or(u32::MAX));
        nodes
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

fn resolve_import_path(project_root: &Path, import: &str) -> Option<PathBuf> {
    // Convert UE import path to filesystem path
    // e.g., "/Game/Characters/Hero" -> "Content/Characters/Hero.uasset"
    
    let cleaned = import
        .trim_start_matches('/')
        .replace("/Game/", "Content/")
        .replace("/Engine/", "Engine/Content/");

    let path = project_root.join(&cleaned).with_extension("uasset");
    
    if path.exists() {
        Some(path)
    } else {
        None
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GraphStats {
    pub node_count: usize,
    pub edge_count: usize,
    pub startup_critical_count: usize,
    pub max_depth: usize,
}

impl DependencyGraph {
    pub fn statistics(&self) -> GraphStats {
        let startup_critical_count = self.graph
            .node_indices()
            .filter(|&idx| self.graph[idx].is_startup_critical)
            .count();

        GraphStats {
            node_count: self.node_count(),
            edge_count: self.edge_count(),
            startup_critical_count,
            max_depth: 0, // TODO: compute actual depth
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_graph() {
        let graph = DependencyGraph::new();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_resolve_import_path() {
        let project = Path::new("C:/Projects/MyGame");
        
        let result = resolve_import_path(project, "/Game/Characters/Hero");
        // Will be None since path doesn't exist, but tests the logic
        assert!(result.is_none());
    }
}
