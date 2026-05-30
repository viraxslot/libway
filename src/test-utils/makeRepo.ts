import type { Repo } from "@/types";

/**
 * Build a Repo for tests. Defaults describe a repo with a known release; pass
 * overrides for the fields a given test cares about (e.g. `{ latestVersion: null }`
 * for a not-yet-checked repo, or `{ hasUnseen: true }`).
 */
export function makeRepo(overrides: Partial<Repo> = {}): Repo {
  return {
    id: 1,
    owner: "owner",
    name: "repo",
    latestVersion: "1.0.0",
    latestUrl: "https://example.com",
    sourceKind: "release",
    hasUnseen: false,
    lastCheckedAt: null,
    tags: [],
    ...overrides,
  };
}
