import { openUrl } from "@tauri-apps/plugin-opener";
import { type KeyboardEvent, useState } from "react";
import { type Repo, repoFullName } from "../types";

interface Props {
  repo: Repo;
  onRemove: (id: number) => void;
  onSeen: (id: number) => void;
  onSetTags: (id: number, tags: string[]) => void;
}

/** Format a unix timestamp (seconds) as a short local time, or a dash. */
function formatChecked(ts: number | null): string {
  if (!ts) return "—";
  return new Date(ts * 1000).toLocaleString();
}

/** One repository row: name, version, "new" badge, tags, open/remove actions. */
export default function RepoRow({ repo, onRemove, onSeen, onSetTags }: Props) {
  const name = repoFullName(repo);
  const [newTag, setNewTag] = useState("");

  async function open() {
    if (repo.latestUrl) {
      await openUrl(repo.latestUrl);
      if (repo.hasUnseen) onSeen(repo.id);
    }
  }

  function addTag() {
    const t = newTag.trim().toLowerCase();
    if (!t || repo.tags.includes(t)) {
      setNewTag("");
      return;
    }
    onSetTags(repo.id, [...repo.tags, t]);
    setNewTag("");
  }

  function removeTag(tag: string) {
    onSetTags(
      repo.id,
      repo.tags.filter((t) => t !== tag),
    );
  }

  function onTagKey(e: KeyboardEvent<HTMLInputElement>) {
    if (e.key === "Enter") {
      e.preventDefault();
      addTag();
    }
  }

  return (
    <li className="repo-row">
      <div className="repo-top">
        <div className="repo-main">
          <span className="repo-name">{name}</span>
          {repo.hasUnseen && <span className="badge">new</span>}
        </div>
        <div className="repo-meta">
          {repo.latestVersion ? (
            <button
              type="button"
              className="link"
              onClick={open}
              title="Open release page"
            >
              {repo.latestVersion}
              {repo.sourceKind === "tag" && (
                <span className="tag-mark"> (tag)</span>
              )}
            </button>
          ) : (
            <span className="muted">not checked yet</span>
          )}
          <span className="checked" title="Last checked">
            {formatChecked(repo.lastCheckedAt)}
          </span>
        </div>
        <button
          type="button"
          className="remove"
          onClick={() => onRemove(repo.id)}
          title="Remove from list"
          aria-label="Remove"
        >
          ✕
        </button>
      </div>

      <div className="repo-tags">
        {repo.tags.map((tag) => (
          <span key={tag} className="chip">
            {tag}
            <button
              type="button"
              className="chip-remove"
              onClick={() => removeTag(tag)}
              aria-label={`Remove tag ${tag}`}
            >
              ×
            </button>
          </span>
        ))}
        <input
          type="text"
          className="tag-input"
          placeholder="+ tag"
          value={newTag}
          onChange={(e) => setNewTag(e.target.value)}
          onKeyDown={onTagKey}
          onBlur={addTag}
          spellCheck={false}
          autoCapitalize="off"
        />
      </div>
    </li>
  );
}
