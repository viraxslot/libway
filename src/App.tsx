import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import {
  addRepo,
  checkNow,
  getAutostart,
  listRepos,
  markAllSeen,
  markSeen,
  removeRepo,
  setAutostart,
} from "./api";
import AddRepoForm from "./components/AddRepoForm";
import IntervalSettings from "./components/IntervalSettings";
import RepoList from "./components/RepoList";
import TokenSettings from "./components/TokenSettings";
import type { Repo } from "./types";

export default function App() {
  const [repos, setRepos] = useState<Repo[]>([]);
  const [checking, setChecking] = useState(false);
  const [autostart, setAutostartState] = useState(false);

  const reload = useCallback(async () => {
    setRepos(await listRepos());
  }, []);

  // Initial load + subscribe to backend-driven updates (scheduler runs).
  useEffect(() => {
    reload();
    getAutostart()
      .then(setAutostartState)
      .catch(() => {});
    const unlisten = listen("repos-updated", () => {
      reload();
    });
    return () => {
      unlisten.then((f) => f());
    };
  }, [reload]);

  async function handleAdd(fullName: string) {
    setRepos(await addRepo(fullName));
  }

  async function handleRemove(id: number) {
    setRepos(await removeRepo(id));
  }

  async function handleSeen(id: number) {
    await markSeen(id);
    reload();
  }

  async function handleCheckNow() {
    setChecking(true);
    try {
      setRepos(await checkNow());
    } finally {
      setChecking(false);
    }
  }

  async function handleMarkAll() {
    await markAllSeen();
    reload();
  }

  async function toggleAutostart() {
    const next = !autostart;
    await setAutostart(next);
    setAutostartState(next);
  }

  const anyUnseen = repos.some((r) => r.hasUnseen);

  return (
    <main className="app">
      <header className="app-header">
        <h1>libway</h1>
        <div className="header-actions">
          <button
            type="button"
            className="secondary"
            onClick={handleMarkAll}
            disabled={!anyUnseen}
          >
            Mark all as read
          </button>
          <button type="button" onClick={handleCheckNow} disabled={checking}>
            {checking ? "Checking…" : "Check now"}
          </button>
        </div>
      </header>

      <AddRepoForm onAdd={handleAdd} />
      <RepoList repos={repos} onRemove={handleRemove} onSeen={handleSeen} />

      <TokenSettings />
      <IntervalSettings />

      <section className="autostart">
        <label>
          <input
            type="checkbox"
            checked={autostart}
            onChange={toggleAutostart}
          />
          Launch at login
        </label>
      </section>
    </main>
  );
}
