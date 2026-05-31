import { openUrl } from "@tauri-apps/plugin-opener";
import { type KeyboardEvent, useState } from "react";
import { useTranslation } from "react-i18next";
import Button from "@/components/ui/Button/Button";
import IconButton from "@/components/ui/IconButton/IconButton";
import Input from "@/components/ui/Input/Input";
import { type Repo, repoFullName } from "@/types";

interface Props {
  repo: Repo;
  onRemove: (id: number) => void;
  onSeen: (id: number) => void;
  onSetTags: (id: number, tags: string[]) => void;
}

/** Format a unix timestamp (seconds) as a short local time, or a dash. */
function formatChecked(ts: number | null): string {
  if (!ts) {
    return "—";
  }
  return new Date(ts * 1000).toLocaleString();
}

/** One repository row: name, version, "new" badge, tags, open/remove actions. */
export default function RepoRow({ repo, onRemove, onSeen, onSetTags }: Props) {
  const name = repoFullName(repo);
  const [newTag, setNewTag] = useState("");
  const { t } = useTranslation();

  async function open() {
    if (repo.latestUrl) {
      await openUrl(repo.latestUrl);
      if (repo.hasUnseen) {
        onSeen(repo.id);
      }
    }
  }

  function addTag() {
    const t = newTag.trim();
    // Keep the case the user typed; treat duplicates case-insensitively.
    const exists = repo.tags.some(
      (existing) => existing.toLowerCase() === t.toLowerCase(),
    );
    if (!t || exists) {
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
          {repo.hasUnseen && <span className="badge">{t("repos.new")}</span>}
        </div>
        <div className="repo-meta">
          {repo.latestVersion ? (
            <Button
              variant="link"
              type="button"
              onClick={open}
              title={t("repos.openReleasePage")}
            >
              {repo.latestVersion}
              {repo.sourceKind === "tag" && (
                <span className="tag-mark"> {t("repos.tag")}</span>
              )}
            </Button>
          ) : (
            <span className="muted">{t("repos.notCheckedYet")}</span>
          )}
          <span className="checked" title={t("repos.lastChecked")}>
            {formatChecked(repo.lastCheckedAt)}
          </span>
        </div>
        <IconButton
          variant="remove"
          type="button"
          onClick={() => onRemove(repo.id)}
          title={t("repos.removeFromList")}
          aria-label={t("repos.remove")}
        >
          ✕
        </IconButton>
      </div>

      <div className="repo-tags">
        {repo.tags.map((tag) => (
          <span key={tag} className="chip">
            {tag}
            <IconButton
              variant="chip-remove"
              type="button"
              onClick={() => removeTag(tag)}
              aria-label={t("repos.removeTagAria", { tag })}
            >
              ×
            </IconButton>
          </span>
        ))}
        <Input
          type="text"
          variant="tag"
          placeholder={t("repos.addTag")}
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
