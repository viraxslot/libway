import { openUrl } from "@tauri-apps/plugin-opener";
import { type Repo, repoFullName } from "../types";

interface Props {
  repo: Repo;
  onRemove: (id: number) => void;
  onSeen: (id: number) => void;
}

/** Format a unix timestamp (seconds) as a short local time, or a dash. */
function formatChecked(ts: number | null): string {
  if (!ts) return "—";
  return new Date(ts * 1000).toLocaleString();
}

/** One repository row: name, version, "new" badge, open and remove actions. */
export default function RepoRow({ repo, onRemove, onSeen }: Props) {
  const name = repoFullName(repo);

  async function open() {
    if (repo.latestUrl) {
      await openUrl(repo.latestUrl);
      if (repo.hasUnseen) onSeen(repo.id);
    }
  }

  return (
    <li className="repo-row">
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
    </li>
  );
}
