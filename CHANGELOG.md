<!-- markdownlint-disable MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/2.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.2.0-alpha] - 2026-06/29

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
