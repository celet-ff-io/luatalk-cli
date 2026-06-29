# LuaTalk CLI

用 Lua 脚本构建类 MMT 文章。

| [English](../README.md)|简体中文|
|---|---|

## TODO

- 支持导出为 Typst 代码，以便逐页渲染为图片。

## 特性

- 文章使用**页**的概念，以便导出为多个文件。
- 用 Lua 脚本构建类 MMT 文章。使用 Lua 5.5 版本。

## 使用

可用 `luatalk help` 和 `luatalk help <COMMAND>` 查看详细的 CLI 使用说明。

### 导出

可导出为不同格式。（目前只有一种）

|Format|Description|
|---|---|
|`momotalk`|[MomoTalk 编辑器](https://github.com/U1805/momotalk/)所用的 JSON 导出格式|

#### 逐页导出到多个文件

需要将导出目标用 `--output` 设置为**目录**或**格式化的文件路径**。

```bash
# Make directory `example/` and write to `example/example_1.json`, ...
luatalk export --lib-default example.lua -f momotalk
# Make directory `output/dir/` and write to `output/dir/example_1.json`, ...
luatlalk export --lib-default example.lua -f momotalk -o output/dir
# Make directory `output/` and write to `output/e_1.json`, ...
luatlalk export --lib-default example.lua -f momotalk -o output/e_{i}.json
# If you really want, the following command will work as expected too
luatlalk export --lib-default example.lua -f momotalk -o output/{i}/{i}.json
```

#### 拼接所有页面并导出到一个文件

用 `-` 导出到 stdout（标准输出）。

```bash
# Write to stdout
luatalk export --lib-default example.lua -f momotalk --concat-pages
# Write to a file
luatalk export --lib-default example.lua -f momotalk --concat-pages -o output.json
```

### 创建用于生成文章的 Lua 脚本

可用 `show` 来查看 Lua 编辑下的文章结构。

#### 使用来自 [`talk.lua`](../assets/lua/lib/talk.lua) 的辅助 DSL 构建

例如 [`example.lua`](../assets/lua/input/example.lua)
使用了 [`talk.lua`](../assets/lua/lib/talk.lua) 中定义的 DSL 特性。

可以：

- 使用 `--lib-default` 标志加载二进制中硬编码的 `talk.lua`（推荐）。
- 使用 `--lib` 将 `talk.lua` 的目录添加到 Lua 包路径中。
- 直接复制 `talk.lua` 到现有的 Lua 包路径中，例如 `.`，或者将其内容直接添加到你的输入文件中。

```bash
# Read from a file, write to stdout
luatalk show --lib-default example.lua
# Read from a stdin, write to stdout
cat example.lua | luatalk show --lib-default -
 # Read from a file, write to a file
luatalk show --lib-default example.lua -o output.txt

# Add directory of `talk.lua` to Lua path
luatalk show --lib /path/to/luatalk-cli/assets/lua/lib example.lua

# Manually copy `talk.lua` to your Lua path
cp /path/to/luatalk-cli/assets/lua/lib/talk.lua talk.lua && luatalk show example.lua
```

#### 直接在 Lua 脚本里返回需要的 table

例如 [`raw_example.lua`](../assets/lua/input/raw_example.lua)
返回了完整的用于解析为 `luatalk::Article` 的 table。

```bash
lua show raw_example.lua
```

### 生成

可以生成来自 `./assets/` 目录的有用资源文件。
它们被硬编码在二进制中。

```bash
luatalk generate example # Output `example.lua`
luatalk generate lib/talk # Output `talk.lua`
```

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
