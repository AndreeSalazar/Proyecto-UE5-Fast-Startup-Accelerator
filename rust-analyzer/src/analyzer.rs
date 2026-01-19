//! Startup Analyzer Module
//! Copyright 2026 Eddi Andreé Salazar Matos
//! Licensed under Apache 2.0
//!
//! Analyzes UE5 project startup patterns and provides optimization recommendations

use crate::graph::DependencyGraph;
use crate::hash::hash_file;
use crate::scanner::{AssetInfo, AssetScanner, AssetType};
use crate::Result;
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tracing::info;

pub struct StartupAnalyzer {
    project_root: PathBuf,
}

impl StartupAnalyzer {
    pub fn new(project_root: &Path) -> Result<Self> {
        Ok(Self {
            project_root: project_root.to_path_buf(),
        })
    }

    pub fn analyze(&self, include_shaders: bool) -> Result<AnalysisReport> {
        info!("Starting project analysis...");

        let scanner = AssetScanner::new(&self.project_root)?;
        let assets = scanner.scan_all(None)?;

        let total_assets = assets.len();
        let total_size: u64 = assets.iter().map(|a| a.size_bytes).sum();

        // Identify startup-critical assets
        let startup_assets = scanner.scan_startup_critical()?;
        let startup_count = startup_assets.len();
        let startup_size: u64 = startup_assets.iter().map(|a| a.size_bytes).sum();

        // Build dependency graph
        let graph = DependencyGraph::build(&self.project_root)?;

        // Analyze asset types
        let mut by_type: HashMap<String, TypeStats> = HashMap::new();
        for asset in &assets {
            let entry = by_type
                .entry(asset.asset_type.as_str().to_string())
                .or_insert(TypeStats::default());
            entry.count += 1;
            entry.total_size += asset.size_bytes;
        }

        // Find duplicate content
        let duplicates = self.find_duplicates(&assets)?;

        // Analyze shader usage if requested
        let shader_analysis = if include_shaders {
            Some(self.analyze_shaders(&assets)?)
        } else {
            None
        };

        // Calculate estimated savings
        let estimated_savings = self.estimate_savings(
            total_assets,
            startup_count,
            &duplicates,
        );

        let recommendations = self.generate_recommendations(
            total_assets,
            startup_count,
            &by_type,
        );

        let report = AnalysisReport {
            project_name: self.project_root
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default(),
            total_assets,
            startup_assets: startup_count,
            total_size_bytes: total_size,
            startup_size_bytes: startup_size,
            by_type,
            dependency_count: graph.edge_count(),
            duplicate_count: duplicates.len(),
            duplicates,
            shader_analysis,
            estimated_savings_seconds: estimated_savings,
            recommendations,
        };

        info!("Analysis complete");
        Ok(report)
    }

    fn find_duplicates(&self, assets: &[AssetInfo]) -> Result<Vec<DuplicateGroup>> {
        info!("Scanning for duplicate content...");

        // Hash all assets
        let hashes: Vec<_> = assets
            .par_iter()
            .filter_map(|asset| {
                let hash = hash_file(&asset.path).ok()?;
                Some((hash.as_u64(), asset.relative_path.clone(), asset.size_bytes))
            })
            .collect();

        // Group by hash
        let mut hash_groups: HashMap<u64, Vec<(String, u64)>> = HashMap::new();
        for (hash, path, size) in hashes {
            hash_groups
                .entry(hash)
                .or_default()
                .push((path, size));
        }

        // Find duplicates (groups with more than one file)
        let duplicates: Vec<_> = hash_groups
            .into_iter()
            .filter(|(_, files)| files.len() > 1)
            .map(|(hash, files)| {
                let wasted_bytes = files.iter().skip(1).map(|(_, s)| s).sum();
                DuplicateGroup {
                    hash,
                    files: files.into_iter().map(|(p, _)| p).collect(),
                    wasted_bytes,
                }
            })
            .collect();

        info!("Found {} duplicate groups", duplicates.len());
        Ok(duplicates)
    }

