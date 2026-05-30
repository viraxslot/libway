import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Button from "@/components/ui/Button/Button";

describe("Button", () => {
  it("renders its children", () => {
    render(<Button>Check now</Button>);
    expect(
      screen.getByRole("button", { name: "Check now" }),
    ).toBeInTheDocument();
  });

  it("defaults to type=button and the primary variant (no class)", () => {
    render(<Button>Add</Button>);
    const btn = screen.getByRole("button");
    expect(btn).toHaveAttribute("type", "button");
    expect(btn.className).toBe("");
  });

  it("maps variant to the matching CSS class", () => {
    render(<Button variant="danger">Delete</Button>);
    expect(screen.getByRole("button")).toHaveClass("danger");
  });

  it("merges an extra className with the variant class", () => {
    render(
      <Button variant="link" className="tag-name">
        build
      </Button>,
    );
    const btn = screen.getByRole("button");
    expect(btn).toHaveClass("link", "tag-name");
  });

  it("forwards onClick", async () => {
    const onClick = vi.fn();
    render(<Button onClick={onClick}>Save</Button>);
    await userEvent.click(screen.getByRole("button"));
    expect(onClick).toHaveBeenCalledOnce();
  });
});
