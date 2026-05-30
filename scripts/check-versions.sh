#!/usr/bin/env bash
# Verify the version is in sync across package.json, tauri.conf.json and
# Cargo.toml. The tray's About menu reads the version from Cargo.toml
# (CARGO_PKG_VERSION), so a mismatch would show the wrong version.
set -euo pipefail

PKG="$(node -p "require('./package.json').version")"
CONF="$(node -p "require('./src-tauri/tauri.conf.json').version")"
CARGO="$(grep -m1 '^version = ' src-tauri/Cargo.toml | sed -E 's/^version = "(.*)"/\1/')"

if [ "$PKG" = "$CONF" ] && [ "$PKG" = "$CARGO" ]; then
  echo "Version in sync: $PKG"
  exit 0
fi

echo "Version mismatch:" >&2
echo "  package.json:    $PKG" >&2
echo "  tauri.conf.json: $CONF" >&2
echo "  Cargo.toml:      $CARGO" >&2
echo "Run 'npm run release:version -- <patch|minor|major|X.Y.Z>' to sync." >&2
exit 1
