## [unreleased]

### 🚀 Features

- Tag-based grouping and a status line in the tray
- Add build:mac script and reuse it from install:mac
- Add an About submenu to the tray
- Verify version is in sync before building/releasing

### 🐛 Bug Fixes

- Stop the +tag input from stretching full width
- Wrap the tag input and enlarge tag chips
- Separate repo name and version with a dash in the tray

### 🚜 Refactor

- Move schema migrations into their own module

### 📚 Documentation

- Mention commitlint commit-msg hook in README
- Update README for tags, tray grouping and migrations
- Document tray About, build:mac and version check in README

### ⚙️ Miscellaneous Tasks

- Add commitlint with conventional commits config
