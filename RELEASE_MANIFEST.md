# SAAR Release Manifest & Software Bill of Materials (SBOM)

**Version:** 0.1.0  
**Target Release:** Production Desktop (Debian GNU/Linux 13 / Microsoft Windows 10/11 x64)  
**Security Classification:** Strict Offline / Air-Gapped Financial Intelligence Workspace  
**Author:** THE SHRIKS  
**License:** Proprietary / Air-Gapped Distribution  

---

## 1. Local Model Weights & Binary Runtimes

All artificial intelligence and OCR capabilities run 100% on the local device without network connectivity. Pre-downloaded weights and compiled engine binaries are pinned by exact cryptographic SHA-256 digests.

| Component | Path / Package | Parameters / Format | License | Cryptographic SHA-256 Digest |
|---|---|---|---|---|
| **Local LLM Engine** | `models/weights/Qwen2.5-3B-Instruct-Q3_K_M.gguf` | 3.09B params (Q3_K_M quantized GGUF) | Apache-2.0 | `8eff4e0eb51a8148abdaa9849f14f187e5ac6cd7d795610caf996a287277c59d` |
| **Inference Runner** | `bin/llama/llama-cli` (Debian Linux AVX2) | llama.cpp build b3948 native binary | MIT | `19796571e3423a7b43ee2e4e023203c738760ef0f156a9213ad5caa78bcc9c21` |
| **Inference Runner** | `bin/llama/llama-cli.exe` (Windows x64) | llama.cpp build b3948 Windows binary | MIT | Pinned in Windows release manifest |
| **OCR Text Engine** | `rapidocr-onnxruntime` | PP-OCRv4 ONNX (Detection, Rec, Cls) | Apache-2.0 | Bundled inside Python sidecar |
| **Digital PDF Parser**| `PyMuPDF` (fitz) | Digital PDF vector/text parser | AGPL-3.0 / Commercial | Bundled inside Python sidecar |

---

## 2. Desktop Core Dependencies (Rust Core — `src-tauri`)

Managed via `Cargo.lock` with strict offline vendoring/cache verification.

| Crate | Version | Upstream License | Purpose |
|---|---|---|---|
| `tauri` | 2.12.5 | MIT OR Apache-2.0 | Native desktop application framework |
| `tauri-build` | 2.7.4 | MIT OR Apache-2.0 | Native platform asset compiler & bundler |
| `rusqlite` | 0.33.0 | MIT | Embedded SQLite database engine with bundled C library |
| `argon2` | 0.5.3 | MIT OR Apache-2.0 | Memory-hard password-based key derivation (KDF) |
| `aes-gcm` | 0.10.3 | Apache-2.0 OR MIT | AES-256-GCM authenticated encryption for blobs and backups |
| `sha2` | 0.10.9 | MIT OR Apache-2.0 | Cryptographic SHA-256 hashing for content-addressed deduplication |
| `tokio` | 1.54.1 | MIT | Asynchronous runtime for sidecar process supervision |
| `serde` | 1.0.229 | MIT OR Apache-2.0 | High-performance serialization framework |
| `serde_json` | 1.0.154 | MIT OR Apache-2.0 | JSON parsing and serialization for IPC and database storage |
| `uuid` | 1.28.0 | Apache-2.0 OR MIT | RFC 4122 v4 unique identifier generation |
| `chrono` | 0.4.45 | MIT OR Apache-2.0 | ISO 8601 date, month, and timestamp formatting |
| `regex` | 1.13.1 | MIT OR Apache-2.0 | Log scrubbing, credential masking, and token parsing |
| `directories` | 6.0.0 | MIT OR Apache-2.0 | Standard platform-specific app data paths |
| `zeroize` | 1.9.1 | Apache-2.0 OR MIT | Memory clearing of cryptographic keys and passphrases |
| `rand` | 0.8.8 | MIT OR Apache-2.0 | Cryptographically secure pseudo-random number generator (OS CSPRNG) |
| `base64` | 0.23.1 | MIT OR Apache-2.0 | Secure in-memory base64 encoding for document preview |
| `hex` | 0.4.3 | MIT OR Apache-2.0 | Hexadecimal encoding for cryptographic hashes |
| `thiserror` | 2.0.21 | MIT OR Apache-2.0 | Type-safe error derivation |

---

## 3. Frontend Presentation Dependencies (React & TypeScript)

Managed via `pnpm-lock.yaml` with zero runtime CDNs, external scripts, or fonts.

| Package | Version | Upstream License | Purpose |
|---|---|---|---|
| `react` | 18.3.1 | MIT | Declarative user interface library |
| `react-dom` | 18.3.1 | MIT | DOM renderer for React |
| `@tauri-apps/api` | 2.12.4 | MIT OR Apache-2.0 | Tauri IPC bridge for invoking trusted Rust core commands |
| `lucide-react` | 0.475.0 | ISC | Local vector icons rendered offline |
| `typescript` | 5.7.3 | Apache-2.0 | Static type system for financial domain safety |
| `vite` | 6.4.5 | MIT | Fast offline bundler and asset compiler |

---

## 4. Integrity Verification Commands

To independently verify the authenticity of all local runtimes without internet access, run the following commands:

```bash
# 1. Verify Model Weight Integrity
sha256sum -c <<EOF
8eff4e0eb51a8148abdaa9849f14f187e5ac6cd7d795610caf996a287277c59d  models/weights/Qwen2.5-3B-Instruct-Q3_K_M.gguf
19796571e3423a7b43ee2e4e023203c738760ef0f156a9213ad5caa78bcc9c21  bin/llama/llama-cli
EOF

# 2. Run Comprehensive Offline Egress & Backup Verification Test Suite
source scripts/env.sh
unshare -rn bash -c "cd src-tauri && cargo test --test p6_reliability_and_backup_verification"
```
