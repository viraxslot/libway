## [0.4.0] "Restless Viper" - 2026-05-30

### 🚀 Features

- Notify about new libway releases in the tray

### 🚜 Refactor

- Encapsulate db locking and split long functions
- Extract scheduler timer into a testable Schedule
- *(github)* Extract reusable http client and version comparison

### 📚 Documentation

- Tighten commit rules in CLAUDE.md

### 🧪 Testing

- Cover tray menu, scheduler, token; refactor build_menu

## [0.3.0] "Clever Moose" - 2026-05-30

### 🚀 Features

- List repositories most recently added first

### 🚜 Refactor

- Split components into ui/ and features/, add @/ alias
- Extract UI primitives and enforce block statements
- Make Tabs/Tab generic over the tab id type
- Decouple commands from the tray to make the backend testable
- Split commands into per-topic submodules
- Split db/tray modules and centralize event names

### 📚 Documentation

- Generalise structure section in README
- Add CLAUDE.md with git confirmation rules

### 🎨 Styling

- Apply rustfmt to the Rust crate

### 🧪 Testing

- Set up Vitest and add a Button test
- Add coverage tooling and tests for the remaining UI primitives
- Cover business components with unit tests
- Add backend e2e tests over the real IPC boundary
- Mock the GitHub network layer to e2e-test add_repo and check_now

### ⚙️ Miscellaneous Tasks

- Drive all pre-commit checks through lint-staged
- Add GitHub Actions workflow for lint, build and tests
- Limit unnecessary runs, tighten permissions, bump checkout
- Report backend test results in the run summary
- Run backend tests with nextest and prettier assertions
- Allow manual workflow runs via workflow_dispatch

## [0.2.1] "Wandering Stoat" - 2026-05-30

### 🐛 Bug Fixes

- Use native bun pm version in release script

### ⚙️ Miscellaneous Tasks

- Migrate package manager from npm to bun

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
