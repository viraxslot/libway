// Registers the jest-dom matchers (toBeInTheDocument, toHaveClass, …) and
// cleans up the rendered DOM after each test. Loaded via vite.config test.setupFiles.
import "@testing-library/jest-dom/vitest";
import { cleanup } from "@testing-library/react";
import { afterEach } from "vitest";

// Initialize i18next (English) so components rendering `t("…")` resolve to the
// English strings the existing tests assert on.
import "@/i18n";

afterEach(() => {
  cleanup();
});
