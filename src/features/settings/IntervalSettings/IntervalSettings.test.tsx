import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { beforeEach, describe, expect, it, vi } from "vitest";
import {
  getCheckInterval,
  getCheckOnStartup,
  setCheckInterval,
  setCheckOnStartup,
} from "@/api";
import IntervalSettings from "@/features/settings/IntervalSettings/IntervalSettings";

vi.mock("@/api", () => ({
  getCheckInterval: vi.fn(),
  setCheckInterval: vi.fn(),
  getCheckOnStartup: vi.fn(),
  setCheckOnStartup: vi.fn(),
}));

const getIntervalMock = vi.mocked(getCheckInterval);
const setIntervalMock = vi.mocked(setCheckInterval);
const getOnStartupMock = vi.mocked(getCheckOnStartup);
const setOnStartupMock = vi.mocked(setCheckOnStartup);

describe("IntervalSettings", () => {
  beforeEach(() => {
    getIntervalMock.mockResolvedValue(30);
    setIntervalMock.mockResolvedValue(undefined);
    getOnStartupMock.mockResolvedValue(true);
    setOnStartupMock.mockResolvedValue(undefined);
  });

  it("loads and displays the current interval", async () => {
    render(<IntervalSettings />);
    expect(await screen.findByDisplayValue("30")).toBeInTheDocument();
  });

  it("saves a changed interval via setCheckInterval", async () => {
    render(<IntervalSettings />);
    const input = await screen.findByDisplayValue("30");

    await userEvent.clear(input);
    await userEvent.type(input, "45");
    await userEvent.click(screen.getByRole("button", { name: "Save" }));

    expect(setIntervalMock).toHaveBeenCalledWith(45);
  });

  it("toggles 'check on startup' via setCheckOnStartup", async () => {
    render(<IntervalSettings />);
    const checkbox = await screen.findByLabelText("Check on startup");

    await userEvent.click(checkbox);
    expect(setOnStartupMock).toHaveBeenCalledWith(false);
  });
});
