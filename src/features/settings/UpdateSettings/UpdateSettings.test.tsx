import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  getCheckInterval,
  getCheckOnStartup,
  getCheckSelfUpdate,
  setCheckInterval,
  setCheckOnStartup,
  setCheckSelfUpdate,
} from "@/api";
import UpdateSettings from "@/features/settings/UpdateSettings/UpdateSettings";

vi.mock("@/api", () => ({
  getCheckInterval: vi.fn(),
  setCheckInterval: vi.fn(),
  getCheckOnStartup: vi.fn(),
  setCheckOnStartup: vi.fn(),
  getCheckSelfUpdate: vi.fn(),
  setCheckSelfUpdate: vi.fn(),
}));

const getIntervalMock = vi.mocked(getCheckInterval);
const setIntervalMock = vi.mocked(setCheckInterval);
const getOnStartupMock = vi.mocked(getCheckOnStartup);
const setOnStartupMock = vi.mocked(setCheckOnStartup);
const getSelfUpdateMock = vi.mocked(getCheckSelfUpdate);
const setSelfUpdateMock = vi.mocked(setCheckSelfUpdate);

describe("UpdateSettings", () => {
  beforeEach(() => {
    getIntervalMock.mockResolvedValue(30);
    setIntervalMock.mockResolvedValue(undefined);
    getOnStartupMock.mockResolvedValue(true);
    setOnStartupMock.mockResolvedValue(undefined);
    getSelfUpdateMock.mockResolvedValue(true);
    setSelfUpdateMock.mockResolvedValue(undefined);
  });

  it("loads and displays the current interval", async () => {
    render(<UpdateSettings />);
    expect(await screen.findByDisplayValue("30")).toBeInTheDocument();
  });

  it("saves a changed interval via setCheckInterval", async () => {
    render(<UpdateSettings />);
    const input = await screen.findByDisplayValue("30");

    await userEvent.clear(input);
    await userEvent.type(input, "45");
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(setIntervalMock).toHaveBeenCalledWith(45);
  });

  it("toggles 'check on startup' via setCheckOnStartup", async () => {
    render(<UpdateSettings />);
    const checkbox = await screen.findByLabelText("Check on startup");

    await userEvent.click(checkbox);
    expect(setOnStartupMock).toHaveBeenCalledWith(false);
  });

  it("toggles 'check for app updates' via setCheckSelfUpdate", async () => {
    render(<UpdateSettings />);
    const checkbox = await screen.findByLabelText("Check for app updates");

    await userEvent.click(checkbox);
    expect(setSelfUpdateMock).toHaveBeenCalledWith(false);
  });
});
