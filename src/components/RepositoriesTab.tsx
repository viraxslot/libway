import { useMemo, useState } from "react";
import { type Repo, repoFullName } from "../types";
import AddRepoForm from "./AddRepoForm";
import ConfirmDialog from "./ConfirmDialog";
import RepoList from "./RepoList";

interface Props {
  repos: Repo[];
  onAdd: (fullName: string) => Promise<void>;
  onRemove: (id: number) => void;
  onSeen: (id: number) => void;
}

/** Repositories tab: add form, search filter, and the tracked list. */
export default function RepositoriesTab({
  repos,
  onAdd,
  onRemove,
  onSeen,
}: Props) {
  const [query, setQuery] = useState("");
  // The repo pending deletion confirmation, or null.
  const [pendingDelete, setPendingDelete] = useState<Repo | null>(null);

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) return repos;
    return repos.filter((r) => repoFullName(r).toLowerCase().includes(q));
  }, [repos, query]);

  function confirmDelete() {
    if (pendingDelete) {
      onRemove(pendingDelete.id);
      setPendingDelete(null);
    }
  }

  return (
    <div className="tab-panel">
      <AddRepoForm onAdd={onAdd} />

      {repos.length > 0 && (
        <input
          type="search"
          className="search"
          placeholder="Search repositories…"
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          spellCheck={false}
          autoCapitalize="off"
        />
      )}

      {repos.length > 0 && filtered.length === 0 ? (
        <p className="muted empty">No repositories match “{query}”.</p>
      ) : (
        <RepoList
          repos={filtered}
          onRemove={(id) =>
            setPendingDelete(repos.find((r) => r.id === id) ?? null)
          }
          onSeen={onSeen}
        />
      )}

      {pendingDelete && (
        <ConfirmDialog
          title="Remove repository"
          message={`Stop tracking ${repoFullName(pendingDelete)}?`}
          confirmLabel="Remove"
          onConfirm={confirmDelete}
          onCancel={() => setPendingDelete(null)}
        />
      )}
    </div>
  );
}
