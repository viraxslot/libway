import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import IconButton from "@/components/ui/IconButton/IconButton";

describe("IconButton", () => {
  it("maps variant to the matching CSS class", () => {
    render(
      <IconButton variant="remove" aria-label="Remove">
        ✕
      </IconButton>,
    );
    expect(screen.getByRole("button", { name: "Remove" })).toHaveClass(
      "remove",
    );
  });

  it("defaults to type=button", () => {
    render(
      <IconButton variant="chip-remove" aria-label="Remove tag">
        ×
      </IconButton>,
    );
    expect(screen.getByRole("button")).toHaveAttribute("type", "button");
  });

  it("forwards onClick", async () => {
    const onClick = vi.fn();
    render(
      <IconButton variant="remove" aria-label="Remove" onClick={onClick}>
        ✕
      </IconButton>,
    );
    await userEvent.click(screen.getByRole("button"));
    expect(onClick).toHaveBeenCalledOnce();
  });
});
