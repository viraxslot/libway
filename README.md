# libway

A macOS menu-bar utility that tracks the versions of GitHub tools you care
about and notifies you about new releases.

- Lives in the menu bar (tray), with no Dock icon.
- Periodically checks a list of repositories (interval configurable, default
  10 minutes): first the latest release (`releases/latest`), falling back to
  the latest tag when there are no releases.
- Native notifications when a new version ships.
- The tray menu shows a status line (update count + last-checked time) and the
  current versions; clicking an entry opens the release page.
- Repositories can be tagged; the tray groups them into per-tag submenus.
- A settings window with two tabs: Repositories (add with validation, search,
  tag, remove) and Settings (token, check interval, check-on-startup, autostart).

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
scripts/install-mac.sh   build the .app and install it into /Applications
```

Data: `~/Library/Application Support/com.libway.tracker/libway.db`.
Token: Keychain, service `libway`, account `github-token`.

## Prerequisites

- **Node 24+** and **Rust 1.96+**, plus the Xcode Command Line Tools.
- Both toolchains are pinned via [`mise`](https://mise.jdx.dev) (`mise.toml`);
  the Node version is additionally enforced by `engines` in `package.json`.

```bash
mise install          # installs the pinned Node (24) and Rust (1.96)
# without mise: ensure `node -v` >= 24 and `rustc --version` >= 1.96
```

## Development

```bash
npm install
npm run tauri dev      # run in dev mode (window + tray)
```

Note: native notifications only work from a built `.app`, not from
`npm run tauri dev` (macOS does not register the unsigned dev binary). The
tray and the window work in dev; use a build to test notifications.

Lint/format (Biome) and Rust tests:

```bash
npm run lint           # biome check src  (lint:fix to autofix)
npm run test:rust      # cargo test
```

Git hooks (husky) run automatically: a pre-commit hook formats staged files
with Biome and runs the Rust tests, and a commit-msg hook enforces
[Conventional Commits](https://www.conventionalcommits.org) via commitlint
(e.g. `feat: …`, `fix: …`, `chore: …`).

Build and install into /Applications (so Spotlight and autostart find it):

```bash
npm run install:mac
```

## Releasing

```bash
npm run release:version -- patch   # or minor | major | X.Y.Z
npm run release:publish
```

`release:version` bumps the version in package.json, tauri.conf.json and
Cargo.toml, then commits and tags it. `release:publish` builds a `.dmg`,
pushes the commit and tag, and creates a GitHub release with auto-generated
notes (requires the `gh` CLI). The `.dmg` is unsigned, so first launch needs
right-click → Open to get past Gatekeeper.

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

