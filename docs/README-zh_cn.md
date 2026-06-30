# LuaTalk CLI

用 Lua 脚本构建类 MMT 文章。

| [English](./README.md)|简体中文|
|---|---|

## TODO

- 支持导出为 Typst 代码，以便逐页渲染为图片。

## 特性

- 文章使用**页**的概念，以便导出为多个文件。
- 用 Lua 脚本构建类 MMT 文章。使用 Lua 5.5 版本。

## 使用

详见 [USAGE](./USAGE-zh_cn.md).

## 下载安装

- 从 Release 下载适合你平台的预编译二进制文件（如果有的话）。
- 使用 `cargo install luatalk-cli` 一键从 [crates.io](https://crates.io/crates/luatalk-cli) 获取源代码，构建并安装。
- 下载源代码并使用 `cargo run` 尝试，或使用 `cargo build --release` 构建 release 二进制。

## 目标

- 提供一个简单的 CLI 工具从 Lua 文件构建文章。
- 输出处理后的文章为不同格式。

## 相关项目

- [MomoTalk 编辑器](https://github.com/U1805/momotalk/) - 碧蓝档案聊天对话生成器

## License

Copyright (c) 2026-present [celet-ff-io](https://github.com/celet-ff-io)

`luatalk-cli` is made available under the terms of
either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.
