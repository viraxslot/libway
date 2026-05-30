#!/usr/bin/env bash
# Move a tag to the current HEAD, locally and on the remote. Useful when a
# release tag was created a few commits early and needs to catch up.
#
# Usage:
#   bun run retag                 # moves the current version's tag (vX.Y.Z)
#   bun run retag v0.1.0          # moves the given tag
#
# Force-updates only the tag (never the branch). Push the branch separately.
set -euo pipefail

TAG="${1:-}"
if [ -z "$TAG" ]; then
  TAG="v$(bun -e "console.log(require('./package.json').version)")"
fi

HEAD_SHA="$(git rev-parse --short HEAD)"
echo "Moving tag ${TAG} to ${HEAD_SHA}..."

# Recreate the tag locally on HEAD.
git tag -f "$TAG"

# Force-update just this tag on the remote.
git push --force origin "refs/tags/$TAG"

echo "Tag $TAG now points at $HEAD_SHA (local and origin)."
