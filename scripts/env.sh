#!/usr/bin/env bash
# Source this file to initialize build environment
REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

export CARGO_HOME="${REPO_ROOT}/.cargo"
export RUSTUP_HOME="${REPO_ROOT}/.rustup"
export PATH="/home/theshriksloki/.local/node/bin:${REPO_ROOT}/.cargo/bin:${REPO_ROOT}/.bin:${PATH}"
export LD_LIBRARY_PATH="${REPO_ROOT}/.local-sys/usr/lib/x86_64-linux-gnu:/usr/lib/x86_64-linux-gnu:${LD_LIBRARY_PATH:-}"
export PKG_CONFIG_PATH="${REPO_ROOT}/.local-sys/usr/lib/x86_64-linux-gnu/pkgconfig:${REPO_ROOT}/.local-sys/usr/share/pkgconfig:/usr/lib/x86_64-linux-gnu/pkgconfig:/usr/share/pkgconfig:${PKG_CONFIG_PATH:-}"
export C_INCLUDE_PATH="${REPO_ROOT}/.local-sys/usr/include:${C_INCLUDE_PATH:-}"
export CPLUS_INCLUDE_PATH="${REPO_ROOT}/.local-sys/usr/include:${CPLUS_INCLUDE_PATH:-}"
export LIBRARY_PATH="${REPO_ROOT}/.local-sys/usr/lib/x86_64-linux-gnu:/usr/lib/x86_64-linux-gnu:${LIBRARY_PATH:-}"
export RUSTFLAGS="-L /usr/lib/x86_64-linux-gnu -L ${REPO_ROOT}/.local-sys/usr/lib/x86_64-linux-gnu ${RUSTFLAGS:-}"
