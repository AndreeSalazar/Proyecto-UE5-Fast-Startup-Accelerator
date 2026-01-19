//! UE5 Fast Startup Accelerator - CLI Entry Point
//! Copyright 2026 Eddi Andreé Salazar Matos
//! Licensed under Apache 2.0

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::path::PathBuf;
use tracing::{info, Level};
use tracing_subscriber::FmtSubscriber;

use ue5_fast_startup::{
    cache::CacheBuilder,
    scanner::AssetScanner,
    graph::DependencyGraph,
    analyzer::StartupAnalyzer,
};

#[derive(Parser)]
#[command(name = "ue5-fast-startup")]
#[command(author = "Eddi Andreé Salazar Matos")]
#[command(version = "0.1.0")]
#[command(about = "High-performance startup accelerator for Unreal Engine 5", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    verbose: bool,

    /// Number of threads (0 = auto)
    #[arg(short, long, global = true, default_value = "0")]
    threads: usize,
}

#[derive(Subcommand)]
enum Commands {
    /// Analyze UE5 project assets and dependencies
    Analyze {
        /// Path to UE5 project root
        #[arg(short, long)]
        project: PathBuf,

        /// Output analysis report as JSON
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Include shader analysis
        #[arg(long)]
        shaders: bool,
    },

    /// Scan project for assets
    Scan {
        /// Path to UE5 project root
        #[arg(short, long)]
        project: PathBuf,

        /// Output asset list as JSON
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Filter by asset type (e.g., "uasset", "umap")
        #[arg(short, long)]
        filter: Option<String>,
    },

    /// Build startup cache
    Cache {
        /// Path to UE5 project root
        #[arg(short, long)]
        project: PathBuf,

        /// Output cache file (.uefast)
        #[arg(short, long)]
        output: PathBuf,

        /// Force rebuild even if cache exists
        #[arg(short, long)]
        force: bool,
    },

    /// Verify existing cache
    Verify {
        /// Path to cache file
        #[arg(short, long)]
        cache: PathBuf,

        /// Path to UE5 project root
        #[arg(short, long)]
        project: PathBuf,
    },

    /// Show cache statistics
    Stats {
        /// Path to cache file
        #[arg(short, long)]
        cache: PathBuf,
    },

    /// Build dependency graph
    Graph {
        /// Path to UE5 project root
        #[arg(short, long)]
        project: PathBuf,

        /// Output graph as DOT format
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Include only startup-critical assets
        #[arg(long)]
        startup_only: bool,
    },

    /// Benchmark performance
    Bench {
        /// Path to UE5 project root
        #[arg(short, long)]
        project: PathBuf,

        /// Number of iterations
        #[arg(short, long, default_value = "3")]
        iterations: u32,
    },

    /// TURBO mode - ultra-fast cache with sampling
    Turbo {
        /// Path to UE5 project root
        #[arg(short, long)]
        project: PathBuf,

        /// Output cache file (.uefast)
        #[arg(short, long)]
        output: PathBuf,
    },

