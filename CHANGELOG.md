<!-- markdownlint-disable MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/2.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- `path` field for the Lua class `Image`.
- `"ko"` and `"zh-Hant"` to `lang` field.

### Changed

- **Breaking:** `ImageValue` and the structures using it
  have got `TryFrom` instead of `From` trait implementation.
- **Breaking:** Changed definition of the Lua class `Image` in `talk.lua`.
- **Breaking:** Use Language-Script tag instead of Language-Region tag.

### Fixed

- Doc test in `lib.rs`.
- Wrong translation for `Ja` (`JaJp`).

## [0.3.0-alpha.2] - 2026-07-05

### Added

- Subcommand `do` for general input processing.
- `--format` option for `do` to specify input format.
- Short flag `-c` stands for `--concat-pages`.
- Output subcommands for different formats with JSON support.
- Add configuration from environment variables for advanced settings.
- Add help for configuration from environment variables.
- Derive `Serialize` for structures in `luatalk::dto`.

### Changed

- **Breaking:** `luatalk::LuaTalkExt` has been refactored to `luatalk::LuaExt`.
- **Breaking:** `luatalk::lua` has been refactored to `luatalk::dto`.

### Removed

- **Breaking:** Subcommand `show` and `export`. Use `do` instead.
- **Breaking:** Advanced options `--lib-default` and `--lib`.
  Use environment variables instead for those settings.

## [0.3.0-alpha.1] - 2026-07-01

### Changed

- `generate completion` subcommand to generate shell completion.

- Update some dependencies.

- Downgrade the minimal supported Rust version to `1.92` (maybe smaller in future)

- **Breaking:** Refactor `generate` subcommand,
  moving old ones to `generate asset`,
  keeping `generate example` as a standalone subcommand.

- **Breaking:** Asset arguments' names has been changed
  for `generate asset` (`generate` in previous versions).

## [0.2.2] - 2026-06-30

### Fixed

- Wrong readme path in `Cargo.toml`.

## [0.2.1] - 2026-06-30

### Added

- Files for cargo-deny and cargo-about.
- Quick view part in doc "Usage" section.

### Changed

- Extract "Usage" section from `README.md` to `USAGE.md`.

## [0.2.0] - 2026-06-29

### Added

- `./docs/README-zh_cn.md`.
- `CHANGELOG.md`.
- Subcommand `generate` to generate useful assets file hard-coded in the binary.
- Allow article to be exported to multiple files in pages
  by specifying `--output` with either directory or file path in format string.
- Allow article with different `lang` field specifying its language to be exported.
- `momotalk` format specified by `--format`
  to export in JSON format for [MomoTalk Editor](https://github.com/U1805/momotalk/).
- `--concat-pages` option for `export` to concatenate all pages to one file.
- Subcommand `export` to export article.

### Changed

- **Breaking:** `lang` field is required in Lua input.
- Refactor CLI logic.

### Removed

- **Breaking:** Remove `-o/--output` option for subcommand `show`.

## [0.1.1] - 2026-06-27

### Added

- Complete doc for `lib.rs` and `talk.lua`.

## [0.1.0] - 2026-06-27

### Added

- And more items to make this project under version `0.1`.
- Subcommand `show` to show LuaTalk article in `luatalk::Article` structure string.
- Common CLI options.
- Lua input examples in `./assets/`.
- `./assets/talk.lua` the useful Lua module helps generate input file for this application,
  under version `0.1.0`.
