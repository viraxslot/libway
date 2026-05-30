export default {
  "src/**/*.{ts,tsx,js,jsx,json,css}": [
    "biome check --write --no-errors-on-unmatched",
  ],
  "src/**/*.{ts,tsx}": ["vitest related --run"],
  "src-tauri/**/*.rs": () => [
    "bun run test:rust:unit",
    "bun run test:rust:doc",
  ],
};
