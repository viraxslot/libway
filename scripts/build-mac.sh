#!/usr/bin/env bash
# Build the release .app without installing it anywhere.
# Run via `bun run build:mac`. Use `bun run install:mac` to also copy it
# into /Applications.
set -euo pipefail

APP_NAME="libway.app"
SRC="src-tauri/target/release/bundle/macos/${APP_NAME}"

# The version is baked into the binary at compile time (CARGO_PKG_VERSION),
# so make sure all three files agree before building.
bash scripts/check-versions.sh

echo "Building release bundle..."
bun run tauri build

if [ ! -d "$SRC" ]; then
  echo "Build did not produce $SRC" >&2
  exit 1
fi

echo "Built $SRC"
echo "Run it with: open \"$SRC\""
