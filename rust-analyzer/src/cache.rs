//! Cache Module
//! Copyright 2026 Eddi Andreé Salazar Matos
//! Licensed under Apache 2.0
//!
//! Startup cache generation and management

use crate::graph::DependencyGraph;
use crate::hash::hash_file;
use crate::scanner::{AssetScanner, AssetType};
use crate::{FastStartupError, Result, CACHE_MAGIC};
use chrono::{DateTime, Utc};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CachedAsset {
    pub relative_path: String,
    pub asset_type: AssetType,
    pub content_hash: u64,
    pub size_bytes: u64,
    pub load_order: u32,
    pub is_startup_critical: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StartupCache {
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub project_name: String,
    pub hash_algorithm: String,
    pub assets: Vec<CachedAsset>,
    pub load_order: Vec<String>,
    pub shader_variants: Vec<ShaderVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShaderVariant {
    pub name: String,
    pub hash: u64,
    pub platform: String,
}

impl StartupCache {
    pub fn new(project_name: &str) -> Self {
        Self {
            version: crate::VERSION.to_string(),
            created_at: Utc::now(),
            project_name: project_name.to_string(),
            hash_algorithm: "xxh3".to_string(),
            assets: Vec::new(),
            load_order: Vec::new(),
            shader_variants: Vec::new(),
        }
    }

    pub fn save(&self, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Write magic bytes
        writer.write_all(CACHE_MAGIC)?;

        // Write cache data as bincode
        bincode::serialize_into(&mut writer, self)
            .map_err(|e| FastStartupError::SerializationError(e.to_string()))?;

        writer.flush()?;
        info!("Cache saved to: {}", path.display());
        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Verify magic bytes
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;

        if &magic != CACHE_MAGIC {
            return Err(FastStartupError::CacheError(
                "Invalid cache file format".to_string()
            ));
        }

        // Read cache data
        let cache: StartupCache = bincode::deserialize_from(&mut reader)
            .map_err(|e| FastStartupError::SerializationError(e.to_string()))?;

        info!("Cache loaded: {} assets", cache.assets.len());
        Ok(cache)
    }

    pub fn verify(&self, project_root: &Path) -> Result<VerifyResult> {
        info!("Verifying cache against project...");

        let scanner = AssetScanner::new(project_root)?;
        let current_assets = scanner.scan_all(None)?;

        let current_map: HashMap<_, _> = current_assets
            .iter()
            .map(|a| (a.relative_path.clone(), a))
            .collect();

        let mut matching = 0;
        let mut changed = Vec::new();
        let mut missing = Vec::new();

        for cached in &self.assets {
            match current_map.get(&cached.relative_path) {
                Some(current) => {
                    // Check if hash matches
                    if let Ok(hash) = hash_file(&current.path) {
                        if hash.as_u64() == cached.content_hash {
                            matching += 1;
                        } else {
                            changed.push(cached.relative_path.clone());
                        }
                    } else {
                        changed.push(cached.relative_path.clone());
                    }
                }
                None => {
                    missing.push(cached.relative_path.clone());
                }
            }
        }

        let is_valid = changed.is_empty() && missing.is_empty();

        Ok(VerifyResult {
            is_valid,
            total_assets: self.assets.len(),
            matching_assets: matching,
            changed_assets: changed,
            missing_assets: missing,
        })
    }

    pub fn asset_count(&self) -> usize {
        self.assets.len()
    }

    pub fn size_bytes(&self) -> usize {
        bincode::serialized_size(self).unwrap_or(0) as usize
    }

    pub fn statistics(&self) -> CacheStats {
        CacheStats {
            version: self.version.clone(),
            created_at: self.created_at.to_rfc3339(),
            asset_count: self.assets.len(),
            size_bytes: self.size_bytes(),
            hash_algorithm: self.hash_algorithm.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VerifyResult {
    pub is_valid: bool,
    pub total_assets: usize,
    pub matching_assets: usize,
    pub changed_assets: Vec<String>,
    pub missing_assets: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CacheStats {
    pub version: String,
    pub created_at: String,
    pub asset_count: usize,
    pub size_bytes: usize,
    pub hash_algorithm: String,
}

pub struct CacheBuilder {
    project_root: PathBuf,
    include_shaders: bool,
}

impl CacheBuilder {
    pub fn new(project_root: &Path) -> Result<Self> {
        if !project_root.exists() {
            return Err(FastStartupError::ProjectNotFound(
                project_root.display().to_string()
            ));
        }

        Ok(Self {
            project_root: project_root.to_path_buf(),
            include_shaders: true,
        })
    }

    pub fn include_shaders(mut self, include: bool) -> Self {
        self.include_shaders = include;
        self
    }

    pub fn build(&self) -> Result<StartupCache> {
        info!("Building startup cache...");

        let project_name = self.project_root
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "Unknown".to_string());

        let mut cache = StartupCache::new(&project_name);

        // Scan assets
        let scanner = AssetScanner::new(&self.project_root)?;
        let assets = scanner.scan_all(None)?;

        info!("Hashing {} assets...", assets.len());

        // Hash assets in parallel
        let cached_assets: Vec<CachedAsset> = assets
            .par_iter()
            .enumerate()
            .filter_map(|(idx, asset)| {
                let hash = hash_file(&asset.path).ok()?;
                
                Some(CachedAsset {
                    relative_path: asset.relative_path.clone(),
                    asset_type: asset.asset_type,
                    content_hash: hash.as_u64(),
                    size_bytes: asset.size_bytes,
                    load_order: idx as u32,
                    is_startup_critical: false,
                })
            })
            .collect();

        cache.assets = cached_assets;

        // Build dependency graph and compute load order
        info!("Computing optimal load order...");
        let mut graph = DependencyGraph::build(&self.project_root)?;
        graph.compute_load_order();

        let ordered_nodes = graph.get_load_order();
        cache.load_order = ordered_nodes
            .iter()
            .map(|n| n.path.to_string_lossy().to_string())
            .collect();

        // Mark startup-critical assets
        for node in ordered_nodes {
            if node.is_startup_critical {
                if let Some(cached) = cache.assets
                    .iter_mut()
                    .find(|a| a.relative_path == node.path.to_string_lossy())
                {
                    cached.is_startup_critical = true;
                }
            }
        }

        info!("Cache built: {} assets", cache.assets.len());
        Ok(cache)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_new() {
        let cache = StartupCache::new("TestProject");
        assert_eq!(cache.project_name, "TestProject");
        assert!(cache.assets.is_empty());
    }

    #[test]
    fn test_cache_stats() {
        let cache = StartupCache::new("TestProject");
        let stats = cache.statistics();
        assert_eq!(stats.asset_count, 0);
    }
}
