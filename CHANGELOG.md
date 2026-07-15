<!-- markdownlint-disable MD024 -->

# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/2.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Update dependencies.

## [0.3.0-rc.4] - 2026-07-15

- Fixed wrong auto extension name when using `do typst-compile --format`
  and format is not `pdf` nor `png`.

## [0.3.0-rc.3] - 2026-07-15

### Fixed

- Do not print creating directories message
  when it is creating an empty one (literally "").

## [0.3.0-rc.2] - 2026-07-14

### Fixed

- Remove redundant code.

## [0.3.0-rc.1] - 2026-07-14

## [0.3.0-alpha.6] - 2026-07-14

### Added

- Add translation for help messages of this CLI program.
- Make `do generate` supports system locale.
- Add `"svg"` as a output format for `do <INPUT> typst-compile`.

### Changed

- Use `include` in `Cargo.toml`.

### Removed

- Do not include `docs` and some files in the crate package.

### Fixed

- `do <INPUT> typst-compile` should create parent directories
  for output file if not exist.

## [0.3.0-alpha.5] - 2026-07-11

### Added

- `do <INPUT> typst-compile` subcommand.
- `do <INPUT> typst` to generate Typst code and JSON at one time.
  `--stem` to specify the output file name stem.
  Also shares the options of `generate typst` subcommand.
- `--offline` option to disable any fetching image from URL.
- `-o/--output` option for `generate`.
- Use `LUATALK__DO_JSON__MINIFY` to minify JSON output.

### Changed

- **Breaking:** Page number placeholder (or index key) has been changed
  from `i` to `p`. For example, you use `output_{p}.json` now.
- **Breaking:** Make `--data` in `generate typst --data` a positional argument
  with default value instead of an option.
- **Breaking:**: Rename `luatalk::ImageValueError` to `luatalk::domain::Error`.
- Change the default output file name stem from `"article"` to `"output"`.
- Split `app.rs` into modules under `app`.

### Fixed

- Make `do <INPUT> typst` able to fetch images from URL.
- Patch `output.typ`.
- Patch some error hints.

## [0.3.0-alpha.4] - 2026-07-08

### Added

- `generate typst` subcommand.
- Base Typst output file `output.typ` to assets.
- `generate license` subcommand.

### Changed

- **Breaking:** Use getters instead of direct access to constant strings in `luatalk::assets`.

## [0.3.0-alpha.3] - 2026-07-07

### Added

- Example variants in languages.
- `path` field for the Lua class `Image`.
- `"ko"` and `"zh-Hant"` to `lang` field.
- Add `luatalk::assets` for asset strings.

### Changed

- Use `mlua` version `0.12.0`.
- Use `include-flate` to compress assets in binary.
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
