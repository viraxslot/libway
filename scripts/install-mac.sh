#!/usr/bin/env bash
# Build the release .app and install it into /Applications so Spotlight and
# autostart can find it. Run via `npm run install:mac`.
set -euo pipefail

APP_NAME="libway.app"
SRC="src-tauri/target/release/bundle/macos/${APP_NAME}"
DEST="/Applications/${APP_NAME}"

echo "Building release bundle…"
npm run tauri build

if [ ! -d "$SRC" ]; then
  echo "Build did not produce $SRC" >&2
  exit 1
fi

# Quit a running instance so we can replace it cleanly.
pkill -f "${APP_NAME}/Contents/MacOS/libway" 2>/dev/null || true
sleep 1

echo "Installing to ${DEST}…"
rm -rf "$DEST"
cp -R "$SRC" "$DEST"

# Register with Launch Services so Spotlight picks it up promptly.
LSREGISTER="/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"
[ -x "$LSREGISTER" ] && "$LSREGISTER" -f "$DEST" || true

echo "Installed ${DEST}"
echo "Launch it from Spotlight (⌘Space → libway) or: open \"$DEST\""
