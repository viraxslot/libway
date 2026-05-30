## [0.2.0] "Radiant Yak" - 2026-05-30

### 🚀 Features

- Add tag manager tab for bulk rename/delete/merge

### 🐛 Bug Fixes

- Give each release its own changelog codename

## [0.1.0] "Bright Viper" - 2026-05-30

### 🚀 Features

- Tag-based grouping and a status line in the tray
- Add build:mac script and reuse it from install:mac
- Add an About submenu to the tray
- Verify version is in sync before building/releasing
- Generate CHANGELOG.md from conventional commits with git-cliff
- Launch the app after install:mac
- Normal-color About info and a link indicator
- Add retag script to move a tag onto HEAD
- Add release codenames via a tiny Rust generator

### 🐛 Bug Fixes

- Stop the +tag input from stretching full width
- Wrap the tag input and enlarge tag chips
- Separate repo name and version with a dash in the tray
- Move the About submenu next to Quit in the tray
- Preserve tag case, dedupe case-insensitively
- Drop the stray divider above the Settings tab
- Show a literal ampersand in the About authors line
- Brace $TAG before the ellipsis in release-publish
- Build the release DMG non-interactively
- Use git-cliff changelog as the GitHub release notes
- Use ASCII in script messages to avoid unbound-variable errors
- Ad-hoc sign the app via signingIdentity to avoid "damaged" error

### 🚜 Refactor

- Move schema migrations into their own module

### 📚 Documentation

- Mention commitlint commit-msg hook in README
- Update README for tags, tray grouping and migrations
- Document tray About, build:mac and version check in README
- Regenerate CHANGELOG.md
- Regenerate CHANGELOG.md under the 0.1.0 release

### ⚙️ Miscellaneous Tasks

- Add commitlint with conventional commits config
