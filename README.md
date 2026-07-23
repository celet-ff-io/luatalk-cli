# LuaTalk CLI

Build article from Lua file.

|English|[简体中文](./docs/README-zh_cn.md)|
|---|---|

## Features

- Build article from Lua file. Lua version 5.5 is supported.
- Articles here are mainly for being compile by [Typst](https://github.com/typst/typst),
  which is great to render fancy document in pages.
- Dump article JSON as intermediate file to be loaded by LuaTalk again,
  usually to keep a page-concatenated version, if needed.
- Dump article to JSON format for [MomoTalk Editor](https://github.com/U1805/momotalk/)
  if you want.

## Usage

See [USAGE](./docs/USAGE.md).

## Install

Use one of the following methods:

- Download pre-built binary from release
  if there is one that suitable for your platform.

- Use `cargo install luatalk-cli`
  to fetch source from [crates.io](https://crates.io/crates/luatalk-cli)
  , build and install it.

- Download source and use `cargo install --path .` to build and install it.

## Project goals

- Provide a simple CLI tool to build article from Lua file.
- Output the processed article in different formats.
- Outputting may work with other programs like Typst.

## Related projects

- [MomoTalk Editor](https://github.com/U1805/momotalk/) -
  Blue Archive Momotalk Editor
- [Typst](https://github.com/typst/typst) -
  A markup-based typesetting system that is powerful and easy to learn

## License

Copyright (c) 2026-present [celet-ff-io](https://github.com/celet-ff-io)

`luatalk-cli` is made available under the terms of
either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.
