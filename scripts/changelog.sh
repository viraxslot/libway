#!/usr/bin/env bash
# Regenerate CHANGELOG.md, giving each release a deterministic codename derived
# from its own version. We dump git-cliff's JSON context, enrich every release
# with a codename via the codename-gen binary, then render from that context —
# so regenerating the whole changelog never overwrites an older release's name.
#
# Any arguments are forwarded to the first git-cliff invocation; release-version
# passes --tag vX.Y.Z to title the not-yet-created tag's section.
#
# Usage:
#   bash scripts/changelog.sh                 # regenerate from existing tags
#   bash scripts/changelog.sh --tag v1.2.3    # include an unreleased section as v1.2.3
set -euo pipefail

# Build the codename generator once; the pipeline calls it per release.
cargo build --quiet --release --manifest-path tools/codename/Cargo.toml
CODENAME_BIN="tools/codename/target/release/codename-gen"

git-cliff "$@" -x \
  | node scripts/changelog-codenames.mjs "$CODENAME_BIN" \
  | git-cliff --from-context - --output CHANGELOG.md

# Collapse the trailing blank line left by the inter-release spacing so the
# file ends with exactly one newline.
perl -i -0pe 's/\n+\z/\n/' CHANGELOG.md
