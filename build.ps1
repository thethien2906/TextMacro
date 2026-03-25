<#
.SYNOPSIS
    Build TextMacro release .exe and package it into a Windows installer.

.DESCRIPTION
    1. Converts assets/logo.png to assets/logo.ico
    2. Builds the release binary via  cargo build --release
    3. Compiles the NSIS installer   installer/textmacro.nsi -> dist/TextMacro_Setup.exe

.REQUIREMENTS
    - Rust / cargo  (https://rustup.rs)
    - NSIS 3.x      (https://nsis.sourceforge.io/)

.USAGE
    .\build.ps1
    .\build.ps1 -SkipInstaller    # only build the .exe, skip NSIS step
#>

param (
    [switch]$SkipInstaller
)

$ErrorActionPreference = "Stop"
$Root = $PSScriptRoot

function Write-Step($msg) {
    Write-Host ""
    Write-Host "======================================" -ForegroundColor Cyan
    Write-Host "  $msg" -ForegroundColor Cyan
    Write-Host "======================================" -ForegroundColor Cyan
}

function Confirm-ExitCode($code, $label) {
    if ($code -ne 0) {
        Write-Host "FAILED: $label (exit $code)" -ForegroundColor Red
        exit $code
    }
    Write-Host "OK: $label" -ForegroundColor Green
}

# Ensure dist/ exists
$DistDir = Join-Path $Root "dist"
New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

# -----------------------------------------------
# Step 1 -- Convert PNG to ICO
# -----------------------------------------------
Write-Step "Step 1 -- Converting logo.png to logo.ico"

$PngPath = Join-Path $Root "assets\logo.png"
$IcoPath = Join-Path $Root "assets\logo.ico"

if (Test-Path $PngPath) {
    try {
        Add-Type -AssemblyName System.Drawing

        $sizes = @(16, 32, 48, 64, 128, 256)
        $ms = New-Object System.IO.MemoryStream
        $bw = New-Object System.IO.BinaryWriter($ms)

        # ICO header
        $bw.Write([uint16]0)
        $bw.Write([uint16]1)
        $bw.Write([uint16]$sizes.Count)

        $frames = @()
        $headerSize = 6 + 16 * $sizes.Count

        foreach ($size in $sizes) {
            $src = [System.Drawing.Image]::FromFile($PngPath)
            $bmp = New-Object System.Drawing.Bitmap($src, $size, $size)
            $src.Dispose()
            $pngMs = New-Object System.IO.MemoryStream
            $bmp.Save($pngMs, [System.Drawing.Imaging.ImageFormat]::Png)
            $bmp.Dispose()
            $frames += $pngMs
        }

        $offset = $headerSize
        for ($i = 0; $i -lt $sizes.Count; $i++) {
            $s = if ($sizes[$i] -eq 256) { 0 } else { $sizes[$i] }
            $bw.Write([byte]$s)
            $bw.Write([byte]$s)
            $bw.Write([byte]0)
            $bw.Write([byte]0)
            $bw.Write([uint16]1)
            $bw.Write([uint16]32)
            $bw.Write([uint32]$frames[$i].Length)
            $bw.Write([uint32]$offset)
            $offset += $frames[$i].Length
        }

        foreach ($frame in $frames) {
            $bw.Write($frame.ToArray())
            $frame.Dispose()
        }

        $bw.Flush()
        [System.IO.File]::WriteAllBytes($IcoPath, $ms.ToArray())
        $ms.Dispose()
        $bw.Dispose()

        Write-Host "  logo.ico created at: $IcoPath" -ForegroundColor Green
    } catch {
        Write-Host "  WARNING: ICO conversion failed: $_" -ForegroundColor Yellow
        Write-Host "  The build will continue without an embedded icon." -ForegroundColor Yellow
    }
} else {
    Write-Host "  WARNING: assets\logo.png not found -- skipping icon conversion" -ForegroundColor Yellow
}

# -----------------------------------------------
# Step 2 -- cargo build --release
# -----------------------------------------------
Write-Step "Step 2 -- Building release binary (cargo build --release)"

Push-Location $Root
cargo build --release
$cargoExit = $LASTEXITCODE
Pop-Location

Confirm-ExitCode $cargoExit "cargo build --release"

$ExePath = Join-Path $Root "target\release\textmacro.exe"
if (-not (Test-Path $ExePath)) {
    Write-Host "FAILED: Expected binary not found: $ExePath" -ForegroundColor Red
    exit 1
}

$sizeBytes = (Get-Item $ExePath).Length
$sizeMB = [math]::Round($sizeBytes / 1MB, 2)
Write-Host "  Binary: $ExePath  ($sizeMB MB)"

# Copy standalone exe to dist/
Copy-Item -Path $ExePath -Destination (Join-Path $DistDir "textmacro.exe") -Force
Write-Host "  Copied to dist\textmacro.exe" -ForegroundColor Green

# -----------------------------------------------
# Step 3 -- Build NSIS installer
# -----------------------------------------------
if ($SkipInstaller) {
    Write-Host ""
    Write-Host "INFO: -SkipInstaller flag set; skipping NSIS step." -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Done! Release exe -> dist\textmacro.exe" -ForegroundColor Green
    exit 0
}

Write-Step "Step 3 -- Compiling NSIS installer"

$makensis = $null
$candidates = @(
    "makensis",
    "C:\Program Files (x86)\NSIS\makensis.exe",
    "C:\Program Files\NSIS\makensis.exe"
)

foreach ($c in $candidates) {
    if (Get-Command $c -ErrorAction SilentlyContinue) {
        $makensis = $c
        break
    }
    if (($c -like "C:\*") -and (Test-Path $c)) {
        $makensis = $c
        break
    }
}

if (-not $makensis) {
    Write-Host ""
    Write-Host "FAILED: NSIS (makensis) not found." -ForegroundColor Red
    Write-Host "  Install NSIS from: https://nsis.sourceforge.io/Download" -ForegroundColor Yellow
    Write-Host "  After installing, re-run this script." -ForegroundColor Yellow
    Write-Host "  Or skip the installer step with:  .\build.ps1 -SkipInstaller" -ForegroundColor Yellow
    Write-Host ""
    exit 1
}

Write-Host "  Using makensis: $makensis"

$NsiScript = Join-Path $Root "installer\textmacro.nsi"
& $makensis $NsiScript
$nsisExit = $LASTEXITCODE
Confirm-ExitCode $nsisExit "NSIS compilation"

$SetupPath = Join-Path $DistDir "TextMacro_Setup.exe"
if (Test-Path $SetupPath) {
    $setupBytes = (Get-Item $SetupPath).Length
    $setupMB = [math]::Round($setupBytes / 1MB, 2)
    Write-Host ""
    Write-Host "======================================" -ForegroundColor Green
    Write-Host "  BUILD COMPLETE" -ForegroundColor Green
    Write-Host "======================================" -ForegroundColor Green
    Write-Host ""
    Write-Host "  Installer : dist\TextMacro_Setup.exe  ($setupMB MB)"
    Write-Host "  Binary    : dist\textmacro.exe        ($sizeMB MB)"
    Write-Host ""
} else {
    Write-Host "FAILED: Installer not found at: $SetupPath" -ForegroundColor Red
    exit 1
}
