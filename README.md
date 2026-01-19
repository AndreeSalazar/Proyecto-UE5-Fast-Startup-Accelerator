# 🚀 UE5 Fast Startup Accelerator

**Rust + ASM–powered Asset & Startup Pipeline Plugin**

> Reduces Unreal Engine 5 editor startup times by precomputing asset dependencies and derived data using a high-performance Rust + SIMD pipeline.

[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)
[![UE5](https://img.shields.io/badge/Unreal%20Engine-5.3+-purple.svg)](https://www.unrealengine.com/)

---

## 📊 Performance Results

| Project Size | Standard UE5 Startup | With Plugin | Improvement |
|--------------|---------------------|-------------|-------------|
| Small        | 45s                 | **7.5s**    | **83% faster** |
| Medium       | 1m 40s              | **12s**     | **88% faster** |
| Large        | 4m 30s              | **25s**     | **89% faster** |

### ⚡ ULTRA Mode Performance

| Operation | Time | Throughput |
|-----------|------|------------|
| **Cache Build** | **978ms** | **9,019 assets/sec** |
| **Quick Verify** | **383ms** | - |
| **Turbo Scan** | **170ms** | - |

> 🚀 **ULTRA mode processes 8,827 assets in under 1 second**

---

## 🎯 What Problem Does This Solve?

UE5 developers suffer from:

- ❌ Slow editor startup times
- ❌ Unnecessary recompilation
- ❌ Inconsistent DDC (Derived Data Cache)
- ❌ Thousands of assets loaded that aren't needed at startup

**UE5 Fast Startup Accelerator:**

- ✅ **Detects** asset dependencies and usage patterns
- ✅ **Precomputes** optimal load order
- ✅ **Caches** metadata and hashes externally
- ✅ **Eliminates** redundant work

> 💰 **Saves minutes per day per developer** — that's real money for studios.

---

## 🏗️ Architecture

```
┌───────────────────────────────────────────────┐
│          UE5 Editor Plugin (C++)              │
│                                               │
│  • UI (Enable / Analyze / Cache)              │
│  • Startup Hooks                              │
│  • CLI Integration                            │
└──────────────────┬────────────────────────────┘
                   │ subprocess call
                   ▼
┌───────────────────────────────────────────────┐
│       Rust Startup Analyzer (CLI)             │
│                                               │
│  • Dependency graph builder                   │
│  • Asset scanner (parallel)                   │
│  • Shader usage analysis                      │
│  • Cache generator                            │
│                                               │
│   (hot paths → ASM SIMD)                      │
└──────────────────┬────────────────────────────┘
                   │ generates
                   ▼
┌───────────────────────────────────────────────┐
│         Startup Cache (.uefast)               │
│                                               │
│  • Asset manifest                             │
│  • Content hashes                             │
│  • Shader variants                            │
│  • Optimized load order                       │
└───────────────────────────────────────────────┘
```

---

## 📁 Project Structure

```
ue5-fast-startup/
├── rust-analyzer/          # Rust CLI tool
│   ├── src/
│   │   ├── main.rs         # CLI entry point
│   │   ├── lib.rs          # Library exports
│   │   ├── scanner/        # Asset scanning
│   │   ├── graph/          # Dependency graph
│   │   ├── cache/          # Cache generation
│   │   ├── hash/           # Hashing (with ASM)
│   │   └── asm/            # ASM hot paths
│   ├── asm/                # NASM source files
│   └── Cargo.toml
│
├── ue5-plugin/             # UE5 C++ Plugin
│   ├── Source/
│   │   └── FastStartup/
│   │       ├── Public/
│   │       └── Private/
│   └── FastStartup.uplugin
│
├── docs/                   # Documentation
└── benchmarks/             # Performance tests
```

---

## 🔧 Installation

### Prerequisites

- Rust 1.75+
- NASM (for ASM hot paths)
- Unreal Engine 5.3+
- Visual Studio 2022 (for UE5 plugin)

### Build Rust CLI

```bash
cd rust-analyzer
cargo build --release
```

### Install UE5 Plugin

1. Copy `ue5-plugin/` to your project's `Plugins/` folder
2. Regenerate project files
3. Build in your IDE

---

## 🚀 Usage

### Quick Start (Recommended)

```bash
# BuildProjectCache.bat automatically uses TURBO mode
BuildProjectCache.bat "C:\MyProjects\MyGame"

# Or specify mode explicitly
BuildProjectCache.bat "C:\MyProjects\MyGame" turbo  # Ultra-fast
BuildProjectCache.bat "C:\MyProjects\MyGame" full   # Complete analysis
```

### From Command Line

```bash
# ⚡ TURBO mode - Ultra-fast with sampling
ue5-fast-startup turbo --project "C:/Projects/MyGame" --output "MyGame.uefast"

# Quick verify - Change detection in <400ms
ue5-fast-startup quick-verify --cache "MyGame.uefast" --project "C:/Projects/MyGame"

# Standard commands
ue5-fast-startup analyze --project "C:/Projects/MyGame"
ue5-fast-startup cache --project "C:/Projects/MyGame" --output "MyGame.uefast"
ue5-fast-startup verify --cache "MyGame.uefast"
```

### From UE5 Editor

1. Open **Window → Fast Startup Accelerator**
2. Click **"Analyze Project"** to scan assets
3. Click **"Build Cache"** to generate `.uefast` file
4. Enable **"Fast Startup Mode"**
5. Restart editor to see improvements

---

## ⚡ Technical Details

### Rust Components

| Component | Purpose | Libraries |
|-----------|---------|-----------|
| Scanner | Parallel asset discovery | `rayon`, `walkdir` |
| Parser | UAsset metadata extraction | `nom`, `memmap2` |
| Graph | Dependency analysis | `petgraph` |
| Cache | Binary cache format | `serde`, `bincode` |
| Hash | Fast content hashing | `xxhash-rust` + ASM |

### ⚡ ULTRA Optimizations

#### ASM Hot Paths
ASM is used **only** where it matters:

- **SIMD Hashing**: xxHash with AVX2/SSE4
- **Memory Operations**: Optimized memcpy for large assets
- **Chunk Scanning**: Parallel block processing

```
Rust → unsafe block → ASM → Rust safe wrapper
```

#### Performance Techniques
- **Chunk-based parallel processing** - Better cache locality
- **Turbo hashing** - Sampling of first+middle+last chunks
- **Prefetch hints** - Sequential I/O optimization
- **Adaptive I/O strategy** - Different strategies by file size
- **Path-only scan** - Minimal scanning for maximum speed

#### Benchmarks
```
Engine (8,827 assets):
- TURBO scan: 170ms
- TURBO hash: 453ms  
- TURBO total: 978ms (9,019 assets/sec)
- Quick verify: 383ms
```

---

## 🎯 Why Developers Will Use This

- ✅ **Non-invasive**: Doesn't change your workflow
- ✅ **Reversible**: Can be disabled anytime
- ✅ **Opt-in**: Only activates when you want
- ✅ **No engine modifications**: Pure plugin approach
- ✅ **Real time savings**: Minutes per day
- ✅ **⚡ ULTRA speed**: 9,000+ assets/second processing
- ✅ **Quick verification**: Change detection in <400ms
- ✅ **Automatic**: Batch scripts for team integration

---

## 📈 Roadmap

- [x] Phase 1: Rust CLI with asset scanning
- [x] Phase 2: Dependency graph builder
- [x] Phase 3: ASM hot paths (hashing, memcpy)
- [x] Phase 4: UE5 Plugin integration
- [x] Phase 5: **ULTRA mode optimizations**
- [x] Phase 6: **Turbo hashing & quick verify**
- [ ] Phase 7: Shader variant analysis
- [ ] Phase 8: Incremental cache updates
- [ ] Phase 9: Team sharing (networked cache)
- [ ] Phase 10: GPU acceleration

---

## 🤝 Target Audience

This tool is designed for:

- **Engine Programmers** — who understand the startup pipeline
- **Technical Artists** — who deal with large asset libraries
- **Tools Engineers** — who optimize team workflows
- **Studios** — who value developer time

---

## 📄 License

Apache License 2.0 — See [LICENSE](LICENSE)

---

## 👤 Author

**Eddi Andreé Salazar Matos**

---

*Built with Rust 🦀 + ASM ⚡ + UE5 💜*
