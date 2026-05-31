import { type KeyboardEvent, useMemo, useState } from "react";
import { useTranslation } from "react-i18next";
import Button from "@/components/ui/Button/Button";
import ConfirmDialog from "@/components/ui/ConfirmDialog/ConfirmDialog";
import IconButton from "@/components/ui/IconButton/IconButton";
import Input from "@/components/ui/Input/Input";
import type { Repo } from "@/types";

interface Props {
  repos: Repo[];
  onRenameTag: (from: string, to: string) => Promise<void>;
  onDeleteTag: (tag: string) => Promise<void>;
}

interface TagInfo {
  /** First spelling encountered, shown to the user. */
  name: string;
  /** Number of repositories carrying this tag. */
  count: number;
}

/** A pending merge or delete awaiting confirmation. */
type Pending =
  | { kind: "merge"; from: string; to: string; count: number }
  | { kind: "delete"; tag: string; count: number };

/** Unique tags across all repos, with repo counts. Grouped case-insensitively,
 * keeping the first spelling — mirrors the backend's join_tags rule. */
function collectTags(repos: Repo[]): TagInfo[] {
  const byLower = new Map<string, TagInfo>();
  for (const repo of repos) {
    for (const tag of repo.tags) {
      const key = tag.toLowerCase();
      const existing = byLower.get(key);
      if (existing) {
        existing.count += 1;
      } else {
        byLower.set(key, { name: tag, count: 1 });
      }
    }
  }
  return [...byLower.values()].sort((a, b) =>
    a.name.toLowerCase().localeCompare(b.name.toLowerCase()),
  );
}

/** Tags tab: list every tag with its repo count and bulk rename/delete it. */
export default function TagsTab({ repos, onRenameTag, onDeleteTag }: Props) {
  const tags = useMemo(() => collectTags(repos), [repos]);
  // The tag currently being renamed, and the draft text.
  const [editing, setEditing] = useState<string | null>(null);
  const [draft, setDraft] = useState("");
  const [pending, setPending] = useState<Pending | null>(null);
  const { t } = useTranslation();

  function startEdit(name: string) {
    setEditing(name);
    setDraft(name);
  }

  function cancelEdit() {
    setEditing(null);
    setDraft("");
  }

  function submitRename(from: string) {
    const to = draft.trim();
    const unchanged = to.toLowerCase() === from.toLowerCase();
    if (!to || unchanged) {
      cancelEdit();
      return;
    }
    // If the target already exists (case-insensitively) it's a merge — confirm.
    const target = tags.find((t) => t.name.toLowerCase() === to.toLowerCase());
    if (target) {
      const fromCount = tags.find((t) => t.name === from)?.count ?? 0;
      setPending({
        kind: "merge",
        from,
        to: target.name,
        count: fromCount + target.count,
      });
      return;
    }
    cancelEdit();
    void onRenameTag(from, to);
  }

  function onRenameKey(e: KeyboardEvent<HTMLInputElement>, from: string) {
    if (e.key === "Enter") {
      e.preventDefault();
      submitRename(from);
    } else if (e.key === "Escape") {
      e.preventDefault();
      cancelEdit();
    }
  }

  function askDelete(name: string, count: number) {
    setPending({ kind: "delete", tag: name, count });
  }

  function confirmPending() {
    if (!pending) {
      return;
    }
    if (pending.kind === "merge") {
      void onRenameTag(pending.from, pending.to);
    } else {
      void onDeleteTag(pending.tag);
    }
    setPending(null);
    cancelEdit();
  }

  if (tags.length === 0) {
    return (
      <div className="tab-panel">
        <p className="muted empty">{t("tags.noTagsYet")}</p>
      </div>
    );
  }

  return (
    <div className="tab-panel">
      <ul className="tag-list">
        {tags.map((tag) => (
          <li key={tag.name} className="tag-row">
            {editing === tag.name ? (
              <Input
                type="text"
                variant="tag"
                value={draft}
                onChange={(e) => setDraft(e.target.value)}
                onKeyDown={(e) => onRenameKey(e, tag.name)}
                onBlur={() => submitRename(tag.name)}
                spellCheck={false}
                autoCapitalize="off"
                autoFocus
              />
            ) : (
              <Button
                variant="link"
                className="tag-name"
                type="button"
                onClick={() => startEdit(tag.name)}
                title={t("tags.renameTag")}
              >
                {tag.name}
              </Button>
            )}
            <span className="muted tag-count">
              {t("tags.repoCount", { count: tag.count })}
            </span>
            <IconButton
              variant="remove"
              type="button"
              onClick={() => askDelete(tag.name, tag.count)}
              title={t("tags.removeTagFromEverywhere")}
              aria-label={`${t("tags.removeTag")} ${tag.name}`}
            >
              ✕
            </IconButton>
          </li>
        ))}
      </ul>

      {pending?.kind === "merge" && (
        <ConfirmDialog
          title={t("tags.mergeTitle")}
          message={t("tags.mergeMessage", {
            from: pending.from,
            to: pending.to,
            count: pending.count,
          })}
          confirmLabel={t("tags.confirmMerge")}
          cancelLabel={t("tags.cancelMerge")}
          onConfirm={confirmPending}
          onCancel={() => setPending(null)}
        />
      )}
      {pending?.kind === "delete" && (
        <ConfirmDialog
          title={t("tags.removeTitle")}
          message={t("tags.removeMessage", {
            tag: pending.tag,
            count: pending.count,
          })}
          confirmLabel={t("tags.confirmRemove")}
          cancelLabel={t("tags.cancelRemove")}
          onConfirm={confirmPending}
          onCancel={() => setPending(null)}
        />
      )}
    </div>
  );
}
