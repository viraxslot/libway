import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import RepoList from "@/features/repos/RepoList/RepoList";
import { makeRepo } from "@/test-utils/makeRepo";

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn() }));

describe("RepoList", () => {
  it("shows the empty message when there are no repos", () => {
    render(
      <RepoList
        repos={[]}
        onRemove={vi.fn()}
        onSeen={vi.fn()}
        onSetTags={vi.fn()}
      />,
    );
    expect(screen.getByText(/No repositories yet/)).toBeInTheDocument();
  });

  it("renders a row for each repo", () => {
    const repos = [
      makeRepo({ id: 1, owner: "owner", name: "repo" }),
      makeRepo({ id: 2, owner: "foo", name: "bar" }),
    ];
    render(
      <RepoList
        repos={repos}
        onRemove={vi.fn()}
        onSeen={vi.fn()}
        onSetTags={vi.fn()}
      />,
    );
    expect(screen.getByText("owner/repo")).toBeInTheDocument();
    expect(screen.getByText("foo/bar")).toBeInTheDocument();
  });
});
