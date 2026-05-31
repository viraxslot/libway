import { render, screen, waitFor } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { getAutostart, getLanguage, setAutostart, setLanguage } from "@/api";
import SystemSettings from "@/features/settings/SystemSettings/SystemSettings";

vi.mock("@/api", () => ({
  getAutostart: vi.fn(),
  setAutostart: vi.fn(),
  getLanguage: vi.fn(),
  setLanguage: vi.fn(),
}));

const getAutostartMock = vi.mocked(getAutostart);
const setAutostartMock = vi.mocked(setAutostart);
const getLanguageMock = vi.mocked(getLanguage);
const setLanguageMock = vi.mocked(setLanguage);

describe("SystemSettings", () => {
  beforeEach(() => {
    getAutostartMock.mockResolvedValue(false);
    setAutostartMock.mockResolvedValue(undefined);
    getLanguageMock.mockResolvedValue("en");
    setLanguageMock.mockResolvedValue(undefined);
  });

  it("reflects the loaded enabled state", async () => {
    getAutostartMock.mockResolvedValue(true);
    render(<SystemSettings />);
    await waitFor(() =>
      expect(screen.getByLabelText("Launch at login")).toBeChecked(),
    );
  });

  it("calls setAutostart with the toggled value", async () => {
    render(<SystemSettings />);
    const checkbox = screen.getByLabelText("Launch at login");
    await waitFor(() => expect(checkbox).not.toBeChecked());

    await userEvent.click(checkbox);
    expect(setAutostartMock).toHaveBeenCalledWith(true);
  });
});
