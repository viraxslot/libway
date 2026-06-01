export default {
  "src/**/*.{ts,tsx,js,jsx,json,css}": [
    "biome check --write --no-errors-on-unmatched",
  ],
  "src/**/*.{ts,tsx}": ["vitest related --run"],
  "src-tauri/**/*.rs": () => [
    "cargo fmt --manifest-path src-tauri/Cargo.toml",
    "cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings",
    "bun run test:rust:unit",
    "bun run test:rust:doc",
  ],
};
