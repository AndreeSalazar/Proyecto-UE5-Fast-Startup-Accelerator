//! ASM Hot Path Bindings
//! Copyright 2026 Eddi Andreé Salazar Matos
//! Licensed under Apache 2.0
//!
//! Safe Rust wrappers for NASM-compiled assembly functions

use std::arch::asm;

#[cfg(all(target_arch = "x86_64", feature = "asm_hotpaths"))]
extern "C" {
    fn hash_block_simd(data: *const u8, state: *mut u64, block_count: usize);
    fn hash_finalize(state: *const u64, total_len: usize) -> u64;
    fn memcpy_fast_avx2(dest: *mut u8, src: *const u8, count: usize) -> *mut u8;
    fn memcpy_fast_sse(dest: *mut u8, src: *const u8, count: usize) -> *mut u8;
    fn scan_for_uasset(buffer: *const u8, size: usize) -> i64;
    fn scan_for_magic(buffer: *const u8, size: usize, magic: u32) -> i64;
    fn count_nulls_simd(buffer: *const u8, size: usize) -> usize;
}

/// Check if ASM functions are available (linked at compile time)
pub fn asm_available() -> bool {
    cfg!(feature = "asm_hotpaths")
}

/// SIMD-accelerated hash state
#[repr(C, align(32))]
pub struct HashState {
    accumulators: [u64; 4],
    total_len: usize,
}

impl HashState {
    pub fn new(seed: u64) -> Self {
        const PRIME64_1: u64 = 0x9E3779B185EBCA87;
        const PRIME64_2: u64 = 0xC2B2AE3D27D4EB4F;
        
        Self {
            accumulators: [
                seed.wrapping_add(PRIME64_1).wrapping_add(PRIME64_2),
                seed.wrapping_add(PRIME64_2),
                seed,
                seed.wrapping_sub(PRIME64_1),
            ],
            total_len: 0,
        }
    }

    /// Process 32-byte blocks using ASM SIMD (with fallback)
    pub fn update(&mut self, data: &[u8]) {
        let block_count = data.len() / 32;
        self.total_len += data.len();

        if block_count > 0 {
            #[cfg(all(target_arch = "x86_64", feature = "asm_hotpaths"))]
            unsafe {
                hash_block_simd(
                    data.as_ptr(),
                    self.accumulators.as_mut_ptr(),
                    block_count,
                );
            }

            #[cfg(not(all(target_arch = "x86_64", feature = "asm_hotpaths")))]
            {
                self.update_rust_fallback(data, block_count);
            }
        }
    }

    /// Pure Rust fallback for hash block processing
    fn update_rust_fallback(&mut self, data: &[u8], block_count: usize) {
        const PRIME64_1: u64 = 0x9E3779B185EBCA87;
        const PRIME64_2: u64 = 0xC2B2AE3D27D4EB4F;

        for i in 0..block_count {
            let offset = i * 32;
            
            for lane in 0..4 {
                let lane_offset = offset + lane * 8;
                let input = u64::from_le_bytes(
                    data[lane_offset..lane_offset + 8].try_into().unwrap()
                );
                
                self.accumulators[lane] = self.accumulators[lane]
                    .wrapping_add(input.wrapping_mul(PRIME64_2));
                self.accumulators[lane] = self.accumulators[lane].rotate_left(31);
                self.accumulators[lane] = self.accumulators[lane].wrapping_mul(PRIME64_1);
            }
        }
    }

    /// Finalize and get hash value
    pub fn finalize(&self) -> u64 {
        #[cfg(all(target_arch = "x86_64", feature = "asm_hotpaths"))]
        unsafe {
            return hash_finalize(self.accumulators.as_ptr(), self.total_len);
        }

        #[cfg(not(all(target_arch = "x86_64", feature = "asm_hotpaths")))]
        {
            self.finalize_rust_fallback()
        }
    }

    fn finalize_rust_fallback(&self) -> u64 {
        const PRIME64_3: u64 = 0x165667B19E3779F9;
        const PRIME64_4: u64 = 0x85EBCA77C2B2AE63;

        let mut hash = self.accumulators[0].rotate_left(1)
            .wrapping_add(self.accumulators[1].rotate_left(7))
            .wrapping_add(self.accumulators[2].rotate_left(12))
            .wrapping_add(self.accumulators[3].rotate_left(18));

        hash = hash.wrapping_add(self.total_len as u64);

        // Avalanche
        hash ^= hash >> 33;
        hash = hash.wrapping_mul(PRIME64_3);
        hash ^= hash >> 29;
        hash = hash.wrapping_mul(PRIME64_4);
        hash ^= hash >> 32;

        hash
    }
}

