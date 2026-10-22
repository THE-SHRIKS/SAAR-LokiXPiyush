<#
.SYNOPSIS
    OpenDeck Native Windows Build & Offline Packaging Pipeline
.DESCRIPTION
    Builds native Windows sidecars (llama-cli.exe, ocr_worker.exe), verifies model weights,
    builds the React frontend, and packages the complete offline NSIS desktop installer.
#>

[CmdletBinding()]
param (
    [string]$Target = "x86_64-pc-windows-msvc",
    [switch]$SkipSidecars = $false
)

$ErrorActionPreference = "Stop"
$ProgressPreference = "SilentlyContinue"

$RootDir = (Resolve-Path "$PSScriptRoot\..").Path
Set-Location $RootDir

Write-Host "===========================================================" -ForegroundColor Cyan
Write-Host "    OpenDeck Native Windows Packaging Pipeline (v0.1.0)    " -ForegroundColor Cyan
Write-Host "===========================================================" -ForegroundColor Cyan
Write-Host "Build Root: $RootDir"
Write-Host "Target Architecture: $Target"

# 1. Prerequisite Validation
Write-Host "`n[1/6] Checking required toolchains..." -ForegroundColor Yellow

function Assert-Tool($name, $command) {
    if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
        Write-Error "Required tool '$name' ($command) was not found in PATH."
    } else {
        Write-Host "  [OK] $name detected ($command)" -ForegroundColor Green
    }
}

Assert-Tool "Rust Compiler" "rustc"
Assert-Tool "Cargo Package Manager" "cargo"
Assert-Tool "Node.js Runtime" "node"
Assert-Tool "pnpm Package Manager" "pnpm"
Assert-Tool "CMake" "cmake"

# 2. Model Integrity Verification
Write-Host "`n[2/6] Verifying local model weights..." -ForegroundColor Yellow
$ModelPath = "$RootDir\models\weights\Qwen2.5-3B-Instruct-Q3_K_M.gguf"
if (-not (Test-Path $ModelPath)) {
    Write-Error "Model file missing: $ModelPath"
}
$ExpectedHash = "8EFF4E0EB51A8148ABDAA9849F14F187E5AC6CD7D795610CAF996A287277C59D"
$ActualHash = (Get-FileHash $ModelPath -Algorithm SHA256).Hash
if ($ActualHash -ne $ExpectedHash) {
    Write-Error "Model hash mismatch! Expected: $ExpectedHash, Got: $ActualHash"
}
Write-Host "  [PASS] Model weight SHA-256 integrity verified." -ForegroundColor Green

# 3. Build llama.cpp Native Windows Binary
if (-not $SkipSidecars) {
    Write-Host "`n[3/6] Building native Windows llama.cpp runner (AVX2 CPU)..." -ForegroundColor Yellow
    $LlamaBinDir = "$RootDir\bin\llama"
    New-Item -ItemType Directory -Force -Path $LlamaBinDir | Out-Null
    $LlamaCliExe = "$LlamaBinDir\llama-cli.exe"

    if (-not (Test-Path $LlamaCliExe)) {
        Write-Host "  Compiling llama.cpp via CMake + MSVC..."
        $LlamaBuildDir = "$RootDir\build-llama-win"
        New-Item -ItemType Directory -Force -Path $LlamaBuildDir | Out-Null
        
        cmake -B $LlamaBuildDir -S "$RootDir\llama.cpp" -DCMAKE_BUILD_TYPE=Release -DLLAMA_AVX2=ON -DLLAMA_BUILD_EXAMPLES=ON
        cmake --build $LlamaBuildDir --config Release --target llama-cli
        Copy-Item "$LlamaBuildDir\bin\Release\llama-cli.exe" $LlamaCliExe -Force
    }
    Write-Host "  [PASS] Native Windows LLM binary ready: $LlamaCliExe" -ForegroundColor Green

    # 4. Build Standalone OCR Sidecar (PyInstaller)
    Write-Host "`n[4/6] Building standalone Windows OCR worker..." -ForegroundColor Yellow
    $OcrBinDir = "$RootDir\workers\ocr\dist"
    New-Item -ItemType Directory -Force -Path $OcrBinDir | Out-Null
    $OcrExe = "$OcrBinDir\ocr_worker.exe"

    if (-not (Test-Path $OcrExe)) {
        Write-Host "  Packaging RapidOCR with PyInstaller..."
        Push-Location "$RootDir\workers\ocr"
        python -m pip install --quiet pyinstaller rapidocr-onnxruntime pymupdf
        pyinstaller --onefile --noconsole --name ocr_worker --collect-all rapidocr_onnxruntime ocr_worker.py
        Pop-Location
    }
    Write-Host "  [PASS] Standalone OCR executable ready: $OcrExe" -ForegroundColor Green
}

# 5. Build React + TypeScript Frontend
Write-Host "`n[5/6] Building frontend assets with Vite..." -ForegroundColor Yellow
pnpm install --frozen-lockfile
pnpm run build
Write-Host "  [PASS] Frontend built to dist/" -ForegroundColor Green

# 6. Build Tauri Desktop Shell & NSIS Offline Installer
Write-Host "`n[6/6] Packaging native Tauri offline desktop installer..." -ForegroundColor Yellow
pnpm tauri build --target $Target

$NsisDir = "$RootDir\src-tauri\target\$Target\release\bundle\nsis"
$Installer = Get-ChildItem "$NsisDir\*.exe" | Select-Object -First 1

if ($Installer) {
    $SizeMb = [math]::Round($Installer.Length / 1MB, 2)
    $InstallerHash = (Get-FileHash $Installer.FullName -Algorithm SHA256).Hash

    Write-Host "`n===========================================================" -ForegroundColor Green
    Write-Host "  OFFLINE WINDOWS INSTALLER SUCCESSFULLY PACKAGED" -ForegroundColor Green
    Write-Host "===========================================================" -ForegroundColor Green
    Write-Host "Installer: $($Installer.FullName)"
    Write-Host "Size:      $SizeMb MB"
    Write-Host "SHA-256:   $InstallerHash"

    $Summary = @{
        product = "OpenDeck"
        version = "0.1.0"
        target = $Target
        installerPath = $Installer.FullName
        sizeBytes = $Installer.Length
        sizeMb = $SizeMb
        sha256 = $InstallerHash
        timestamp = (Get-Date -Format "o")
    }
    $Summary | ConvertTo-Json -Depth 4 | Set-Content "$RootDir\windows_build_summary.json"
} else {
    Write-Warning "Tauri executable built, NSIS installer not found in $NsisDir"
}
