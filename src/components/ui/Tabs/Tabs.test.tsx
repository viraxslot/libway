import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import Tab from "@/components/ui/Tab/Tab";
import Tabs from "@/components/ui/Tabs/Tabs";

describe("Tabs", () => {
  it("renders a .tabs nav wrapping its tabs", () => {
    render(
      <Tabs value="repos" onChange={() => {}}>
        <Tab value="repos">Repositories</Tab>
      </Tabs>,
    );
    const nav = screen.getByRole("navigation");
    expect(nav).toHaveClass("tabs");
    expect(nav).toContainElement(
      screen.getByRole("button", { name: "Repositories" }),
    );
  });
});
