#!/usr/bin/env bash
# Build a .dmg and publish a GitHub release for the current version tag.
# Run after `npm run release:version`. Usage: npm run release:publish
set -euo pipefail

VERSION="$(node -p "require('./package.json').version")"
TAG="v$VERSION"

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

echo "Building .dmg for $TAG…"
npm run tauri build -- --bundles dmg

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

# Create the GitHub release with auto-generated notes and the .dmg attached.
echo "Creating GitHub release $TAG…"
gh release create "$TAG" "$DMG" --title "$TAG" --generate-notes

echo "Released $TAG."
