import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Tab from "@/components/ui/Tab/Tab";
import Tabs from "@/components/ui/Tabs/Tabs";

describe("Tab", () => {
  it("marks the active tab and not the others", () => {
    render(
      <Tabs value="repos" onChange={() => {}}>
        <Tab value="repos">Repositories</Tab>
        <Tab value="tags">Tags</Tab>
      </Tabs>,
    );
    expect(screen.getByRole("button", { name: "Repositories" })).toHaveClass(
      "tab",
      "active",
    );
    const tags = screen.getByRole("button", { name: "Tags" });
    expect(tags).toHaveClass("tab");
    expect(tags).not.toHaveClass("active");
  });

  it("calls onChange with its value when clicked", async () => {
    const onChange = vi.fn();
    render(
      <Tabs value="repos" onChange={onChange}>
        <Tab value="repos">Repositories</Tab>
        <Tab value="tags">Tags</Tab>
      </Tabs>,
    );
    await userEvent.click(screen.getByRole("button", { name: "Tags" }));
    expect(onChange).toHaveBeenCalledWith("tags");
  });

  it("throws when used outside <Tabs>", () => {
    // Silence the expected React error boundary logging for this case.
    const spy = vi.spyOn(console, "error").mockImplementation(() => {});
    expect(() => render(<Tab value="x">X</Tab>)).toThrow(
      "Tab must be used within <Tabs>",
    );
    spy.mockRestore();
  });
});
