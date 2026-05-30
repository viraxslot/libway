// Thin wrappers over Tauri's invoke() — the bridge to the Rust commands.
// Keeping them in one place gives the UI a typed, named API.

import { invoke } from "@tauri-apps/api/core";
import type { Repo } from "./types";

export function listRepos(): Promise<Repo[]> {
  return invoke<Repo[]>("list_repos");
}

/** Add a repo by "owner/name" (or a github.com URL). Returns the updated list. */
export function addRepo(fullName: string): Promise<Repo[]> {
  return invoke<Repo[]>("add_repo", { fullName });
}

export function removeRepo(id: number): Promise<Repo[]> {
  return invoke<Repo[]>("remove_repo", { id });
}

/** Replace a repo's tags. Returns the updated list. */
export function setRepoTags(id: number, tags: string[]): Promise<Repo[]> {
  return invoke<Repo[]>("set_repo_tags", { id, tags });
}

/** Rename a tag across all repos. Merges if the new name already exists. Returns the updated list. */
export function renameTag(from: string, to: string): Promise<Repo[]> {
  return invoke<Repo[]>("rename_tag", { from, to });
}

/** Remove a tag from all repos. Returns the updated list. */
export function deleteTag(tag: string): Promise<Repo[]> {
  return invoke<Repo[]>("delete_tag", { tag });
}

export function markSeen(id: number): Promise<void> {
  return invoke("mark_seen", { id });
}

export function markAllSeen(): Promise<void> {
  return invoke("mark_all_seen");
}

/** Trigger an immediate check of all repos. Returns the refreshed list. */
export function checkNow(): Promise<Repo[]> {
  return invoke<Repo[]>("check_now");
}

// --- Check interval ---

export function getCheckInterval(): Promise<number> {
  return invoke<number>("get_check_interval");
}

/** Set the check interval in minutes (>= 1). */
export function setCheckInterval(minutes: number): Promise<void> {
  return invoke("set_check_interval", { minutes });
}

export function getCheckOnStartup(): Promise<boolean> {
  return invoke<boolean>("get_check_on_startup");
}

export function setCheckOnStartup(enabled: boolean): Promise<void> {
  return invoke("set_check_on_startup", { enabled });
}

// --- GitHub token (stored in the Keychain) ---

export function hasToken(): Promise<boolean> {
  return invoke<boolean>("has_token");
}

/** Set the token; an empty string clears it. */
export function setToken(token: string): Promise<void> {
  return invoke("set_token", { token });
}

export function clearToken(): Promise<void> {
  return invoke("clear_token");
}

// --- Autostart ---

export function getAutostart(): Promise<boolean> {
  return invoke<boolean>("get_autostart");
}

export function setAutostart(enabled: boolean): Promise<void> {
  return invoke("set_autostart", { enabled });
}
