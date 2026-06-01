#!/usr/bin/env bash
# Build a .dmg and publish a GitHub release for the current version tag.
# Run after `bun run release:version`. Usage: bun run release:publish
set -euo pipefail

VERSION="$(bun -e "console.log(require('./package.json').version)")"
TAG="v$VERSION"

# Fail early if the version isn't in sync across the three files (the binary
# bakes in Cargo.toml's version, which the tag/release should match).
bash scripts/check-versions.sh

# The tag is created near the end (after a successful build). Fail only if a
# release for this tag already exists, to avoid clobbering it.
if gh release view "$TAG" >/dev/null 2>&1; then
  echo "Release $TAG already exists." >&2
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
CI=true bun run tauri build --bundles dmg

DMG="$(ls -t src-tauri/target/release/bundle/dmg/*.dmg 2>/dev/null | head -1)"
if [ -z "$DMG" ]; then
  echo "No .dmg was produced." >&2
  exit 1
fi
echo "Built $DMG"

# In local use, push the release commit. In CI the commit already reached main
# via the merged release PR, so SKIP_BRANCH_PUSH=1 skips it. The tag is pushed
# later, only after the build succeeds, to avoid a dangling tag.
if [ "${SKIP_BRANCH_PUSH:-}" != "1" ]; then
  echo "Pushing release commit..."
  git push
fi

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
  | bun scripts/changelog-codenames.mjs "$CODENAME_BIN" \
  | git-cliff --from-context - --strip header > "$NOTES_FILE" 2>/dev/null || true

# Build and notes succeeded — now create and push the tag, then the release.
# Doing this last means a failed build leaves no dangling tag.
if ! git rev-parse "$TAG" >/dev/null 2>&1; then
  git tag "$TAG"
fi
git push origin "$TAG"

if [ -s "$NOTES_FILE" ]; then
  gh release create "$TAG" "$DMG" --title "$TITLE" --notes-file "$NOTES_FILE"
else
  gh release create "$TAG" "$DMG" --title "$TITLE" --generate-notes
fi

echo "Released $TAG."
