import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import Input from "@/components/ui/Input/Input";

describe("Input", () => {
  it("renders the default variant without a class", () => {
    render(<Input placeholder="owner/repo" />);
    expect(screen.getByPlaceholderText("owner/repo").className).toBe("");
  });

  it("maps variant to the matching CSS class", () => {
    render(<Input variant="tag" placeholder="+ tag" />);
    expect(screen.getByPlaceholderText("+ tag")).toHaveClass("tag-input");
  });

  it("merges an extra className with the variant class", () => {
    render(<Input variant="search" className="extra" placeholder="Search" />);
    expect(screen.getByPlaceholderText("Search")).toHaveClass(
      "search",
      "extra",
    );
  });

  it("forwards typing through onChange", async () => {
    const onChange = vi.fn();
    render(<Input placeholder="x" onChange={onChange} />);
    await userEvent.type(screen.getByPlaceholderText("x"), "ab");
    expect(onChange).toHaveBeenCalledTimes(2);
  });
});
