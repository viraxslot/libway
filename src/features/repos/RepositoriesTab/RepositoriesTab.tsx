import { useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import ConfirmDialog from "@/components/ui/ConfirmDialog/ConfirmDialog";
import Input from "@/components/ui/Input/Input";
import AddRepoForm from "@/features/repos/AddRepoForm/AddRepoForm";
import RepoList from "@/features/repos/RepoList/RepoList";
import { type Repo, repoFullName } from "@/types";

interface Props {
  repos: Repo[];
  onAdd: (fullName: string) => Promise<void>;
  onRemove: (id: number) => void;
  onSeen: (id: number) => void;
  onSetTags: (id: number, tags: string[]) => void;
}

/** Repositories tab: add form, search filter, and the tracked list. */
export default function RepositoriesTab({
  repos,
  onAdd,
  onRemove,
  onSeen,
  onSetTags,
}: Props) {
  const [query, setQuery] = useState("");
  // The repo pending deletion confirmation, or null.
  const [pendingDelete, setPendingDelete] = useState<Repo | null>(null);
  const { t } = useTranslation();

  const filtered = useMemo(() => {
    const q = query.trim().toLowerCase();
    if (!q) {
      return repos;
    }
    // Match against the full name and any tag.
    return repos.filter(
      (r) =>
        repoFullName(r).toLowerCase().includes(q) ||
        r.tags.some((t) => t.includes(q)),
    );
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
        <Input
          type="search"
          variant="search"
          placeholder={t("repos.searchRepos")}
          value={query}
          onChange={(e) => setQuery(e.target.value)}
          spellCheck={false}
          autoCapitalize="off"
        />
      )}

      {repos.length > 0 && filtered.length === 0 ? (
        <p className="muted empty">
          {t("repos.noMatch")} “{query}”.
        </p>
      ) : (
        <RepoList
          repos={filtered}
          onRemove={(id) =>
            setPendingDelete(repos.find((r) => r.id === id) ?? null)
          }
          onSeen={onSeen}
          onSetTags={onSetTags}
        />
      )}

      {pendingDelete && (
        <ConfirmDialog
          title={t("repos.removeTitle")}
          message={`${t("repos.removeMessage1")} ${repoFullName(pendingDelete)}?`}
          confirmLabel={t("repos.confirmRemove")}
          cancelLabel={t("repos.cancelRemove")}
          onConfirm={confirmDelete}
          onCancel={() => setPendingDelete(null)}
        />
      )}
    </div>
  );
}
