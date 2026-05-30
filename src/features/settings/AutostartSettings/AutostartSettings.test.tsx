import { render, screen, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { getAutostart, setAutostart } from "@/api";
import AutostartSettings from "@/features/settings/AutostartSettings/AutostartSettings";

vi.mock("@/api", () => ({
  getAutostart: vi.fn(),
  setAutostart: vi.fn(),
}));

const getAutostartMock = vi.mocked(getAutostart);
const setAutostartMock = vi.mocked(setAutostart);

describe("AutostartSettings", () => {
  beforeEach(() => {
    getAutostartMock.mockResolvedValue(false);
    setAutostartMock.mockResolvedValue(undefined);
  });

  it("reflects the loaded enabled state", async () => {
    getAutostartMock.mockResolvedValue(true);
    render(<AutostartSettings />);
    await waitFor(() =>
      expect(screen.getByLabelText("Launch at login")).toBeChecked(),
    );
  });

  it("calls setAutostart with the toggled value", async () => {
    render(<AutostartSettings />);
    const checkbox = screen.getByLabelText("Launch at login");
    await waitFor(() => expect(checkbox).not.toBeChecked());

    await userEvent.click(checkbox);
    expect(setAutostartMock).toHaveBeenCalledWith(true);
  });
});