    /// Quick verify - fast change detection
    QuickVerify {
        /// Path to cache file
        #[arg(short, long)]
        cache: PathBuf,

        /// Path to UE5 project root
        #[arg(short, long)]
        project: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Setup logging
    let level = if cli.verbose { Level::DEBUG } else { Level::INFO };
    let subscriber = FmtSubscriber::builder()
        .with_max_level(level)
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Configure thread pool
    if cli.threads > 0 {
        rayon::ThreadPoolBuilder::new()
            .num_threads(cli.threads)
            .build_global()?;
    }

    info!("UE5 Fast Startup Accelerator v0.1.0");

    match cli.command {
        Commands::Analyze { project, output, shaders } => {
            cmd_analyze(project, output, shaders)
        }
        Commands::Scan { project, output, filter } => {
            cmd_scan(project, output, filter)
        }
        Commands::Cache { project, output, force } => {
            cmd_cache(project, output, force)
        }
        Commands::Verify { cache, project } => {
            cmd_verify(cache, project)
        }
        Commands::Stats { cache } => {
            cmd_stats(cache)
        }
        Commands::Graph { project, output, startup_only } => {
            cmd_graph(project, output, startup_only)
        }
        Commands::Bench { project, iterations } => {
            cmd_bench(project, iterations)
        }
        Commands::Turbo { project, output } => {
            cmd_turbo(project, output)
        }
        Commands::QuickVerify { cache, project } => {
            cmd_quick_verify(cache, project)
        }
    }
}

fn cmd_analyze(project: PathBuf, output: Option<PathBuf>, include_shaders: bool) -> Result<()> {
    info!("Analyzing project: {}", project.display());

    let analyzer = StartupAnalyzer::new(&project)?;
    let report = analyzer.analyze(include_shaders)?;

    info!("Analysis complete:");
    info!("  Total assets: {}", report.total_assets);
    info!("  Startup assets: {}", report.startup_assets);
    info!("  Estimated savings: {:.1}s", report.estimated_savings_seconds);

    if let Some(output_path) = output {
        let json = serde_json::to_string_pretty(&report)?;
        std::fs::write(&output_path, json)?;
        info!("Report saved to: {}", output_path.display());
    }

    Ok(())
}

fn cmd_scan(project: PathBuf, output: Option<PathBuf>, filter: Option<String>) -> Result<()> {
    info!("Scanning project: {}", project.display());

    let scanner = AssetScanner::new(&project)?;
    let assets = scanner.scan_all(filter.as_deref())?;

    info!("Found {} assets", assets.len());

    if let Some(output_path) = output {
        let json = serde_json::to_string_pretty(&assets)?;
        std::fs::write(&output_path, json)?;
        info!("Asset list saved to: {}", output_path.display());
    }

    Ok(())
}

fn cmd_cache(project: PathBuf, output: PathBuf, force: bool) -> Result<()> {
    info!("Building cache for: {}", project.display());

    if output.exists() && !force {
        info!("Cache already exists. Use --force to rebuild.");
        return Ok(());
    }

    let builder = CacheBuilder::new(&project)?;
    let cache = builder.build()?;
    cache.save(&output)?;

    info!("Cache saved to: {}", output.display());
    info!("  Assets cached: {}", cache.asset_count());
    info!("  Cache size: {} KB", cache.size_bytes() / 1024);

    Ok(())
}

fn cmd_verify(cache_path: PathBuf, project: PathBuf) -> Result<()> {
    info!("Verifying cache: {}", cache_path.display());

    let cache = ue5_fast_startup::cache::StartupCache::load(&cache_path)?;
    let result = cache.verify(&project)?;

    if result.is_valid {
        info!("✓ Cache is valid");
        info!("  Matching assets: {}/{}", result.matching_assets, result.total_assets);
    } else {
        info!("✗ Cache is invalid");
        info!("  Changed assets: {}", result.changed_assets.len());
        for asset in result.changed_assets.iter().take(10) {
            info!("    - {}", asset);
        }
        if result.changed_assets.len() > 10 {
            info!("    ... and {} more", result.changed_assets.len() - 10);
        }
    }

    Ok(())
}

fn cmd_stats(cache_path: PathBuf) -> Result<()> {
    info!("Cache statistics: {}", cache_path.display());

    let cache = ue5_fast_startup::cache::StartupCache::load(&cache_path)?;
    let stats = cache.statistics();

    info!("  Version: {}", stats.version);
    info!("  Created: {}", stats.created_at);
    info!("  Assets: {}", stats.asset_count);
    info!("  Size: {} KB", stats.size_bytes / 1024);
    info!("  Hash algorithm: {}", stats.hash_algorithm);

    Ok(())
}

fn cmd_graph(project: PathBuf, output: Option<PathBuf>, startup_only: bool) -> Result<()> {
    info!("Building dependency graph: {}", project.display());

    let graph = DependencyGraph::build(&project)?;
    
    let filtered = if startup_only {
        graph.filter_startup_critical()
    } else {
        graph
    };

    info!("Graph built:");
    info!("  Nodes: {}", filtered.node_count());
    info!("  Edges: {}", filtered.edge_count());

    if let Some(output_path) = output {
        let dot = filtered.to_dot();
        std::fs::write(&output_path, dot)?;
        info!("Graph saved to: {}", output_path.display());
    }

    Ok(())
}

fn cmd_bench(project: PathBuf, iterations: u32) -> Result<()> {
    info!("Benchmarking with {} iterations", iterations);

    let mut scan_times = Vec::new();
    let mut hash_times = Vec::new();

    for i in 1..=iterations {
        info!("Iteration {}/{}", i, iterations);

        // Benchmark scanning
        let start = std::time::Instant::now();
        let scanner = AssetScanner::new(&project)?;
        let assets = scanner.scan_all(None)?;
        let scan_time = start.elapsed();
        scan_times.push(scan_time);

        // Benchmark hashing
        let start = std::time::Instant::now();
        for asset in assets.iter().take(100) {
            let _ = ue5_fast_startup::hash::hash_file(&asset.path);
        }
        let hash_time = start.elapsed();
        hash_times.push(hash_time);
    }

    let avg_scan: f64 = scan_times.iter().map(|d| d.as_secs_f64()).sum::<f64>() / iterations as f64;
    let avg_hash: f64 = hash_times.iter().map(|d| d.as_secs_f64()).sum::<f64>() / iterations as f64;

    info!("Results:");
    info!("  Average scan time: {:.3}s", avg_scan);
    info!("  Average hash time (100 assets): {:.3}s", avg_hash);

    Ok(())
}

/// TURBO mode - ultra-fast cache building with sampling
fn cmd_turbo(project: PathBuf, output: PathBuf) -> Result<()> {
    use rayon::prelude::*;
    use std::time::Instant;

    info!("⚡ TURBO MODE - Ultra-fast cache building");
    info!("Project: {}", project.display());

    let start = Instant::now();

    // Step 1: Fast path-only scan
    info!("[1/3] Turbo scanning...");
    let scanner = AssetScanner::new(&project)?;
    let paths = scanner.scan_paths_only()?;
    info!("  Found {} assets in {:.2}ms", paths.len(), start.elapsed().as_millis());

    // Step 2: Parallel turbo hashing with sampling
    info!("[2/3] Turbo hashing with sampling...");
    let hash_start = Instant::now();
    
    let hashes: Vec<_> = paths
        .par_iter()
        .filter_map(|path| {
            ue5_fast_startup::hash::turbo_hash(path)
                .ok()
                .map(|h| (path.clone(), h.as_u64()))
        })
        .collect();

    info!("  Hashed {} files in {:.2}ms", hashes.len(), hash_start.elapsed().as_millis());

    // Step 3: Build minimal cache
    info!("[3/3] Building turbo cache...");
    let project_name = project
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    let mut cache = ue5_fast_startup::cache::StartupCache::new(&project_name);
    
    for (path, hash) in hashes {
        let relative = path.strip_prefix(&project)
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");
        
        cache.assets.push(ue5_fast_startup::cache::CachedAsset {
            relative_path: relative,
            asset_type: ue5_fast_startup::scanner::AssetType::from_extension(ext),
            content_hash: hash,
            size_bytes: std::fs::metadata(&path).map(|m| m.len()).unwrap_or(0),
            load_order: 0,
            is_startup_critical: false,
        });
    }

    cache.save(&output)?;

    let total_time = start.elapsed();
    info!("⚡ TURBO COMPLETE in {:.2}ms", total_time.as_millis());
    info!("  Assets: {}", cache.asset_count());
    info!("  Cache size: {} KB", cache.size_bytes() / 1024);
    info!("  Throughput: {:.0} assets/sec", cache.asset_count() as f64 / total_time.as_secs_f64());

    Ok(())
}

/// Quick verify - fast change detection using turbo hashing
fn cmd_quick_verify(cache_path: PathBuf, project: PathBuf) -> Result<()> {
    use rayon::prelude::*;
    use std::time::Instant;

    info!("⚡ Quick verify: {}", cache_path.display());

    let start = Instant::now();

    let cache = ue5_fast_startup::cache::StartupCache::load(&cache_path)?;
    
    // Build hash map of cached assets
    let cached_hashes: std::collections::HashMap<_, _> = cache.assets
        .iter()
        .map(|a| (a.relative_path.clone(), a.content_hash))
        .collect();

    // Quick scan current assets
    let scanner = AssetScanner::new(&project)?;
    let paths = scanner.scan_paths_only()?;

    // Parallel quick hash and compare
    let changes: Vec<_> = paths
        .par_iter()
        .filter_map(|path| {
            let relative = path.strip_prefix(&project)
                .ok()?
                .to_string_lossy()
                .to_string();
            
            let current_hash = ue5_fast_startup::hash::turbo_hash(path).ok()?.as_u64();
            
            match cached_hashes.get(&relative) {
                Some(&cached_hash) if cached_hash != current_hash => {
                    Some(relative)
                }
                None => Some(relative), // New file
                _ => None, // Unchanged
            }
        })
        .collect();

    let elapsed = start.elapsed();

    if changes.is_empty() {
        info!("✓ No changes detected in {:.2}ms", elapsed.as_millis());
    } else {
        info!("⚠ {} changes detected in {:.2}ms", changes.len(), elapsed.as_millis());
        for change in changes.iter().take(10) {
            info!("  - {}", change);
        }
        if changes.len() > 10 {
            info!("  ... and {} more", changes.len() - 10);
        }
    }

    Ok(())
}
