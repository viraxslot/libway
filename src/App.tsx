import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useState } from "react";
import {
  addRepo,
  checkNow,
  deleteTag,
  listRepos,
  markAllSeen,
  markSeen,
  removeRepo,
  renameTag,
  setRepoTags,
} from "@/api";
import Button from "@/components/ui/Button/Button";
import Tab from "@/components/ui/Tab/Tab";
import Tabs from "@/components/ui/Tabs/Tabs";
import RepositoriesTab from "@/features/repos/RepositoriesTab/RepositoriesTab";
import SettingsTab from "@/features/settings/SettingsTab/SettingsTab";
import TagsTab from "@/features/tags/TagsTab/TagsTab";
import type { Repo } from "@/types";

type TabId = "repositories" | "tags" | "settings";

export default function App() {
  const [repos, setRepos] = useState<Repo[]>([]);
  const [checking, setChecking] = useState(false);
  const [tab, setTab] = useState<TabId>("repositories");

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

  async function handleRenameTag(from: string, to: string) {
    setRepos(await renameTag(from, to));
  }

  async function handleDeleteTag(tag: string) {
    setRepos(await deleteTag(tag));
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
          <Button
            type="button"
            variant="secondary"
            onClick={handleMarkAll}
            disabled={!anyUnseen}
          >
            Mark all as read
          </Button>
          <Button type="button" onClick={handleCheckNow} disabled={checking}>
            {checking ? "Checking…" : "Check now"}
          </Button>
        </div>
      </header>

      <Tabs value={tab} onChange={(v) => setTab(v as TabId)}>
        <Tab value="repositories">Repositories</Tab>
        <Tab value="tags">Tags</Tab>
        <Tab value="settings">Settings</Tab>
      </Tabs>

      {tab === "repositories" && (
        <RepositoriesTab
          repos={repos}
          onAdd={handleAdd}
          onRemove={handleRemove}
          onSeen={handleSeen}
          onSetTags={handleSetTags}
        />
      )}
      {tab === "tags" && (
        <TagsTab
          repos={repos}
          onRenameTag={handleRenameTag}
          onDeleteTag={handleDeleteTag}
        />
      )}
      {tab === "settings" && <SettingsTab />}
    </main>
  );
}
