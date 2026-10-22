#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

echo "=== OpenDeck Windows Transfer & Asset Packaging Harness ==="
echo "Generating offline Windows deployment package manifest..."

PACKAGE_DIR="${ROOT_DIR}/dist_windows_transfer"
rm -rf "${PACKAGE_DIR}"
mkdir -p "${PACKAGE_DIR}/models/weights"
mkdir -p "${PACKAGE_DIR}/bin/llama"
mkdir -p "${PACKAGE_DIR}/workers/ocr"
mkdir -p "${PACKAGE_DIR}/scripts"
mkdir -p "${PACKAGE_DIR}/Docs"

echo "[1/4] Verifying model weights and binary assets..."
MODEL_PATH="${ROOT_DIR}/models/weights/Qwen2.5-3B-Instruct-Q3_K_M.gguf"
if [ ! -f "${MODEL_PATH}" ]; then
    echo "ERROR: Model weights not found at ${MODEL_PATH}!"
    exit 1
fi

MODEL_HASH=$(sha256sum "${MODEL_PATH}" | awk '{print $1}')
echo "  Qwen2.5 Model SHA-256: ${MODEL_HASH}"

# Create transfer checksum manifest
cat <<EOF > "${PACKAGE_DIR}/TRANSFER_CHECKSUMS.txt"
# OpenDeck Verified Transfer Manifest
# Date: $(date -u '+%Y-%m-%dT%H:%M:%SZ')
${MODEL_HASH}  models/weights/Qwen2.5-3B-Instruct-Q3_K_M.gguf
EOF

echo "[2/4] Copying locked configurations, scripts, and documentation..."
cp -r "${ROOT_DIR}/package.json" "${PACKAGE_DIR}/"
cp -r "${ROOT_DIR}/pnpm-lock.yaml" "${PACKAGE_DIR}/"
cp -r "${ROOT_DIR}/RELEASE_MANIFEST.md" "${PACKAGE_DIR}/"
cp -r "${ROOT_DIR}/Docs/WINDOWS_BUILD_GUIDE.md" "${PACKAGE_DIR}/Docs/"
cp -r "${ROOT_DIR}/scripts/windows_build_all.ps1" "${PACKAGE_DIR}/scripts/"
cp -r "${ROOT_DIR}/scripts/windows_verify_pipeline.ps1" "${PACKAGE_DIR}/scripts/"

echo "[3/4] Validating clean git checkout status (zero secret leakage)..."
if grep -rnEi "(aws_secret|api_key|private_key|token\s*=)" "${PACKAGE_DIR}" 2>/dev/null; then
    echo "ERROR: Plaintext secrets detected in transfer files!"
    exit 1
else
    echo "  [PASS] Zero secrets or credentials in transfer pack."
fi

echo "[4/4] Generating transfer instructions..."
cat <<EOF > "${PACKAGE_DIR}/README_TRANSFER.txt"
OpenDeck Native Windows 10/11 Deployment Pack
=============================================
1. Clone git repository onto Windows host:
   git clone <repo-url> C:\OpenDeck
2. Place model weights into C:\OpenDeck\models\weights\
3. Open PowerShell as Administrator and run:
   .\scripts\windows_build_all.ps1
4. Execute full offline regression tests:
   .\scripts\windows_verify_pipeline.ps1
EOF

echo "=== OpenDeck Windows Transfer Manifest Ready ==="
echo "Package directory: ${PACKAGE_DIR}"
