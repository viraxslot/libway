import { render, screen } from "@testing-library/react";
import { userEvent } from "@testing-library/user-event";
import { describe, expect, it, vi } from "vitest";
import AddRepoForm from "@/features/repos/AddRepoForm/AddRepoForm";

describe("AddRepoForm", () => {
  it("calls onAdd with the entered value and clears the input", async () => {
    const onAdd = vi.fn().mockResolvedValue(undefined);
    render(<AddRepoForm onAdd={onAdd} />);

    const input = screen.getByPlaceholderText("owner/repo");
    await userEvent.type(input, "owner/repo");
    await userEvent.click(screen.getByRole("button", { name: "Add" }));

    expect(onAdd).toHaveBeenCalledWith("owner/repo");
    expect(input).toHaveValue("");
  });

  it("does not call onAdd for blank input", async () => {
    const onAdd = vi.fn().mockResolvedValue(undefined);
    render(<AddRepoForm onAdd={onAdd} />);

    // Button is disabled with no input, so the form can't be submitted.
    expect(screen.getByRole("button", { name: "Add" })).toBeDisabled();

    await userEvent.type(screen.getByPlaceholderText("owner/repo"), "   ");
    expect(screen.getByRole("button", { name: "Add" })).toBeDisabled();
    expect(onAdd).not.toHaveBeenCalled();
  });

  it("shows an error message when onAdd rejects", async () => {
    const onAdd = vi.fn().mockRejectedValue(new Error("boom"));
    render(<AddRepoForm onAdd={onAdd} />);

    await userEvent.type(screen.getByPlaceholderText("owner/repo"), "a/b");
    await userEvent.click(screen.getByRole("button", { name: "Add" }));

    expect(await screen.findByText(/boom/)).toBeInTheDocument();
  });
});
