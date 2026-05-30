import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Checkbox from "@/components/ui/Checkbox/Checkbox";

describe("Checkbox", () => {
  it("renders a checkbox labelled by its text", () => {
    render(<Checkbox label="Launch at login" checked onChange={() => {}} />);
    expect(
      screen.getByRole("checkbox", { name: "Launch at login" }),
    ).toBeChecked();
  });

  it("forwards the checked state", () => {
    render(
      <Checkbox label="Check on startup" checked={false} onChange={() => {}} />,
    );
    expect(screen.getByRole("checkbox")).not.toBeChecked();
  });

  it("fires onChange when toggled", async () => {
    const onChange = vi.fn();
    render(<Checkbox label="Toggle" checked={false} onChange={onChange} />);
    await userEvent.click(screen.getByRole("checkbox"));
    expect(onChange).toHaveBeenCalledOnce();
  });
});
