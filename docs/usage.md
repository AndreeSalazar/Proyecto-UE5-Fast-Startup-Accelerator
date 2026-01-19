# UE5 Fast Startup Accelerator - Usage Guide

## Installation

### Prerequisites

1. **Rust 1.75+** - Install from https://rustup.rs
2. **NASM** - For ASM hot paths (optional but recommended)
3. **Unreal Engine 5.7.1+**
4. **Visual Studio 2022** - For UE5 plugin compilation

### Building the Rust CLI

```bash
cd rust-analyzer
cargo build --release
```

The executable will be at `target/release/ue5-fast-startup.exe`

### Installing the UE5 Plugin

1. Copy the `ue5-plugin` folder to your project's `Plugins/` directory
2. Rename it to `FastStartup`
3. Copy `ue5-fast-startup.exe` to `Plugins/FastStartup/Binaries/`
4. Regenerate project files (right-click .uproject → Generate Visual Studio project files)
5. Build your project

## Usage

### From the Editor

1. Open your UE5 project
2. Go to **Window → Fast Startup Accelerator**
3. The plugin window will open

#### Analyze Project
Click **"Analyze Project"** to scan your assets and dependencies. This will:
- Count all assets in Content/
- Identify startup-critical assets
- Build a dependency graph
- Calculate estimated savings

#### Build Cache
Click **"Build Cache"** to generate the startup cache:
- Hashes all asset content
- Computes optimal load order
- Saves to `Saved/FastStartup/startup.uefast`

#### Enable Fast Startup
Toggle **"Enable Fast Startup Mode"** to activate the optimization.

### From Command Line

The Rust CLI can be used independently:

```bash
# Analyze project
ue5-fast-startup analyze --project "C:/Projects/MyGame" --output analysis.json

# Build cache
ue5-fast-startup cache --project "C:/Projects/MyGame" --output startup.uefast

# Verify cache
ue5-fast-startup verify --cache startup.uefast --project "C:/Projects/MyGame"

# Show statistics
ue5-fast-startup stats --cache startup.uefast

# Build dependency graph
ue5-fast-startup graph --project "C:/Projects/MyGame" --output deps.dot

# Benchmark
ue5-fast-startup bench --project "C:/Projects/MyGame" --iterations 5
```

## Configuration

### Cache Location
Default: `<Project>/Saved/FastStartup/startup.uefast`

### CLI Location
The plugin looks for the CLI in:
1. `<Project>/Plugins/FastStartup/Binaries/ue5-fast-startup.exe`
2. `<Project>/Binaries/ue5-fast-startup.exe`

## Best Practices

### When to Rebuild Cache
- After adding/removing many assets
- After major content changes
- After engine updates
- Weekly for active development

### Optimal Workflow
1. Run analysis at project start
2. Build cache once stable
3. Enable fast startup for daily use
4. Rebuild cache periodically

## Troubleshooting

### Cache Not Found
- Ensure you've run "Build Cache" at least once
- Check `Saved/FastStartup/` directory exists

### CLI Not Found
- Verify `ue5-fast-startup.exe` is in the correct location
- Check file permissions

### Slow Analysis
- Large projects (10k+ assets) may take several minutes
- Consider running overnight for initial analysis

### Invalid Cache
- Cache may be invalidated by asset changes
- Run "Verify Cache" to check status
- Rebuild if necessary

## Performance Tips

1. **Use SSDs** - Significantly faster scanning
2. **Close other programs** - More CPU for parallel processing
3. **Exclude generated content** - Focus on source assets
4. **Regular maintenance** - Keep cache up to date
