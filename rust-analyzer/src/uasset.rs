//! UAsset Parser Module
//! Copyright 2026 Eddi Andreé Salazar Matos
//! Licensed under Apache 2.0
//!
//! Minimal UAsset parsing for dependency extraction

use crate::{FastStartupError, Result};
use memmap2::Mmap;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::Path;

const UASSET_MAGIC: u32 = 0x9E2A83C1;

#[derive(Debug, Clone)]
pub struct UAssetHeader {
    pub magic: u32,
    pub legacy_version: i32,
    pub legacy_ue3_version: i32,
    pub file_version_ue4: i32,
    pub file_version_ue5: i32,
    pub file_version_licensee_ue4: i32,
    pub total_header_size: i32,
    pub package_name: String,
    pub package_flags: u32,
    pub name_count: i32,
    pub name_offset: i32,
    pub import_count: i32,
    pub import_offset: i32,
    pub export_count: i32,
    pub export_offset: i32,
}

pub struct UAssetParser;

impl UAssetParser {
    pub fn parse_header(path: &Path) -> Result<UAssetHeader> {
        let mut file = File::open(path)?;
        let mut buffer = [0u8; 4];

        // Read magic
        file.read_exact(&mut buffer)?;
        let magic = u32::from_le_bytes(buffer);

        if magic != UASSET_MAGIC {
            return Err(FastStartupError::AssetError(
                format!("Invalid UAsset magic: {:08X}", magic)
            ));
        }

        // Read legacy version
        file.read_exact(&mut buffer)?;
        let legacy_version = i32::from_le_bytes(buffer);

        // Read legacy UE3 version
        file.read_exact(&mut buffer)?;
        let legacy_ue3_version = i32::from_le_bytes(buffer);

        // Read UE4 file version
        file.read_exact(&mut buffer)?;
        let file_version_ue4 = i32::from_le_bytes(buffer);

        // Read UE5 file version (if applicable)
        file.read_exact(&mut buffer)?;
        let file_version_ue5 = i32::from_le_bytes(buffer);

        // Read licensee version
        file.read_exact(&mut buffer)?;
        let file_version_licensee_ue4 = i32::from_le_bytes(buffer);

        // Skip custom versions
        file.read_exact(&mut buffer)?;
        let custom_version_count = i32::from_le_bytes(buffer);
        
        // Each custom version is 20 bytes (GUID + version)
        file.seek(SeekFrom::Current(custom_version_count as i64 * 20))?;

        // Read total header size
        file.read_exact(&mut buffer)?;
        let total_header_size = i32::from_le_bytes(buffer);

        // Read package name (FString)
        let package_name = Self::read_fstring(&mut file)?;

        // Read package flags
        file.read_exact(&mut buffer)?;
        let package_flags = u32::from_le_bytes(buffer);

        // Read name count and offset
        file.read_exact(&mut buffer)?;
        let name_count = i32::from_le_bytes(buffer);
        file.read_exact(&mut buffer)?;
        let name_offset = i32::from_le_bytes(buffer);

        // Skip some fields to get to imports
        file.seek(SeekFrom::Current(16))?; // Skip gatherable text data

        // Read export count and offset
        file.read_exact(&mut buffer)?;
        let export_count = i32::from_le_bytes(buffer);
        file.read_exact(&mut buffer)?;
        let export_offset = i32::from_le_bytes(buffer);

        // Read import count and offset
        file.read_exact(&mut buffer)?;
        let import_count = i32::from_le_bytes(buffer);
        file.read_exact(&mut buffer)?;
        let import_offset = i32::from_le_bytes(buffer);

        Ok(UAssetHeader {
            magic,
            legacy_version,
            legacy_ue3_version,
            file_version_ue4,
            file_version_ue5,
            file_version_licensee_ue4,
            total_header_size,
            package_name,
            package_flags,
            name_count,
            name_offset,
            import_count,
            import_offset,
            export_count,
            export_offset,
        })
    }

