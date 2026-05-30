#!/usr/bin/env bash
# Bump the app version across package.json, tauri.conf.json and Cargo.toml,
# regenerate CHANGELOG.md, then commit and tag it. Usage:
#   bun run release:version patch | minor | major | X.Y.Z
set -euo pipefail

BUMP="${1:-}"
if [ -z "$BUMP" ]; then
  echo "Usage: bun run release:version <patch|minor|major|X.Y.Z>" >&2
  exit 1
fi

# Require a clean working tree so the release commit is just the bump.
if [ -n "$(git status --porcelain)" ]; then
  echo "Working tree is not clean - commit or stash changes first." >&2
  exit 1
fi

# Bump package.json (no git tag yet) and capture the resulting version.
bun pm version "$BUMP" --no-git-tag-version >/dev/null
VERSION="$(bun -e "console.log(require('./package.json').version)")"
echo "New version: $VERSION"

# Mirror the version into the Tauri config and the Rust crate.
bun -e "
  const fs = require('fs');
  const p = 'src-tauri/tauri.conf.json';
  const c = JSON.parse(fs.readFileSync(p, 'utf8'));
  c.version = '$VERSION';
  fs.writeFileSync(p, JSON.stringify(c, null, 2) + '\n');
"
# Cargo.toml: replace the version on the first 'version = \"...\"' line only.
perl -i -pe 'if (!$done && /^version = ".*"/) { s/^version = ".*"/version = "'"$VERSION"'"/; $done=1 }' src-tauri/Cargo.toml

# Refresh Cargo.lock so the new version is recorded.
cargo update -p libway --manifest-path src-tauri/Cargo.toml >/dev/null 2>&1 || true

# Regenerate the changelog, attributing unreleased commits to this version.
# --tag labels the not-yet-created tag so the new section is titled correctly.
# scripts/changelog.sh derives each release's codename from its own version.
bash scripts/changelog.sh --tag "v$VERSION"

git add package.json bun.lock src-tauri/tauri.conf.json src-tauri/Cargo.toml src-tauri/Cargo.lock CHANGELOG.md
git commit -m "chore(release): v$VERSION"
git tag "v$VERSION"

echo "Committed and tagged v$VERSION."
echo "Next: bun run release:publish"
