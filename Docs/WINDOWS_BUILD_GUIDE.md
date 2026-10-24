# OpenDeck — Native Windows 10/11 Build & Packaging Guide

This guide describes the complete, reproducible process to build, verify, and package **OpenDeck** on a native Windows 10 or Windows 11 64-bit environment without relying on external cloud services or network access during installation.

---

## 1. Prerequisites on Windows Host

Ensure the following toolchains are installed on the build machine:

1. **Microsoft Visual Studio 2022 C++ Build Tools:**
   - Workload: "Desktop development with C++"
   - Components: MSVC v143 x64/x86 build tools, Windows 10/11 SDK (10.0.22621+), CMake tools for Windows.
2. **Rust Toolchain:**
   - Architecture: `x86_64-pc-windows-msvc` (Rust 1.84+ or stable)
   - Install via `rustup-init.exe`: `rustup default stable-x86_64-pc-windows-msvc`
3. **Node.js & Package Manager:**
   - Node.js LTS v20.x or v22.x
   - Corepack enabled with `pnpm`: `npm install -g pnpm`
4. **Python 3.12 (Build Time Only):**
   - Python 3.12 64-bit with `pip`
   - Required build-time packages: `pyinstaller`, `rapidocr-onnxruntime`, `pymupdf`
   - *Note: Python is ONLY required to compile the standalone PyInstaller OCR sidecar binary. The final packaged OpenDeck installer requires NO Python on user workstations.*
5. **Packaging Tools (Optional for MSI/NSIS):**
   - NSIS 3.09+ (included in path) or WiX Toolset v3.14 (if building MSI).

---

## 2. Transferring the Repository & Model Assets

1. Clone or copy the OpenDeck repository:
   ```powershell
   git clone https://github.com/theshriks/OpenDeck.git C:\OpenDeck
   cd C:\OpenDeck
   ```
2. Verify model weights exist in `models\weights\`:
   ```powershell
   Get-FileHash models\weights\Qwen3-4B-Q4_K_M.gguf -Algorithm SHA256
   # Must match: 7485FE6F11AF29433BC51CAB58009521F205840F5B4AE3A32FA7F92E8534FDF5
   ```
3. Zero environment secrets or cloud API keys are required. OpenDeck is 100% offline.

---

## 3. Automated One-Click Build

Open **PowerShell (x64 Developer PowerShell for VS 2022)** as Administrator and run:

```powershell
cd C:\OpenDeck
.\scripts\windows_build_all.ps1
```

This automated script performs:
1. **Frontend Compilation:** Installs frozen dependencies and bundles React UI with Vite (`pnpm install --frozen-lockfile`, `pnpm run build`).
2. **Native llama.cpp Build:** Compiles `llama.cpp` using MSVC C++ with AVX2 CPU optimizations into `bin\llama\llama-cli.exe`.
3. **Standalone OCR Sidecar Packaging:** Packages `ocr_worker.py` into a single-file executable `workers\ocr\dist\ocr_worker.exe` using PyInstaller.
4. **Native Tauri Desktop Application:** Compiles the trusted Rust core (`cargo build --release`) and bundles the offline NSIS installer (`src-tauri\target\release\bundle\nsis\OpenDeck_0.1.0_x64-setup.exe`).
5. **Integrity Validation:** Verifies SHA-256 hashes of all generated artifacts.

---

## 4. Manual Step-by-Step Instructions

If you prefer building each component manually:

### Step 4.1: Compile Native llama.cpp for Windows
```powershell
cd C:\OpenDeck
git submodule update --init --recursive
cmake -B build-llama -S llama.cpp -DCMAKE_BUILD_TYPE=Release -DLLAMA_AVX2=ON -DLLAMA_BUILD_EXAMPLES=ON
cmake --build build-llama --config Release --target llama-cli
New-Item -ItemType Directory -Force -Path bin\llama
Copy-Item build-llama\bin\Release\llama-cli.exe bin\llama\llama-cli.exe
```

### Step 4.2: Build Standalone OCR Sidecar
```powershell
cd C:\OpenDeck\workers\ocr
python -m venv .venv
.\.venv\Scripts\Activate.ps1
pip install pyinstaller rapidocr-onnxruntime pymupdf
pyinstaller --onefile --name ocr_worker --collect-all rapidocr_onnxruntime ocr_worker.py
Copy-Item dist\ocr_worker.exe ..\..\bin\ocr\ocr_worker.exe
```

### Step 4.3: Build Frontend & Tauri App
```powershell
cd C:\OpenDeck
pnpm install --frozen-lockfile
pnpm run build
pnpm tauri build --target x86_64-pc-windows-msvc
```

---

## 5. Offline Installation Verification

1. Take the generated installer:
   `src-tauri\target\release\bundle\nsis\OpenDeck_0.1.0_x64-setup.exe`
2. Copy it to a clean test machine with **no internet connection** and **no developer tools** (no Rust, no Node, no Python).
3. Run the installer:
   - Follow prompts (Current User install, no admin privileges required).
   - Launch OpenDeck from Start Menu or Desktop shortcut.
4. Verify offline behavior:
   - Create workspace with a passphrase.
   - Import a test PDF invoice and statement CSV.
   - Run OCR extraction and check bounding box highlights.
   - Reconcile transactions and approve matches.
   - Query grounded chat assistant.
   - Export Source-Traceable PDF and CSV reports.
   - Generate an encrypted `.opendeck` backup and test restore.