/// Fast memory copy using SIMD
pub fn fast_memcpy(dest: &mut [u8], src: &[u8]) -> usize {
    let len = dest.len().min(src.len());

    #[cfg(all(target_arch = "x86_64", feature = "asm_hotpaths"))]
    {
        if is_avx2_supported() && len >= 256 {
            unsafe {
                memcpy_fast_avx2(dest.as_mut_ptr(), src.as_ptr(), len);
            }
            return len;
        } else if len >= 128 {
            unsafe {
                memcpy_fast_sse(dest.as_mut_ptr(), src.as_ptr(), len);
            }
            return len;
        }
    }

    // Fallback to standard copy
    dest[..len].copy_from_slice(&src[..len]);
    len
}

/// Scan buffer for UAsset magic bytes
pub fn scan_uasset_magic(buffer: &[u8]) -> Option<usize> {
    #[cfg(all(target_arch = "x86_64", feature = "asm_hotpaths"))]
    unsafe {
        let result = scan_for_uasset(buffer.as_ptr(), buffer.len());
        if result >= 0 {
            return Some(result as usize);
        }
        return None;
    }

    #[cfg(not(all(target_arch = "x86_64", feature = "asm_hotpaths")))]
    {
        scan_uasset_magic_fallback(buffer)
    }
}

fn scan_uasset_magic_fallback(buffer: &[u8]) -> Option<usize> {
    const UASSET_MAGIC: [u8; 4] = [0xC1, 0x83, 0x2A, 0x9E];
    
    buffer.windows(4)
        .position(|window| window == UASSET_MAGIC)
}

/// Scan for arbitrary 4-byte magic value
pub fn scan_magic(buffer: &[u8], magic: u32) -> Option<usize> {
    #[cfg(all(target_arch = "x86_64", feature = "asm_hotpaths"))]
    unsafe {
        let result = scan_for_magic(buffer.as_ptr(), buffer.len(), magic);
        if result >= 0 {
            return Some(result as usize);
        }
        return None;
    }

    #[cfg(not(all(target_arch = "x86_64", feature = "asm_hotpaths")))]
    {
        let magic_bytes = magic.to_le_bytes();
        buffer.windows(4)
            .position(|window| window == magic_bytes)
    }
}

/// Count null bytes using SIMD
pub fn count_nulls(buffer: &[u8]) -> usize {
    #[cfg(all(target_arch = "x86_64", feature = "asm_hotpaths"))]
    unsafe {
        return count_nulls_simd(buffer.as_ptr(), buffer.len());
    }

    #[cfg(not(all(target_arch = "x86_64", feature = "asm_hotpaths")))]
    {
        buffer.iter().filter(|&&b| b == 0).count()
    }
}

/// Check AVX2 support at runtime
#[cfg(all(target_arch = "x86_64", feature = "asm_hotpaths"))]
fn is_avx2_supported() -> bool {
    #[cfg(target_feature = "avx2")]
    {
        true
    }
    #[cfg(not(target_feature = "avx2"))]
    {
        // Runtime check using CPUID
        is_x86_feature_detected!("avx2")
    }
}

#[cfg(not(target_arch = "x86_64"))]
fn is_avx2_supported() -> bool {
    false
}

/// RDTSC-based high-precision timing
#[cfg(target_arch = "x86_64")]
pub fn rdtsc() -> u64 {
    unsafe {
        let lo: u32;
        let hi: u32;
        asm!(
            "rdtsc",
            out("eax") lo,
            out("edx") hi,
            options(nostack, nomem)
        );
        ((hi as u64) << 32) | (lo as u64)
    }
}

#[cfg(not(target_arch = "x86_64"))]
pub fn rdtsc() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_state() {
        let mut state = HashState::new(0);
        let data = vec![0u8; 64];
        state.update(&data);
        let hash = state.finalize();
        assert_ne!(hash, 0);
    }

    #[test]
    fn test_scan_magic_fallback() {
        let buffer = [0u8, 0, 0xC1, 0x83, 0x2A, 0x9E, 0, 0];
        let result = scan_uasset_magic_fallback(&buffer);
        assert_eq!(result, Some(2));
    }

    #[test]
    fn test_count_nulls() {
        let buffer = [0u8, 1, 0, 2, 0, 3, 0, 0];
        let count = count_nulls(&buffer);
        assert_eq!(count, 5);
    }
}
