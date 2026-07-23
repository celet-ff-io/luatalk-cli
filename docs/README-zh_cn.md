# LuaTalk CLI

用 Lua 脚本构建类似 MMT 的文章。

|[English](../README.md)|简体中文|
|---|---|

## 特性

- 用 Lua 脚本构建类 MMT 文章。使用 Lua 5.5 版本。
- 文章主要用于被 [Typst](https://github.com/typst/typst) 编译。
  Typst 十分适合渲染出准确的页面排版。
- 可以将文章中间结果以 JSON 格式导出，以便再次被 LuaTalk 加载（通常用于保存页面拼接后的版本）。
- 可以将文章导出为 JSON 格式，以便被 [MomoTalk Editor](https://github.com/U1805/momotalk/)
  加载。

## 使用

详见 [USAGE](./USAGE-zh_cn.md).

## 下载安装

- 从 Release 下载适合你平台的预编译二进制文件（如果有的话）。

- 使用 `cargo install luatalk-cli`
  一键从 [crates.io](https://crates.io/crates/luatalk-cli) 获取源代码，构建并安装。

- 下载源代码并使用 `cargo install --path .` 构建并安装。

## 目标

- 提供一个简单的 CLI 工具从 Lua 文件构建文章。
- 输出处理后的文章为不同格式。
- 输出结果可以与其他程序（如 Typst）配合使用。

## 相关项目

- [MomoTalk Editor](https://github.com/U1805/momotalk/) -
  Blue Archive Momotalk Editor
- [Typst](https://github.com/typst/typst) -
  A markup-based typesetting system that is powerful and easy to learn

## License

Copyright (c) 2026-present [celet-ff-io](https://github.com/celet-ff-io)

`luatalk-cli` is made available under the terms of
either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.
