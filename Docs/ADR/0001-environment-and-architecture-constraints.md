# ADR 0001: Debian Environment Inspection and Architecture Constraints

**Date:** 2026-10-22  
**Status:** Accepted  
**Author:** THE SHRIKS  
**Task ID:** P0-01  

---

## 1. Context and Purpose
Before any application code or runtime infrastructure is modified, `AGENTS.md` and `PLAN.md` (Phase 0) require a rigorous inspection of the Debian development host, runtime environment, hardware constraints, and native Windows target boundaries.

---

## 2. Environment Inspection Log

### 2.1 Operating System & Kernel
- **OS Distribution:** Debian GNU/Linux 13 (trixie) x86_64 (version 13.7)
- **Kernel Version:** `Linux sandbox 6.12.107+deb13-amd64 #1 SMP PREEMPT_DYNAMIC Debian 6.12.107-1 (2026-08-29) x86_64 GNU/Linux`
- **User / Shell:** `theshriksloki` (UID 1000, GID 1000), Bash `/bin/bash`
- **Desktop Environment:** XFCE 4.20 on X11 (`XDG_CURRENT_DESKTOP=XFCE`, lightdm session)

### 2.2 CPU Architecture & Capabilities
- **Processor:** Intel(R) Core(TM) i3-7020U CPU @ 2.30GHz
- **Sockets / Cores / Threads:** 1 socket, 2 physical cores, 4 threads (2 threads per core, SMT enabled)
- **Instruction Set Extensions:** `fpu`, `vme`, `de`, `pse`, `tsc`, `msr`, `pae`, `cx8`, `apic`, `sep`, `mtrr`, `pge`, `cmov`, `pat`, `clflush`, `mmx`, `fxsr`, `sse`, `sse2`, `ss`, `ht`, `syscall`, `nx`, `lm`, `pni`, `pclmulqdq`, `ssse3`, `fma`, `cx16`, `sse4_1`, `sse4_2`, `movbe`, `popcnt`, `aes`, `xsave`, `avx`, `f16c`, `rdrand`, `bmi1`, `avx2`, `bmi2`, `erms`, `invpcid`, `rdseed`, `adx`, `smap`, `clflushopt`, `xsaveopt`
- **Notable Absences:** No AVX-512, no AMX.
- **Cache Topology:** L1d 64 KiB, L1i 64 KiB, L2 512 KiB, L3 3 MiB.

### 2.3 Memory & Swap
- **Physical RAM:** 11 GiB Total (`MemTotal: ~11,700,000 kB`)
- **Available RAM:** ~6.8 GiB (`free -h` shows 4.0 GiB completely free, 3.8 GiB buff/cache, 6.8 GiB available)
- **Swap Space:** 12 GiB configured (`12 GiB free`)

### 2.4 GPU & Hardware Acceleration
- **GPU Controller:** Intel Corporation HD Graphics 620 [8086:5921] (rev 06) integrated graphics
- **Discrete GPU:** NONE. No NVIDIA hardware present.
- **CUDA Status:** `nvidia-smi` not found. No CUDA drivers or runtime available.
- **Acceleration Support:** Intel VA-API / Mesa OpenGL / OpenCL possible; CPU execution is mandatory baseline.

### 2.5 Disk & Filesystem
- **Storage Device:** `/dev/sdb2` mounted on `/home/theshriksloki/OpenDeck` (ext4) with ~376 GB available space (14% utilization).
- **Filesystem Permissions:**
  - Workspace `/home/theshriksloki/OpenDeck`: Read/Write
  - User Cache `/home/theshriksloki/.cache`: Read/Write
  - System Paths (`/usr`, `/lib`, `/bin`, `/etc`): Read-Only mount overlay
  - Temp: `/tmp` mounted tmpfs (rw, noatime)

### 2.6 Available Toolchains & System Libraries
- **Node.js:** v24.19.0 (located in `~/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node`)
- **pnpm:** 11.25.0
- **Python:** Python 3.12.14 (located in `~/.cache/codex-runtimes/codex-primary-runtime/dependencies/python/bin/python3`)
- **pip:** 26.2.1
- **Installed Python Libraries:**
  - PDF manipulation & extraction: `pdfplumber 0.11.9`, `pypdf 6.10.0`, `pypdfium2 5.14.0`, `pdf2image 1.17.0`
  - Imaging & Data: `pillow 12.3.0`, `pandas 2.2.3`, `numpy 2.3.5`, `reportlab 4.4.9`
  - Cryptography & Typing: `cryptography 50.0.2`, `pydantic 2.14.1`
- **Native PDF Utilities:** Poppler 25.03.0 (`pdftoppm`, `pdfinfo`, etc.)
- **C/C++ Toolchain:** GCC 14.2.0 (`/usr/bin/gcc`), G++ 14.2.0 (`/usr/bin/g++`), GNU Make 4.3 (`/usr/bin/make`)
- **Installed System GUI Libraries:**
  - `libwebkit2gtk-4.1-0` (WebKitGTK 2.52.6)
  - `libgtk-3-0t64` (GTK 3.24.49)
  - `libsqlite3-0` (SQLite 3.46.1)
  - `libssl3t64` / `openssl` (OpenSSL 3.5.7)

---

## 3. Architecture Constraints & Invariants

1. **CPU-First Inference (No Discrete GPU):**
   With an Intel Core i3-7020U (2 cores / 4 threads, 3MB L3 cache) and no NVIDIA GPU, model execution will be CPU-bound.
   - Text LLM (Qwen3-4B candidate): Must use highly quantized GGUF (e.g. Q4_K_M or Q3_K_M / Q2_K if memory bandwidth is tight) compiled for AVX2 without CUDA. Thread count must be tuned to 2-3 threads (avoiding core thrashing on a 2-core / 4-thread CPU).
   - Ingestion / OCR: Born-digital documents must aggressively use native text-layer extraction (`pypdfium2` / `pdfplumber` / poppler) which executes in milliseconds and consumes near-zero RAM. Raster OCR (PP-OCRv6) should be invoked selectively on scanned pages and images.
2. **Memory Ceiling (11 GiB Total RAM):**
   The application footprint, sidecar processes, and model weights combined must never exceed ~4-5 GiB peak RAM to leave headroom for OS buffers and prevent swap thrashing on the 2-core CPU.
3. **No Network at Runtime (Local-Only Guarantee):**
   Per `RULES.md` and `PRD.md`, zero external egress is permitted during runtime. Development setup (package installs, model weight verification) is decoupled from runtime operation.
4. **Target Platform Compatibility:**
   - Debian 13 trixie serves as the local development and verification environment.
   - Windows 10/11 x64 is the primary delivery target. Architecture designs must not rely on Linux-only kernel features (e.g., must use cross-platform process isolation and sidecars supported on Windows Tauri 2).
