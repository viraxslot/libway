#!/usr/bin/env node
// Enrich a git-cliff JSON context (read from stdin) with a per-release codename
// and write it back to stdout. Each release's codename is derived from its own
// version via the codename-gen binary, so regenerating the whole changelog is
// deterministic and never overwrites an older release's name.
//
// Usage (in a pipeline):
//   git-cliff -x --tag vX.Y.Z | node scripts/changelog-codenames.mjs <codename-gen> | git-cliff --from-context -

import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";

const codenameBin = process.argv[2];
if (!codenameBin) {
  console.error("usage: changelog-codenames.mjs <path-to-codename-gen-binary>");
  process.exit(1);
}

const releases = JSON.parse(readFileSync(0, "utf8"));
for (const release of releases) {
  // Unreleased sections have a null version and get no codename.
  if (!release.version) continue;
  const codename = execFileSync(codenameBin, [release.version], {
    encoding: "utf8",
  }).trim();
  release.extra = { ...release.extra, codename };
}

process.stdout.write(JSON.stringify(releases));