    fn analyze_shaders(&self, assets: &[AssetInfo]) -> Result<ShaderAnalysis> {
        let shader_assets: Vec<_> = assets
            .iter()
            .filter(|a| a.asset_type == AssetType::Shader)
            .collect();

        let total_shaders = shader_assets.len();
        let total_size: u64 = shader_assets.iter().map(|a| a.size_bytes).sum();

        Ok(ShaderAnalysis {
            total_shaders,
            total_size_bytes: total_size,
            estimated_compile_time_seconds: (total_shaders as f64 * 0.5) as u64,
        })
    }

    fn estimate_savings(
        &self,
        total_assets: usize,
        startup_assets: usize,
        duplicates: &[DuplicateGroup],
    ) -> f64 {
        // Rough estimation based on typical UE5 startup patterns
        let non_startup = total_assets - startup_assets;
        let deferred_load_savings = non_startup as f64 * 0.01; // ~10ms per deferred asset
        
        let duplicate_savings = duplicates.len() as f64 * 0.05; // ~50ms per duplicate avoided
        
        deferred_load_savings + duplicate_savings
    }

    fn generate_recommendations(
        &self,
        total_assets: usize,
        startup_assets: usize,
        by_type: &HashMap<String, TypeStats>,
    ) -> Vec<Recommendation> {
        let mut recommendations = Vec::new();

        // Check startup ratio
        let startup_ratio = startup_assets as f64 / total_assets as f64;
        if startup_ratio > 0.3 {
            recommendations.push(Recommendation {
                priority: Priority::High,
                category: "Startup".to_string(),
                message: format!(
                    "{}% of assets are loaded at startup. Consider lazy loading.",
                    (startup_ratio * 100.0) as u32
                ),
                estimated_impact_seconds: startup_ratio * 10.0,
            });
        }

        // Check texture count
        if let Some(textures) = by_type.get("texture") {
            if textures.count > 1000 {
                recommendations.push(Recommendation {
                    priority: Priority::Medium,
                    category: "Textures".to_string(),
                    message: format!(
                        "{} textures found. Consider using texture streaming.",
                        textures.count
                    ),
                    estimated_impact_seconds: 5.0,
                });
            }
        }

        // Check blueprint count
        if let Some(blueprints) = by_type.get("blueprint") {
            if blueprints.count > 500 {
                recommendations.push(Recommendation {
                    priority: Priority::Medium,
                    category: "Blueprints".to_string(),
                    message: format!(
                        "{} blueprints found. Consider nativizing hot paths.",
                        blueprints.count
                    ),
                    estimated_impact_seconds: 3.0,
                });
            }
        }

        recommendations
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub project_name: String,
    pub total_assets: usize,
    pub startup_assets: usize,
    pub total_size_bytes: u64,
    pub startup_size_bytes: u64,
    pub by_type: HashMap<String, TypeStats>,
    pub dependency_count: usize,
    pub duplicate_count: usize,
    pub duplicates: Vec<DuplicateGroup>,
    pub shader_analysis: Option<ShaderAnalysis>,
    pub estimated_savings_seconds: f64,
    pub recommendations: Vec<Recommendation>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TypeStats {
    pub count: usize,
    pub total_size: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DuplicateGroup {
    pub hash: u64,
    pub files: Vec<String>,
    pub wasted_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ShaderAnalysis {
    pub total_shaders: usize,
    pub total_size_bytes: u64,
    pub estimated_compile_time_seconds: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Recommendation {
    pub priority: Priority,
    pub category: String,
    pub message: String,
    pub estimated_impact_seconds: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Priority {
    High,
    Medium,
    Low,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_stats_default() {
        let stats = TypeStats::default();
        assert_eq!(stats.count, 0);
        assert_eq!(stats.total_size, 0);
    }
}
