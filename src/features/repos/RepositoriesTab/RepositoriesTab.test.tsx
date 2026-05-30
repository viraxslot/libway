import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { beforeAll, describe, expect, it, vi } from "vitest";
import RepositoriesTab from "@/features/repos/RepositoriesTab/RepositoriesTab";
import { makeRepo } from "@/test-utils/makeRepo";

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn() }));

beforeAll(() => {
  HTMLDialogElement.prototype.showModal = vi.fn(function (
    this: HTMLDialogElement,
  ) {
    this.open = true;
  });
});

const repos = [
  makeRepo({ id: 1, owner: "alpha", name: "one" }),
  makeRepo({ id: 2, owner: "beta", name: "two" }),
];

function renderTab(props: Partial<Parameters<typeof RepositoriesTab>[0]> = {}) {
  return render(
    <RepositoriesTab
      repos={repos}
      onAdd={vi.fn().mockResolvedValue(undefined)}
      onRemove={vi.fn()}
      onSeen={vi.fn()}
      onSetTags={vi.fn()}
      {...props}
    />,
  );
}

describe("RepositoriesTab", () => {
  it("filters the list by the search query", async () => {
    renderTab();
    expect(screen.getByText("alpha/one")).toBeInTheDocument();
    expect(screen.getByText("beta/two")).toBeInTheDocument();

    await userEvent.type(
      screen.getByPlaceholderText("Search repositories…"),
      "alpha",
    );

    expect(screen.getByText("alpha/one")).toBeInTheDocument();
    expect(screen.queryByText("beta/two")).not.toBeInTheDocument();
  });

  it("hides the search box when there are no repos", () => {
    renderTab({ repos: [] });
    expect(
      screen.queryByPlaceholderText("Search repositories…"),
    ).not.toBeInTheDocument();
  });

  it("confirms before removing a repo", async () => {
    const onRemove = vi.fn();
    renderTab({ onRemove });

    const removeButtons = screen.getAllByRole("button", { name: "Remove" });
    await userEvent.click(removeButtons[0]);

    expect(screen.getByText("Remove repository")).toBeInTheDocument();

    // The confirm button inside the dialog also has the "Remove" label.
    const confirm = screen
      .getAllByRole("button", { name: "Remove" })
      .find((b) => b.classList.contains("danger"));
    await userEvent.click(confirm as HTMLElement);

    expect(onRemove).toHaveBeenCalledWith(1);
  });
});
