# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project tries to adhere to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [Unreleased]

### Added

- **Docs:** A `CHANGELOG.md` for easier tracking of changes.
- **CLI:** A `--log-file` argument allowing for verbose logging to a file.
- **Kihon:** A post-generation cleanup step that deletes three dataset files
  needed only for initial database population.
- **Core:** More and improved logging throughout the codebase.
- **Core:** A custom font loading mechanism that checks for supported fonts on
  the system, if none are found a default font is downloaded.
- **CLI:** A `--font` argument for specifying a system font to be used.
- **UI:** A pause/resume button to the tray menu to pause detection in watch mode.
- **UI:** Helpful tooltips when hovering over buttons.
- **Kihon:** A copy button to each definition term.
- **Core:** MangaOCR as a new OCR engine.
- **UI:** A tray menu button which allows switching between Tesseract and MangaOCR.
- **UI:** An edit button to manually edit the input text.
- **UI:** A reverse button to reverse the input text. Useful in some cases where
  Tesseract recognizes horizontal text correctly but outputs it in reverse due to
  wrongful parsing as vertical text.

### Changed

- **Core:** Migrated logging system from `log`/`env_logger` to `tracing`.
- **Core:** Implemented pre-process upscaling of input image for Tesseract which
  vastly improves recognition of smaller font sizes.

### Deprecated

### Removed

- **Core:** The statically linked default font. This effectively reduces the
  size of the binary/executable by ~30MiB.

### Fixed

- **Core:** A duplicate check for whether Tesseract is installed.
- **Core:** A rare case where Tesseract would not parse horizontal text when
  vertical confidence is NaN.
- **Core:** The bug where scrolling horizontally and then clicking on a token in
  the input text would make the scroll-bar jump.

### Security

---

## [0.1.1] - 2026-03-09

### Added

- **Internal:** Basic metadata to `Cargo.toml`.

### Changed

- **Linux:** Watch mode with tray icon is now the default when no arguments are
  provided.
- **Windows:** Improved Tesseract detection by checking the default install path
  automatically.
- **Windows:** Suppressed the brief console window flicker when OCR is used.
- **Docs:** General improvements and updates to the README.

### Fixed

- **Linux:** Fixed a bug where the window position was being continuously set on
  x11.
- **Windows:** Fixed a regression in the tray icon functionality.

---

## [0.1.0] - 2026-02-28

- **Core:** Initial pre-release of the project.

---

[unreleased]: https://github.com/jasmine-blush/popup_dictionary/compare/v0.1.1...HEAD
[0.1.1]: https://github.com/jasmine-blush/popup_dictionary/compare/v0.1.0...v0.1.1
[0.1.0]: https://github.com/jasmine-blush/popup_dictionary/releases/tag/v0.1.0
