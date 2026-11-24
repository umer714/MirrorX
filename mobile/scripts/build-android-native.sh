#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUTPUT_DIR="$ROOT_DIR/mobile/modules/mirrorx-capture/android/src/main/jniLibs"
MANIFEST="$ROOT_DIR/crates/mirrorx-media/Cargo.toml"

command -v cargo >/dev/null 2>&1 || { echo "Rust toolchain is required." >&2; exit 1; }
cargo ndk --version >/dev/null 2>&1 || {
  echo "cargo-ndk is required. Install it with: cargo install cargo-ndk" >&2
  exit 1
}

mkdir -p "$OUTPUT_DIR"
# cargo-ndk runs cargo metadata from its working directory; this repository has
# no top-level Cargo workspace, so enter the standalone media crate first.
cd "$ROOT_DIR/crates/mirrorx-media"
cargo ndk \
  -t arm64-v8a \
  -t armeabi-v7a \
  -t x86_64 \
  -p 24 \
  -o "$OUTPUT_DIR" \
  build --release
