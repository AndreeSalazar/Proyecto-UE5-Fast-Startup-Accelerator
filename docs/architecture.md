# UE5 Fast Startup Accelerator - Architecture

## Overview

This document describes the technical architecture of the UE5 Fast Startup Accelerator plugin.

## System Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    UE5 Editor (5.7.1+)                      │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              FastStartup Plugin (C++)               │    │
│  │                                                     │    │
│  │  ┌──────────────┐  ┌──────────────┐  ┌───────────┐  │    │
│  │  │ FastStartup  │  │ FastStartup  │  │  Widget   │  │    │
│  │  │    Module    │  │    Core      │  │    UI     │  │    │
│  │  └──────┬───────┘  └──────┬───────┘  └─────┬─────┘  │    │
│  │         │                 │                │        │    │
│  └─────────┼─────────────────┼────────────────┼────────┘    │
│            │                 │                │             │
│            └────────┬────────┴────────────────┘             │
│                     │ subprocess                            │
│                     ▼                                       │
│  ┌─────────────────────────────────────────────────────┐    │
│  │           Rust CLI (ue5-fast-startup.exe)           │    │
│  │                                                     │    │
│  │  ┌──────────┐ ┌──────────┐ ┌──────────┐ ┌────────┐  │    │
│  │  │ Scanner  │ │  Graph   │ │  Cache   │ │ Hash   │  │    │
│  │  │          │ │ Builder  │ │ Builder  │ │ (ASM)  │  │    │
│  │  └──────────┘ └──────────┘ └──────────┘ └────────┘  │    │
│  │                                                     │    │
│  │  ┌──────────────────────────────────────────────┐   │    │
│  │  │            ASM Hot Paths (NASM)              │   │    │
│  │  │  • SIMD Hashing  • Fast Memcpy  • Scanning   │   │    │
│  │  └──────────────────────────────────────────────┘   │    │
│  └─────────────────────────────────────────────────────┘    │
│                     │                                       │
│                     ▼                                       │
│  ┌─────────────────────────────────────────────────────┐    │
│  │              Startup Cache (.uefast)                │    │
│  │                                                     │    │
│  │  • Asset manifest with hashes                       │    │
│  │  • Dependency graph                                 │    │
│  │  • Optimal load order                               │    │
│  │  • Shader variant info                              │    │
│  └─────────────────────────────────────────────────────┘    │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

## Components

### 1. UE5 Plugin (C++)

#### FastStartupCore Module
- **Loading Phase**: PreDefault
- **Purpose**: Core functionality loaded early in startup
- **Responsibilities**:
  - Cache validation
  - CLI path resolution
  - Enable/disable state management

#### FastStartup Module (Editor)
- **Loading Phase**: PostEngineInit
- **Purpose**: Editor UI and integration
- **Responsibilities**:
  - Menu/toolbar integration
  - Widget management
  - CLI invocation

### 2. Rust CLI

#### Scanner Module
- Parallel asset discovery using `rayon`
- File type detection
- Metadata extraction

#### Graph Module
- Dependency graph construction using `petgraph`
- Topological sorting for load order
- Startup-critical asset identification

#### Cache Module
- Binary cache format (.uefast)
- Incremental updates
- Verification

#### Hash Module
- xxHash3 for content hashing
- ASM acceleration for large files
- Memory-mapped I/O

### 3. ASM Hot Paths

#### hash_simd.asm
- SIMD-accelerated xxHash-style hashing
- AVX2/SSE4 support
- 32-byte block processing

#### memcpy_fast.asm
- AVX2 256-byte unrolled copy
- SSE2 fallback
- Aligned memory handling

#### scan_chunk.asm
- UAsset magic byte detection
- Parallel block scanning
- SIMD null counting

## Data Flow

### Analysis Flow
```
User clicks "Analyze" 
    → Plugin spawns CLI process
    → CLI scans Content/ directory
    → CLI parses UAsset headers
    → CLI builds dependency graph
    → CLI outputs analysis.json
    → Plugin reads results
```

### Cache Build Flow
```
User clicks "Build Cache"
    → Plugin spawns CLI process
    → CLI scans all assets
    → CLI hashes content (ASM accelerated)
    → CLI computes load order
    → CLI writes .uefast binary
    → Plugin validates cache
```

### Startup Flow (with cache)
```
Editor starts
    → FastStartupCore loads (PreDefault)
    → Core checks for valid .uefast
    → If valid: enable fast mode
    → Assets loaded in optimal order
    → Reduced startup time
```

## Cache Format

### Header (8 bytes)
```
UEFAST01
```

### Body (bincode serialized)
```rust
struct StartupCache {
    version: String,
    created_at: DateTime<Utc>,
    project_name: String,
    hash_algorithm: String,
    assets: Vec<CachedAsset>,
    load_order: Vec<String>,
    shader_variants: Vec<ShaderVariant>,
}
```

## Performance Considerations

### Parallel Processing
- Asset scanning uses `rayon` for parallel directory traversal
- Hashing uses parallel file processing
- Graph construction is parallelized where possible

### Memory Efficiency
- Memory-mapped I/O for large files
- Streaming hash computation
- Minimal allocations in hot paths

### ASM Optimization
- SIMD instructions for bulk operations
- Cache-friendly memory access patterns
- Minimal branch mispredictions

## Compatibility

### UE5 Versions
- Minimum: 5.3
- Tested: 5.7.1
- API: Uses stable editor APIs

### Platforms
- Windows (primary)
- Linux (planned)
- macOS (planned)

## Security

- No network access required
- All processing is local
- Cache files are project-specific
- No sensitive data stored
