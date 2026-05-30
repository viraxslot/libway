import { openUrl } from "@tauri-apps/plugin-opener";
import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import RepoRow from "@/features/repos/RepoRow/RepoRow";
import { makeRepo } from "@/test-utils/makeRepo";

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn() }));

const openUrlMock = vi.mocked(openUrl);

describe("RepoRow", () => {
  beforeEach(() => {
    openUrlMock.mockResolvedValue(undefined);
  });

  it("renders the name and a 'new' badge when hasUnseen", () => {
    render(
      <RepoRow
        repo={makeRepo({ hasUnseen: true })}
        onRemove={vi.fn()}
        onSeen={vi.fn()}
        onSetTags={vi.fn()}
      />,
    );
    expect(screen.getByText("owner/repo")).toBeInTheDocument();
    expect(screen.getByText("new")).toBeInTheDocument();
  });

  it("opens the url and marks seen when the version is clicked", async () => {
    const onSeen = vi.fn();
    render(
      <RepoRow
        repo={makeRepo({
          hasUnseen: true,
          latestUrl: "https://example.com/release",
        })}
        onRemove={vi.fn()}
        onSeen={onSeen}
        onSetTags={vi.fn()}
      />,
    );
    await userEvent.click(screen.getByRole("button", { name: /1\.0\.0/ }));
    expect(openUrlMock).toHaveBeenCalledWith("https://example.com/release");
    expect(onSeen).toHaveBeenCalledWith(1);
  });

  it("calls onRemove with the repo id when remove is clicked", async () => {
    const onRemove = vi.fn();
    render(
      <RepoRow
        repo={makeRepo({ id: 7 })}
        onRemove={onRemove}
        onSeen={vi.fn()}
        onSetTags={vi.fn()}
      />,
    );
    await userEvent.click(screen.getByRole("button", { name: "Remove" }));
    expect(onRemove).toHaveBeenCalledWith(7);
  });

  it("adds a tag on Enter via onSetTags", async () => {
    const onSetTags = vi.fn();
    render(
      <RepoRow
        repo={makeRepo({ tags: ["ci"] })}
        onRemove={vi.fn()}
        onSeen={vi.fn()}
        onSetTags={onSetTags}
      />,
    );
    const input = screen.getByPlaceholderText("+ tag");
    await userEvent.type(input, "dev{Enter}");
    expect(onSetTags).toHaveBeenCalledWith(1, ["ci", "dev"]);
  });
});
