# libway

A macOS menu-bar utility that tracks the versions of GitHub tools you care
about and notifies you about new releases.

- Lives in the menu bar (tray), with no Dock icon.
- Checks a list of repositories every 10 minutes: first the latest release
  (`releases/latest`), falling back to the latest tag when there are no
  releases.
- Native notifications when a new version ships.
- The tray menu shows current versions; clicking opens the release page.
- The repository list and token are managed in the settings window.

## Stack

- **Tauri 2** + **React + TypeScript** (Vite) — the settings window UI.
- **Rust** — tray, GitHub client, SQLite, notifications, scheduler.
- **SQLite** (`rusqlite`) — repository list and settings.
- **Keychain** (`keyring`) — the GitHub token (never stored in the database).

## Structure

```
src/                React + TS (settings window)
  api.ts            wrappers over Tauri invoke()
  components/       RepoList, RepoRow, AddRepoForm, TokenSettings
src-tauri/src/
  lib.rs            app setup, state, window events
  commands.rs       commands invoked from the UI
  db.rs             SQLite: schema + CRUD
  keychain.rs       token in the Keychain
  github.rs         GitHub API: releases → tags, version comparison
  checker.rs        core check logic (shared by command and scheduler)
  scheduler.rs      background checking loop
  tray.rs           tray menu and indicator
  notify.rs         native notifications
```

Data: `~/Library/Application Support/com.libway.app/libway.db`.
Token: Keychain, service `libway`, account `github-token`.

## Development

```bash
npm install
npm run tauri dev      # run in dev mode (window + tray)
```

Rust tests:

```bash
cargo test --manifest-path src-tauri/Cargo.toml
```

Release build (.app):

```bash
npm run tauri build
```

## GitHub token

Not required for public repositories, but it raises the API rate limit
(60 → 5000 requests per hour). Create a classic token with no scopes (public
repositories need no permissions) and paste it into the settings window — it
will be stored in the Keychain.

## Notes

- Pre-release versions are ignored (we use `releases/latest`).
- "New" means a version newer than the one the app has already shown; the
  indicator clears when the release is opened or via `mark_seen`.
- Launch at login is toggled in the settings (`tauri-plugin-autostart`,
  Login Items).
