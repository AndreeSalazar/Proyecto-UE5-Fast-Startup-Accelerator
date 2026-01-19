//! Asset Scanner Module
//! Copyright 2026 Eddi Andreé Salazar Matos
//! Licensed under Apache 2.0
//!
//! Parallel asset discovery for UE5 projects

use crate::{FastStartupError, Result};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    pub path: PathBuf,
    pub relative_path: String,
    pub asset_type: AssetType,
    pub size_bytes: u64,
    pub modified: u64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AssetType {
    UAsset,
    UMap,
    UExp,
    UBulk,
    Shader,
    Texture,
    Audio,
    Animation,
    Blueprint,
    Material,
    Other,
}

impl AssetType {
    pub fn from_extension(ext: &str) -> Self {
        match ext.to_lowercase().as_str() {
            "uasset" => AssetType::UAsset,
            "umap" => AssetType::UMap,
            "uexp" => AssetType::UExp,
            "ubulk" => AssetType::UBulk,
            "ushaderbytecode" | "ush" => AssetType::Shader,
            "png" | "jpg" | "tga" | "dds" | "exr" => AssetType::Texture,
            "wav" | "ogg" | "mp3" => AssetType::Audio,
            "uanimation" => AssetType::Animation,
            _ => AssetType::Other,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            AssetType::UAsset => "uasset",
            AssetType::UMap => "umap",
            AssetType::UExp => "uexp",
            AssetType::UBulk => "ubulk",
            AssetType::Shader => "shader",
            AssetType::Texture => "texture",
            AssetType::Audio => "audio",
            AssetType::Animation => "animation",
            AssetType::Blueprint => "blueprint",
            AssetType::Material => "material",
            AssetType::Other => "other",
        }
    }
}

pub struct AssetScanner {
    project_root: PathBuf,
    content_dir: PathBuf,
}

impl AssetScanner {
    pub fn new(project_root: &Path) -> Result<Self> {
        let project_root = project_root.to_path_buf();
        
        // Find Content directory
        let content_dir = project_root.join("Content");
        if !content_dir.exists() {
            return Err(FastStartupError::InvalidProject(
                format!("Content directory not found: {}", content_dir.display())
            ));
        }

        Ok(Self {
            project_root,
            content_dir,
        })
    }

    pub fn scan_all(&self, filter: Option<&str>) -> Result<Vec<AssetInfo>> {
        info!("Scanning assets in: {}", self.content_dir.display());

        let entries: Vec<_> = WalkDir::new(&self.content_dir)
            .follow_links(true)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .collect();

        info!("Found {} files, processing...", entries.len());

        let assets: Vec<AssetInfo> = entries
            .par_iter()
            .filter_map(|entry| {
                let path = entry.path();
                let ext = path.extension()?.to_str()?;
                
                // Apply filter if specified
                if let Some(filter_ext) = filter {
                    if !ext.eq_ignore_ascii_case(filter_ext) {
                        return None;
                    }
                }

                let asset_type = AssetType::from_extension(ext);
                
                // Skip non-asset files unless explicitly filtered
                if filter.is_none() && matches!(asset_type, AssetType::Other) {
                    return None;
                }

                let metadata = entry.metadata().ok()?;
                let modified = metadata.modified().ok()?
                    .duration_since(std::time::UNIX_EPOCH).ok()?
                    .as_secs();

                let relative_path = path.strip_prefix(&self.project_root)
                    .ok()?
                    .to_string_lossy()
                    .to_string();

                Some(AssetInfo {
                    path: path.to_path_buf(),
                    relative_path,
                    asset_type,
                    size_bytes: metadata.len(),
                    modified,
                })
            })
            .collect();

        info!("Processed {} assets", assets.len());
        Ok(assets)
    }

    pub fn scan_by_type(&self, asset_type: AssetType) -> Result<Vec<AssetInfo>> {
        self.scan_all(Some(asset_type.as_str()))
    }

    pub fn scan_startup_critical(&self) -> Result<Vec<AssetInfo>> {
        info!("Scanning startup-critical assets...");

        let all_assets = self.scan_all(None)?;
        
        // Filter for assets that are typically loaded at startup
        let critical: Vec<_> = all_assets
            .into_iter()
            .filter(|asset| {
                // Maps are always startup-critical
                if asset.asset_type == AssetType::UMap {
                    return true;
                }
                
                // Check for common startup paths
                let path_lower = asset.relative_path.to_lowercase();
                path_lower.contains("startup") ||
                path_lower.contains("default") ||
                path_lower.contains("core") ||
                path_lower.contains("engine") ||
                path_lower.contains("ui") ||
                path_lower.contains("hud")
            })
            .collect();

        info!("Found {} startup-critical assets", critical.len());
        Ok(critical)
    }

    pub fn get_total_size(&self) -> Result<u64> {
        let assets = self.scan_all(None)?;
        Ok(assets.iter().map(|a| a.size_bytes).sum())
    }

    pub fn project_root(&self) -> &Path {
        &self.project_root
    }

    pub fn content_dir(&self) -> &Path {
        &self.content_dir
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScanReport {
    pub total_assets: usize,
    pub by_type: std::collections::HashMap<String, usize>,
    pub total_size_bytes: u64,
    pub scan_duration_ms: u64,
}

impl ScanReport {
    pub fn from_assets(assets: &[AssetInfo], duration_ms: u64) -> Self {
        let mut by_type = std::collections::HashMap::new();
        
        for asset in assets {
            *by_type.entry(asset.asset_type.as_str().to_string()).or_insert(0) += 1;
        }

        Self {
            total_assets: assets.len(),
            by_type,
            total_size_bytes: assets.iter().map(|a| a.size_bytes).sum(),
            scan_duration_ms: duration_ms,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_asset_type_from_extension() {
        assert_eq!(AssetType::from_extension("uasset"), AssetType::UAsset);
        assert_eq!(AssetType::from_extension("UMAP"), AssetType::UMap);
        assert_eq!(AssetType::from_extension("png"), AssetType::Texture);
        assert_eq!(AssetType::from_extension("unknown"), AssetType::Other);
    }
}
