// Shared frontend types. They mirror the Rust structs in src-tauri, which are
// serialized via serde into camelCase.

/** Where a version was obtained from. */
export type SourceKind = "release" | "tag";

/** A tracked repository together with its current state. */
export interface Repo {
  id: number;
  owner: string;
  name: string;
  /** Latest version the tool fetched from GitHub (release or tag name). */
  latestVersion: string | null;
  /** Link to the release/tag — opened on click. */
  latestUrl: string | null;
  /** Where the version came from. */
  sourceKind: SourceKind | null;
  /** Has an unseen update (for the tray indicator). */
  hasUnseen: boolean;
  /** Unix time of the last successful check (seconds), or null. */
  lastCheckedAt: number | null;
  /** User-assigned tags for grouping. */
  tags: string[];
}

/** Full repository name "owner/name". */
export function repoFullName(r: Pick<Repo, "owner" | "name">): string {
  return `${r.owner}/${r.name}`;
}
