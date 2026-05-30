// Registers the jest-dom matchers (toBeInTheDocument, toHaveClass, …) and
// cleans up the rendered DOM after each test. Loaded via vite.config test.setupFiles.
import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

afterEach(() => {
  cleanup();
});
