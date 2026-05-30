#!/usr/bin/env bash
# Build a .dmg and publish a GitHub release for the current version tag.
# Run after `npm run release:version`. Usage: npm run release:publish
set -euo pipefail

VERSION="$(node -p "require('./package.json').version")"
TAG="v$VERSION"

# Fail early if the version isn't in sync across the three files (the binary
# bakes in Cargo.toml's version, which the tag/release should match).
bash scripts/check-versions.sh

# The tag must exist (created by release-version.sh).
if ! git rev-parse "$TAG" >/dev/null 2>&1; then
  echo "Tag $TAG not found — run 'npm run release:version' first." >&2
  exit 1
fi

# gh must be available and authenticated.
if ! command -v gh >/dev/null; then
  echo "GitHub CLI (gh) is not installed." >&2
  exit 1
fi

echo "Building .dmg for ${TAG}…"
# CI=true makes the DMG step non-interactive: it skips the AppleScript that
# opens a Finder window to lay out the drag-to-Applications view.
CI=true npm run tauri build -- --bundles dmg

DMG="$(ls -t src-tauri/target/release/bundle/dmg/*.dmg 2>/dev/null | head -1)"
if [ -z "$DMG" ]; then
  echo "No .dmg was produced." >&2
  exit 1
fi
echo "Built $DMG"

# Push the release commit and tag.
echo "Pushing commit and tag…"
git push
git push origin "$TAG"

# Build release notes from the changelog (git-cliff), grouped by commit type.
# Fall back to GitHub's auto-notes if git-cliff produces nothing.
echo "Creating GitHub release ${TAG}…"
NOTES_FILE="$(mktemp)"
trap 'rm -f "$NOTES_FILE"' EXIT
git-cliff --current --strip header --tag "$TAG" > "$NOTES_FILE" 2>/dev/null || true

if [ -s "$NOTES_FILE" ]; then
  gh release create "$TAG" "$DMG" --title "$TAG" --notes-file "$NOTES_FILE"
else
  gh release create "$TAG" "$DMG" --title "$TAG" --generate-notes
fi

echo "Released $TAG."
