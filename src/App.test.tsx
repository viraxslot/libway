import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import App from "@/App";

vi.mock("@/api", () => ({
  addRepo: vi.fn().mockResolvedValue([]),
  checkNow: vi.fn().mockResolvedValue([]),
  deleteTag: vi.fn().mockResolvedValue([]),
  listRepos: vi.fn().mockResolvedValue([]),
  markAllSeen: vi.fn().mockResolvedValue(undefined),
  markSeen: vi.fn().mockResolvedValue(undefined),
  removeRepo: vi.fn().mockResolvedValue([]),
  renameTag: vi.fn().mockResolvedValue([]),
  setRepoTags: vi.fn().mockResolvedValue([]),
}));

vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

vi.mock("@tauri-apps/plugin-opener", () => ({ openUrl: vi.fn() }));

describe("App (smoke)", () => {
  // App loads repos via listRepos() in an effect; awaiting a findBy* query lets
  // that state update settle inside act() so the tests don't log act warnings.
  it("renders the title", async () => {
    render(<App />);
    expect(
      await screen.findByRole("heading", { name: "libway" }),
    ).toBeInTheDocument();
  });

  it("renders the three tabs", async () => {
    render(<App />);
    expect(await screen.findByText("Repositories")).toBeInTheDocument();
    expect(screen.getByText("Tags")).toBeInTheDocument();
    expect(screen.getByText("Settings")).toBeInTheDocument();
  });
});
