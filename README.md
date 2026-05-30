# libway

A macOS menu-bar utility that periodically checks a list of GitHub
repositories and notifies you about new releases. It lives in the tray
(no Dock icon), groups repositories by tag, and is managed from a small
settings window.

See [CHANGELOG.md](CHANGELOG.md) for the history of changes.

## Installation

Download the `.dmg` from the [latest release](../../releases/latest) and drag
`libway.app` into Applications. The app is ad-hoc signed but not notarized by
Apple, so on first launch macOS blocks it as an "unidentified developer".
Either:

- **right-click** `libway.app` in Applications → **Open** → **Open**, or
- run `xattr -dr com.apple.quarantine /Applications/libway.app`, then open it.

## Stack

- **Tauri 2** + **React + TypeScript** (Vite) — the settings window UI.
- **Rust** — tray, GitHub client, SQLite, notifications, scheduler.
- **SQLite** (`rusqlite`) — repository list and settings.
- **Keychain** (`keyring`) — the GitHub token (never stored in the database).

## Structure

```
src/                React + TS (settings window)
  api.ts            wrappers over Tauri invoke()
  components/       RepositoriesTab, SettingsTab and their parts
                    (AddRepoForm, RepoList, RepoRow, ConfirmDialog,
                     TokenSettings, IntervalSettings, AutostartSettings)
src-tauri/src/
  lib.rs            app setup, state, window events
  commands.rs       commands invoked from the UI
  db.rs             SQLite: Repo model + CRUD
  migrations.rs     versioned schema migrations (PRAGMA user_version)
  keychain.rs       token in the Keychain
  github.rs         GitHub API: existence check, releases → tags, comparison
  checker.rs        core check logic (shared by command and scheduler)
  scheduler.rs      background checking loop
  tray.rs           tray menu and indicator
  notify.rs         native notifications
scripts/build-mac.sh     build the .app (no install)
scripts/install-mac.sh   build the .app and install it into /Applications
scripts/check-versions.sh  assert the version matches across config files
scripts/retag.sh         move a tag onto the current HEAD (local + remote)
```

Data: `~/Library/Application Support/com.libway.tracker/libway.db`.
Token: Keychain, service `libway`, account `github-token`.

## Prerequisites

- **Bun 1.3+** and **Rust 1.96+**, plus the Xcode Command Line Tools.
- Both toolchains are pinned via [`mise`](https://mise.jdx.dev) (`mise.toml`);
  the Bun version is additionally enforced by `engines` in `package.json`.

```bash
mise install          # installs the pinned Bun (1.3), Rust (1.96), git-cliff
# without mise: ensure `bun -v` >= 1.3 and `rustc --version` >= 1.96
```

## Development

```bash
bun install
bun run tauri dev      # run in dev mode (window + tray)
```

Note: native notifications only work from a built `.app`, not from
`bun run tauri dev` (macOS does not register the unsigned dev binary). The
tray and the window work in dev; use a build to test notifications.

Lint/format (Biome) and Rust tests:

```bash
bun run lint           # biome check src  (lint:fix to autofix)
bun run test:rust      # cargo test
```

Git hooks (husky) run automatically: a pre-commit hook formats staged files
with Biome and runs the Rust tests, and a commit-msg hook enforces
[Conventional Commits](https://www.conventionalcommits.org) via commitlint
(e.g. `feat: …`, `fix: …`, `chore: …`).

Build the `.app` (without installing), or build and install it into
/Applications (so Spotlight and autostart find it):

```bash
bun run build:mac      # builds the .app, prints its path
bun run install:mac    # builds and copies it into /Applications
```

## Releasing

```bash
bun run release:version patch   # or minor | major | X.Y.Z
bun run release:publish
```

`release:version` bumps the version in package.json, tauri.conf.json and
Cargo.toml, regenerates `CHANGELOG.md` from the Conventional Commits with
[git-cliff](https://git-cliff.org), then commits and tags it. Run
`bun run changelog` to regenerate it manually. `release:publish` builds a `.dmg`,
pushes the commit and tag, and creates a GitHub release whose notes are the
git-cliff changelog for that tag (requires the `gh` CLI). The `.dmg` is
unsigned, so first launch needs right-click → Open to get past Gatekeeper.

If a tag was created a few commits early, move it onto the current HEAD
(locally and on the remote) before publishing:

```bash
bun run retag              # moves the current version's tag (vX.Y.Z)
bun run retag v0.1.0       # moves a specific tag
```

The version must stay in sync across those three files (the tray's About
reads it from Cargo.toml). `build:mac` and `release:publish` verify this
automatically; run `bun run check:versions` to check manually.

## GitHub token

Not required for public repositories, but it raises the API rate limit
(60 → 5000 requests per hour). Create a classic token with no scopes (public
repositories need no permissions) and paste it into the settings window — it
will be stored in the Keychain.

## Notes

- Adding a repository verifies it exists on GitHub first; unknown repos are
  rejected with an error.
- Pre-release versions are ignored (we use `releases/latest`).
- "New" means a version newer than the one the app has already shown; the
  indicator clears when the release is opened, via "Mark all as read", or
  when a single entry is opened.
- The check interval and whether to check on startup are configurable in the
  Settings tab.
- Launch at login is toggled in the settings (`tauri-plugin-autostart`,
  Login Items).
- Tags group repositories in the tray; untagged repos fall under "Ungrouped",
  and with no tags at all the tray shows a flat list.
- Schema changes are append-only migrations in `src-tauri/src/migrations.rs`
  (add a new SQL entry to `MIGRATIONS`; the array index is the version).