    pub fn parse_imports(path: &Path) -> Result<Vec<String>> {
        let file = File::open(path)?;
        let mmap = unsafe { Mmap::map(&file)? };

        // Verify magic using ASM if available
        if mmap.len() < 4 {
            return Err(FastStartupError::AssetError("File too small".to_string()));
        }

        let magic = u32::from_le_bytes([mmap[0], mmap[1], mmap[2], mmap[3]]);
        if magic != UASSET_MAGIC {
            return Err(FastStartupError::AssetError(
                format!("Invalid UAsset magic: {:08X}", magic)
            ));
        }

        // Parse header to get import offset
        let header = Self::parse_header(path)?;

        if header.import_count <= 0 || header.import_offset <= 0 {
            return Ok(Vec::new());
        }

        // Read name table first
        let names = Self::read_name_table(&mmap, &header)?;

        // Read imports
        let mut imports = Vec::new();
        let mut offset = header.import_offset as usize;

        for _ in 0..header.import_count {
            if offset + 28 > mmap.len() {
                break;
            }

            // Import structure:
            // - ClassPackage (FName index) - 8 bytes
            // - ClassName (FName index) - 8 bytes
            // - OuterIndex - 4 bytes
            // - ObjectName (FName index) - 8 bytes

            let class_package_idx = i32::from_le_bytes([
                mmap[offset], mmap[offset + 1], mmap[offset + 2], mmap[offset + 3]
            ]) as usize;

            let _object_name_idx = i32::from_le_bytes([
                mmap[offset + 20], mmap[offset + 21], mmap[offset + 22], mmap[offset + 23]
            ]) as usize;

            if class_package_idx < names.len() {
                let package_name = &names[class_package_idx];
                if package_name.starts_with("/Game/") || package_name.starts_with("/Engine/") {
                    imports.push(package_name.clone());
                }
            }

            offset += 28;
        }

        Ok(imports)
    }

    fn read_name_table(mmap: &Mmap, header: &UAssetHeader) -> Result<Vec<String>> {
        let mut names = Vec::with_capacity(header.name_count as usize);
        let mut offset = header.name_offset as usize;

        for _ in 0..header.name_count {
            if offset + 4 > mmap.len() {
                break;
            }

            // Read string length
            let len = i32::from_le_bytes([
                mmap[offset], mmap[offset + 1], mmap[offset + 2], mmap[offset + 3]
            ]);
            offset += 4;

            if len <= 0 {
                names.push(String::new());
                continue;
            }

            let str_len = len.abs() as usize;
            if offset + str_len > mmap.len() {
                break;
            }

            // Read string (null-terminated)
            let name = if len > 0 {
                // ASCII/UTF-8
                String::from_utf8_lossy(&mmap[offset..offset + str_len - 1]).to_string()
            } else {
                // UTF-16
                let utf16: Vec<u16> = mmap[offset..offset + str_len]
                    .chunks(2)
                    .map(|c| u16::from_le_bytes([c[0], c.get(1).copied().unwrap_or(0)]))
                    .collect();
                String::from_utf16_lossy(&utf16)
            };

            names.push(name);
            offset += str_len;

            // Skip hash
            if offset + 4 <= mmap.len() {
                offset += 4;
            }
        }

        Ok(names)
    }

    fn read_fstring(file: &mut File) -> Result<String> {
        let mut buffer = [0u8; 4];
        file.read_exact(&mut buffer)?;
        let len = i32::from_le_bytes(buffer);

        if len == 0 {
            return Ok(String::new());
        }

        let str_len = len.abs() as usize;
        let mut str_buffer = vec![0u8; str_len];
        file.read_exact(&mut str_buffer)?;

        if len > 0 {
            // ASCII/UTF-8 (null-terminated)
            Ok(String::from_utf8_lossy(&str_buffer[..str_len.saturating_sub(1)]).to_string())
        } else {
            // UTF-16
            let utf16: Vec<u16> = str_buffer
                .chunks(2)
                .map(|c| u16::from_le_bytes([c[0], c.get(1).copied().unwrap_or(0)]))
                .collect();
            Ok(String::from_utf16_lossy(&utf16))
        }
    }

    pub fn is_valid_uasset(path: &Path) -> bool {
        if let Ok(file) = File::open(path) {
            if let Ok(mmap) = unsafe { Mmap::map(&file) } {
                if mmap.len() >= 4 {
                    let magic = u32::from_le_bytes([mmap[0], mmap[1], mmap[2], mmap[3]]);
                    return magic == UASSET_MAGIC;
                }
            }
        }
        false
    }

    pub fn get_ue_version(path: &Path) -> Result<(i32, i32)> {
        let header = Self::parse_header(path)?;
        Ok((header.file_version_ue4, header.file_version_ue5))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_uasset_magic() {
        assert_eq!(UASSET_MAGIC, 0x9E2A83C1);
    }
}
