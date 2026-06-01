import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { beforeAll, beforeEach, describe, expect, it, vi } from "vitest";
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

// jsdom doesn't implement <dialog>.showModal(); stub it so the confirmation
// dialog's contents become accessible to role queries.
beforeAll(() => {
  HTMLDialogElement.prototype.showModal = vi.fn(function (
    this: HTMLDialogElement,
  ) {
    this.open = true;
  });
});

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

  it("asks for confirmation before clearing a stored token", async () => {
    hasTokenMock.mockResolvedValue(true);
    render(<TokenSettings />);

    const remove = await screen.findByRole("button", { name: "Remove" });
    await userEvent.click(remove);

    // Clicking Remove only opens the dialog; the token is not cleared yet.
    expect(clearTokenMock).not.toHaveBeenCalled();
    expect(
      await screen.findByRole("button", { name: "Remove token" }),
    ).toBeInTheDocument();
  });

  it("clears the token and updates the UI when confirmed", async () => {
    hasTokenMock.mockResolvedValue(true);
    render(<TokenSettings />);

    await userEvent.click(
      await screen.findByRole("button", { name: "Remove" }),
    );
    await userEvent.click(
      await screen.findByRole("button", { name: "Remove token" }),
    );

    expect(clearTokenMock).toHaveBeenCalledOnce();
    expect(await screen.findByText(/No token set\./)).toBeInTheDocument();
    expect(
      screen.queryByRole("button", { name: "Remove" }),
    ).not.toBeInTheDocument();
  });

  it("does not clear the token when the dialog is cancelled", async () => {
    hasTokenMock.mockResolvedValue(true);
    render(<TokenSettings />);

    await userEvent.click(
      await screen.findByRole("button", { name: "Remove" }),
    );
    await userEvent.click(screen.getByRole("button", { name: "Cancel" }));

    expect(clearTokenMock).not.toHaveBeenCalled();
  });
});
