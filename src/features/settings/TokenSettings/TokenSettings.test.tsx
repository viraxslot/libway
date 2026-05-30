import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { clearToken, hasToken, setToken } from "@/api";
import TokenSettings from "@/features/settings/TokenSettings/TokenSettings";

vi.mock("@/api", () => ({
  hasToken: vi.fn(),
  setToken: vi.fn(),
  clearToken: vi.fn(),
}));

const hasTokenMock = vi.mocked(hasToken);
const setTokenMock = vi.mocked(setToken);
const clearTokenMock = vi.mocked(clearToken);

describe("TokenSettings", () => {
  beforeEach(() => {
    hasTokenMock.mockResolvedValue(false);
    setTokenMock.mockResolvedValue(undefined);
    clearTokenMock.mockResolvedValue(undefined);
  });

  it("shows 'No token set.' when there is no stored token", async () => {
    render(<TokenSettings />);
    expect(await screen.findByText(/No token set\./)).toBeInTheDocument();
  });

  it("saves the entered token via setToken", async () => {
    render(<TokenSettings />);
    await screen.findByText(/No token set\./);

    await userEvent.type(screen.getByPlaceholderText("ghp_…"), "ghp_secret");
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(setTokenMock).toHaveBeenCalledWith("ghp_secret");
  });

  it("shows Remove and calls clearToken when a token is stored", async () => {
    hasTokenMock.mockResolvedValue(true);
    render(<TokenSettings />);

    const remove = await screen.findByRole("button", { name: "Remove" });
    await userEvent.click(remove);

    expect(clearTokenMock).toHaveBeenCalledOnce();
  });
});
