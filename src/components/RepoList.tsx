import type { Repo } from "../types";
import RepoRow from "./RepoRow";

interface Props {
  repos: Repo[];
  onRemove: (id: number) => void;
  onSeen: (id: number) => void;
  onSetTags: (id: number, tags: string[]) => void;
}

/** The list of tracked repositories. */
export default function RepoList({
  repos,
  onRemove,
  onSeen,
  onSetTags,
}: Props) {
  if (repos.length === 0) {
    return <p className="muted empty">No repositories yet. Add one above.</p>;
  }
  return (
    <ul className="repo-list">
      {repos.map((repo) => (
        <RepoRow
          key={repo.id}
          repo={repo}
          onRemove={onRemove}
          onSeen={onSeen}
          onSetTags={onSetTags}
        />
      ))}
    </ul>
  );
}
