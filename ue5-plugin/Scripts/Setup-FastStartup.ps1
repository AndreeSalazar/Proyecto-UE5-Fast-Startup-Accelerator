# Fast Startup Accelerator - Setup Script
# Copyright 2026 Eddi Andreé Salazar Matos - Apache 2.0

param(
    [Parameter(Mandatory=$false)]
    [string]$ProjectPath,
    
    [Parameter(Mandatory=$false)]
    [switch]$BuildEngineCache,
    
    [Parameter(Mandatory=$false)]
    [switch]$Verbose
)

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$PluginDir = Split-Path -Parent $ScriptDir
$CLIPath = Join-Path $PluginDir "Binaries\ue5-fast-startup.exe"
$EngineDir = "C:\Program Files\Epic Games\UE_5.7\Engine"

Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Fast Startup Accelerator Setup" -ForegroundColor Cyan
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""

# Verify CLI exists
if (-not (Test-Path $CLIPath)) {
    Write-Host "ERROR: CLI not found at $CLIPath" -ForegroundColor Red
    exit 1
}

Write-Host "CLI found: $CLIPath" -ForegroundColor Green

# Build Engine cache if requested
if ($BuildEngineCache) {
    Write-Host ""
    Write-Host "Building Engine cache..." -ForegroundColor Yellow
    
    $EngineCacheDir = Join-Path $PluginDir "Cache"
    if (-not (Test-Path $EngineCacheDir)) {
        New-Item -ItemType Directory -Path $EngineCacheDir -Force | Out-Null
    }
    
    $Args = @("cache", "--project", $EngineDir, "--output", "$EngineCacheDir\engine.uefast")
    if ($Verbose) { $Args += "-v" }
    
    & $CLIPath $Args
    
    Write-Host "Engine cache built: $EngineCacheDir\engine.uefast" -ForegroundColor Green
}

# Build project cache if path provided
if ($ProjectPath) {
    if (-not (Test-Path $ProjectPath)) {
        Write-Host "ERROR: Project path not found: $ProjectPath" -ForegroundColor Red
        exit 1
    }
    
    Write-Host ""
    Write-Host "Building project cache for: $ProjectPath" -ForegroundColor Yellow
    
    $ProjectCacheDir = Join-Path $ProjectPath "Saved\FastStartup"
    if (-not (Test-Path $ProjectCacheDir)) {
        New-Item -ItemType Directory -Path $ProjectCacheDir -Force | Out-Null
    }
    
    # Analyze
    Write-Host "  Analyzing project..." -ForegroundColor Gray
    $Args = @("analyze", "--project", $ProjectPath, "--output", "$ProjectCacheDir\analysis.json")
    if ($Verbose) { $Args += "-v" }
    & $CLIPath $Args
    
    # Build cache
    Write-Host "  Building cache..." -ForegroundColor Gray
    $Args = @("cache", "--project", $ProjectPath, "--output", "$ProjectCacheDir\startup.uefast")
    if ($Verbose) { $Args += "-v" }
    & $CLIPath $Args
    
    Write-Host "Project cache built: $ProjectCacheDir\startup.uefast" -ForegroundColor Green
}

Write-Host ""
Write-Host "========================================" -ForegroundColor Cyan
Write-Host " Setup Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "The Fast Startup Accelerator plugin is now installed globally."
Write-Host "It will be available in all UE5 5.7 projects."
Write-Host ""
Write-Host "To access: Window -> Fast Startup Accelerator"
Write-Host ""
