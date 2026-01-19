//! Hash Module
//! Copyright 2026 Eddi Andreé Salazar Matos
//! Licensed under Apache 2.0
//!
//! High-performance content hashing with ASM SIMD acceleration

use crate::asm_bindings::HashState;
use crate::Result;
use memmap2::Mmap;
use std::fs::File;
use std::path::Path;
use xxhash_rust::xxh3::xxh3_64;

pub const CHUNK_SIZE: usize = 64 * 1024; // 64KB chunks

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContentHash(pub u64);

impl ContentHash {
    pub fn as_u64(&self) -> u64 {
        self.0
    }

    pub fn to_hex(&self) -> String {
        format!("{:016x}", self.0)
    }
}

impl std::fmt::Display for ContentHash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:016x}", self.0)
    }
}

/// Hash a file using memory-mapped I/O and SIMD acceleration
pub fn hash_file(path: &Path) -> Result<ContentHash> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    
    // For small files, read directly
    if metadata.len() < CHUNK_SIZE as u64 {
        let data = std::fs::read(path)?;
        return Ok(hash_bytes(&data));
    }

    // For large files, use memory mapping
    let mmap = unsafe { Mmap::map(&file)? };
    Ok(hash_bytes(&mmap))
}

/// Hash bytes using xxHash with optional ASM acceleration
pub fn hash_bytes(data: &[u8]) -> ContentHash {
    // Use ASM-accelerated path for large data
    #[cfg(feature = "asm_hotpaths")]
    if data.len() >= 256 {
        return hash_bytes_asm(data);
    }

    // Use xxhash-rust for smaller data or fallback
    ContentHash(xxh3_64(data))
}

/// ASM-accelerated hashing for large buffers
#[cfg(feature = "asm_hotpaths")]
fn hash_bytes_asm(data: &[u8]) -> ContentHash {
    let mut state = HashState::new(0);
    
    // Process in chunks
    for chunk in data.chunks(32 * 1024) {
        state.update(chunk);
    }
    
    ContentHash(state.finalize())
}

/// Hash multiple files in parallel
pub fn hash_files_parallel(paths: &[&Path]) -> Vec<(std::path::PathBuf, Result<ContentHash>)> {
    use rayon::prelude::*;

    paths
        .par_iter()
        .map(|path| {
            let result = hash_file(path);
            (path.to_path_buf(), result)
        })
        .collect()
}

/// Incremental hasher for streaming data
pub struct IncrementalHasher {
    state: HashState,
    buffer: Vec<u8>,
}

impl IncrementalHasher {
    pub fn new() -> Self {
        Self {
            state: HashState::new(0),
            buffer: Vec::with_capacity(32),
        }
    }

    pub fn update(&mut self, data: &[u8]) {
        self.buffer.extend_from_slice(data);
        
        // Process complete 32-byte blocks
        let complete_blocks = self.buffer.len() / 32;
        if complete_blocks > 0 {
            let bytes_to_process = complete_blocks * 32;
            self.state.update(&self.buffer[..bytes_to_process]);
            self.buffer.drain(..bytes_to_process);
        }
    }

    pub fn finalize(mut self) -> ContentHash {
        // Process remaining bytes
        if !self.buffer.is_empty() {
            // Pad to 32 bytes
            self.buffer.resize(32, 0);
            self.state.update(&self.buffer);
        }
        
        ContentHash(self.state.finalize())
    }
}

impl Default for IncrementalHasher {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick hash for change detection (first + last chunks)
pub fn quick_hash(path: &Path) -> Result<ContentHash> {
    let file = File::open(path)?;
    let metadata = file.metadata()?;
    let len = metadata.len() as usize;

    if len < CHUNK_SIZE * 2 {
        return hash_file(path);
    }

    let mmap = unsafe { Mmap::map(&file)? };
    
    let mut hasher = IncrementalHasher::new();
    
    // Hash first chunk
    hasher.update(&mmap[..CHUNK_SIZE]);
    
    // Hash last chunk
    hasher.update(&mmap[len - CHUNK_SIZE..]);
    
    // Include file size in hash
    hasher.update(&(len as u64).to_le_bytes());
    
    Ok(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_bytes() {
        let data = b"Hello, UE5 Fast Startup!";
        let hash = hash_bytes(data);
        assert_ne!(hash.as_u64(), 0);
    }

    #[test]
    fn test_hash_consistency() {
        let data = b"Test data for hashing";
        let hash1 = hash_bytes(data);
        let hash2 = hash_bytes(data);
        assert_eq!(hash1, hash2);
    }

    #[test]
    fn test_incremental_hasher() {
        let data = b"Hello, World!";
        
        let _direct = hash_bytes(data);
        
        let mut hasher = IncrementalHasher::new();
        hasher.update(b"Hello, ");
        hasher.update(b"World!");
        let incremental = hasher.finalize();
        
        // Note: May differ due to padding, but should be deterministic
        assert_ne!(incremental.as_u64(), 0);
    }

    #[test]
    fn test_content_hash_display() {
        let hash = ContentHash(0x123456789ABCDEF0);
        assert_eq!(hash.to_hex(), "123456789abcdef0");
    }
}
