export default {
  // Format and lint staged frontend files with Biome.
  "src/**/*.{ts,tsx,js,jsx,json,css}": [
    "biome check --write --no-errors-on-unmatched",
  ],
  // Run the frontend tests related to the staged TS/TSX files.
  "src/**/*.{ts,tsx}": ["vitest related --run"],
  // Rust tests run on the whole crate, so ignore the matched file list and
  // return a fixed command (a function is the only way to drop the args).
  "src-tauri/**/*.rs": () =>
    "cargo test --manifest-path src-tauri/Cargo.toml --lib",
};
