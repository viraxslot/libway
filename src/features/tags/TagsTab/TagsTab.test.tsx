import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { beforeAll, describe, expect, it, vi } from "vitest";
import TagsTab from "@/features/tags/TagsTab/TagsTab";
import { makeRepo } from "@/test-utils/makeRepo";

beforeAll(() => {
  HTMLDialogElement.prototype.showModal = vi.fn(function (
    this: HTMLDialogElement,
  ) {
    this.open = true;
  });
});

describe("TagsTab", () => {
  it("shows the empty message when no repos have tags", () => {
    render(<TagsTab repos={[]} onRenameTag={vi.fn()} onDeleteTag={vi.fn()} />);
    expect(screen.getByText(/No tags yet/)).toBeInTheDocument();
  });

  it("lists tags with their repo count", () => {
    render(
      <TagsTab
        repos={[makeRepo({ tags: ["ci"] })]}
        onRenameTag={vi.fn()}
        onDeleteTag={vi.fn()}
      />,
    );
    expect(screen.getByRole("button", { name: "ci" })).toBeInTheDocument();
    expect(screen.getByText("1 repo")).toBeInTheDocument();
  });

  it("confirms before deleting a tag", async () => {
    const onDeleteTag = vi.fn().mockResolvedValue(undefined);
    render(
      <TagsTab
        repos={[makeRepo({ tags: ["ci"] })]}
        onRenameTag={vi.fn()}
        onDeleteTag={onDeleteTag}
      />,
    );

    await userEvent.click(
      screen.getByRole("button", { name: "Delete tag ci" }),
    );
    expect(screen.getByText("Delete tag")).toBeInTheDocument();

    await userEvent.click(screen.getByRole("button", { name: "Delete" }));
    expect(onDeleteTag).toHaveBeenCalledWith("ci");
  });

  it("renames a tag to a new name on Enter", async () => {
    const onRenameTag = vi.fn().mockResolvedValue(undefined);
    render(
      <TagsTab
        repos={[makeRepo({ tags: ["ci"] })]}
        onRenameTag={onRenameTag}
        onDeleteTag={vi.fn()}
      />,
    );

    await userEvent.click(screen.getByRole("button", { name: "ci" }));
    const input = screen.getByDisplayValue("ci");
    await userEvent.clear(input);
    await userEvent.type(input, "build{Enter}");

    expect(onRenameTag).toHaveBeenCalledWith("ci", "build");
  });
});
