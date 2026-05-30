import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import {
  addRepo,
  checkNow,
  listRepos,
  markAllSeen,
  markSeen,
  removeRepo,
  setRepoTags,
} from "./api";
import RepositoriesTab from "./components/RepositoriesTab";
import SettingsTab from "./components/SettingsTab";
import type { Repo } from "./types";

type Tab = "repositories" | "settings";

export default function App() {
  const [repos, setRepos] = useState<Repo[]>([]);
  const [checking, setChecking] = useState(false);
  const [tab, setTab] = useState<Tab>("repositories");

  const reload = useCallback(async () => {
    setRepos(await listRepos());
  }, []);

  // Initial load + subscribe to backend-driven updates (scheduler runs).
  useEffect(() => {
    reload();
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

  async function handleSetTags(id: number, tags: string[]) {
    setRepos(await setRepoTags(id, tags));
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

      <nav className="tabs">
        <button
          type="button"
          className={tab === "repositories" ? "tab active" : "tab"}
          onClick={() => setTab("repositories")}
        >
          Repositories
        </button>
        <button
          type="button"
          className={tab === "settings" ? "tab active" : "tab"}
          onClick={() => setTab("settings")}
        >
          Settings
        </button>
      </nav>

      {tab === "repositories" ? (
        <RepositoriesTab
          repos={repos}
          onAdd={handleAdd}
          onRemove={handleRemove}
          onSeen={handleSeen}
          onSetTags={handleSetTags}
        />
      ) : (
        <SettingsTab />
      )}
    </main>
  );
}
