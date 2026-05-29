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
