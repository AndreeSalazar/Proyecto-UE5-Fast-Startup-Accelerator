//! UE5 Fast Startup Accelerator - Library
//! Copyright 2026 Eddi Andreé Salazar Matos
//! Licensed under Apache 2.0

pub mod analyzer;
pub mod cache;
pub mod graph;
pub mod hash;
pub mod scanner;
pub mod asm_bindings;
pub mod uasset;

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FastStartupError {
    #[error("Project not found: {0}")]
    ProjectNotFound(String),

    #[error("Invalid UE5 project: {0}")]
    InvalidProject(String),

    #[error("Asset error: {0}")]
    AssetError(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    SerializationError(String),
}

pub type Result<T> = std::result::Result<T, FastStartupError>;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const CACHE_MAGIC: &[u8; 8] = b"UEFAST01";
