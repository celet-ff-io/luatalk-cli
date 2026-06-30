# LuaTalk CLI

Build article from Lua file.

|English|[简体中文](./docs/README-zh_cn.md)|
|---|---|

## TODO

- Make article exportable to Typst code
  to make it easy to render as pictures page by page.

## Features

- Article has **pages** make it easy to export to multiple files.
- Build article from Lua file. Using Lua version 5.5.

## Usage

See [USAGE](./docs/USAGE.md).

## Install

- Download prebuilt binary from Release if there is one suitable for your platform.
- Use `cargo install luatalk-cli`
  to fetch source from [crates.io](https://crates.io/crates/luatalk-cli)
  , build and install it.
- Download source and use `cargo run` to try it
or use `cargo build --release` to build release yourself.

## Project goals

- Provide a simple CLI tool to build article from Lua file.
- Output the processed article in different formats.

## Related projects

- [MomoTalk Editor](https://github.com/U1805/momotalk/) -
  Blue Archive Momotalk Editor

## License

Copyright (c) 2026-present [celet-ff-io](https://github.com/celet-ff-io)

`luatalk-cli` is made available under the terms of
either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.
