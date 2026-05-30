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
  echo "Tag $TAG not found - run 'npm run release:version' first." >&2
  exit 1
fi

# gh must be available and authenticated.
if ! command -v gh >/dev/null; then
  echo "GitHub CLI (gh) is not installed." >&2
  exit 1
fi

echo "Building .dmg for ${TAG}..."
# The .app is ad-hoc signed via bundle.macOS.signingIdentity = "-" in
# tauri.conf.json, which produces a valid signature and avoids the "app is
# damaged" error. Downloaders still clear the quarantine (see notes footer).
# CI=true keeps the dmg step non-interactive (no Finder layout window).
CI=true npm run tauri build -- --bundles dmg

DMG="$(ls -t src-tauri/target/release/bundle/dmg/*.dmg 2>/dev/null | head -1)"
if [ -z "$DMG" ]; then
  echo "No .dmg was produced." >&2
  exit 1
fi
echo "Built $DMG"

# Push the release commit and tag.
echo "Pushing commit and tag..."
git push
git push origin "$TAG"

# Deterministic codename for this tag — used in the release title and, via the
# JSON-context pipeline below, the generated notes.
cargo build --quiet --release --manifest-path tools/codename/Cargo.toml
CODENAME_BIN="tools/codename/target/release/codename-gen"
RELEASE_CODENAME="$("$CODENAME_BIN" "$TAG")"
TITLE="$TAG \"$RELEASE_CODENAME\""

# Build release notes for this tag from the changelog (git-cliff), grouped by
# commit type. Mirror the changelog pipeline: dump the JSON context, enrich each
# release with its version-derived codename, then render from that context so
# the notes carry the right codename. Fall back to GitHub's auto-notes if
# git-cliff produces nothing.
echo "Creating GitHub release ${TAG} (${RELEASE_CODENAME})..."
NOTES_FILE="$(mktemp)"
trap 'rm -f "$NOTES_FILE"' EXIT
git-cliff --current --tag "$TAG" -x 2>/dev/null \
  | node scripts/changelog-codenames.mjs "$CODENAME_BIN" \
  | git-cliff --from-context - --strip header > "$NOTES_FILE" 2>/dev/null || true

if [ -s "$NOTES_FILE" ]; then
  gh release create "$TAG" "$DMG" --title "$TITLE" --notes-file "$NOTES_FILE"
else
  gh release create "$TAG" "$DMG" --title "$TITLE" --generate-notes
fi

echo "Released $TAG."
