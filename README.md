# libway

A macOS menu-bar utility that tracks the versions of GitHub tools you care
about and notifies you about new releases.

- Lives in the menu bar (tray), with no Dock icon.
- Periodically checks a list of repositories (interval configurable, default
  10 minutes): first the latest release (`releases/latest`), falling back to
  the latest tag when there are no releases.
- Native notifications when a new version ships.
- The tray menu shows current versions; clicking opens the release page.
- A settings window with two tabs: Repositories (add with validation, search,
  remove) and Settings (token, check interval, check-on-startup, autostart).

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
  db.rs             SQLite: schema + CRUD
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

A Biome + cargo-test pre-commit hook runs automatically (husky + lint-staged).

Build and install into /Applications (so Spotlight and autostart find it):

```bash
npm run install:mac
```

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
