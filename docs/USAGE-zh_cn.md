# 使用

|[README](./README-zh_cn.md)|USAGE|
|---|---|

更多信息，如何安装等，请参阅 `README`。

## 速览

### 与 [MomoTalk Editor](https://github.com/U1805/momotalk) 共同使用

1. `luatalk generate example > article.lua`
   来生成示例 `article.lua`。

1. 将其中 `her` 的 `avatar` 的 URL 占位符 ("placeholder")
  替换为真实的角色头像图片 URL，
  比如从 MMT 导出的 JSON 文件里那个。
  例：`https://BlueArcbox.github.io/resources/Avatars/Kivo/Released/10000.webp`
  （由 MMT 作者 [U1805](https://github.com/U1805) 上传）

1. 根据文件中提示编辑 `article.lua`。
  若要查看 `talk.lua` 的详细定义，请使用 `luatalk generate asset lua/lib/talk.lua > talk.lua`。

1. `luatalk do article.lua show` 来检查文件是否会被正确解析。

1. `luatalk do article.lua momotalk -o article`。
  将创建 `article/` 目录，并将页面导出为 `article/article_1.json` 等文件。
  (更多细节请参阅“导出”部分)。

1. 上传导出的 JSON 文件到 [MMT 网页版](https://u1805.github.io/momotalk)。

## 详细说明

可用 `luatalk help` 和 `luatalk help <COMMAND>` 查看详细的 CLI 使用说明。

### 处理

可将输入文章处理后输出为不同格式。

#### 输入

|格式|说明|
|---|---|
|`show`|`luatalk::Article` 结构的 Rust debug 输出|
|`lua`|Lua 文件，使用 `talk.lua` 中定义的 DSL 或直接返回 `luatalk::Article` 的原始 Lua table|
|`json`|LuaTalk 文章的标准 JSON 格式|

这个程序会根据输入文件的扩展名来自动判断格式。
如果扩展名无法识别，你需要使用 `--format` 选项手动指定输入格式。

对于来自标准输入的输入，默认格式为 `json`，便于管道处理。

```bash
# Format from file extension
luatalk do example.lua show
# Manually specify input format
luatalk do example.lua.bak -f lua show
# Process stdin in JSON
luatalk do example.lua json | luatalk do - show
# Process stdin in other formats
cat example.lua | luatalk do - -f lua show
```

#### 输出

可用以格式命名的子命令来指定输出格式。

|格式|说明|
|---|---|
|`json`|LuaTalk 文章的标准 JSON 格式|
|`momotalk`|[MomoTalk 编辑器](https://github.com/U1805/momotalk)|

输出文件通常由 `-o/--output` 指定，单文件默认输出到标准输出（stdout）。

##### 输出到单文件

```bash
luatalk do example.lua json -o output.json
luatalk do example.lua json > output.json
```

##### 分页输出到多个文件

对于某些不支持单文件多页的格式，例如 `momotalk`，
你必须将输出目标设置为**目录**或**格式化的文件路径**。

```bash
# Make directory `example/` and write to `example/example_1.json`, ...
luatalk do example.lua momotalk
# Make directory `output/dir/` and write to `output/dir/example_1.json`, ...
luatalk do example.lua momotalk -o output/dir
# Make directory `output/` and write to `output/e_1.json`, ...
luatalk do example.lua momotalk -o output/e_{i}.json
# If you really want, the following command will work as expected too
luatalk do example.lua momotalk -o output/{i}/{i}.json
```

然而，对于只有一页的文章，输出的工作方式将与单文件输出相同。
可用 `--pl` 手动指定是否输出到多个文件。

```bash
# To single file
luatalk do example.lua momotalk --pl=single -o output.json
# To multiple files
luatalk do example.lua momotalk --pl=multi -o output/e_{i}.json
```

##### 拼接所有页面并导出到单文件

可用 `-c/--concat-pages` 将所有页面拼接到单个文件。

```bash
luatalk do -c example.lua momotalk -o output.json
luatalk do -c example.lua json | luatalk do - momotalk -o output.json
```

### 生成

可以生成一些有用的文件。
对于 `asset` 子命令生成的文件，
它们与 `assets/` 目录下的原始文件没有区别。

```bash
# Write `example.lua` to your file
luatalk generate example > article.lua
# Use bash completion script in this session
source <(luatalk generate completion bash)
# Print content of `talk.lua`
luatalk generate asset lua/lib/talk.lua
```

### 更多

可用 `luatalk generate config-help` 查看更多关于配置的细节。

#### 关于来自 [`talk.lua`](../assets/lua/lib/talk.lua) 的辅助 DSL

像 [`example.lua`](../assets/lua/input/example.lua) 这种文件
使用了 [`talk.lua`](../assets/lua/lib/talk.lua) 中定义的 DSL 特性。
而像 [`raw_example.lua`](../assets/lua/input/raw_example.lua) 这种文件
则不需要它。

你可以用环境变量配置程序来禁用默认对 `talk.lua` 的自动加载。

```bash
LUATALK__DO_LUA__NO_DEFAULT_LIB=1 luatalk do raw_example.lua show

# Manually copy `talk.lua` to your Lua path
cp /path/to/luatalk-cli/assets/lua/lib/talk.lua talk.lua && \
LUATALK__DO_LUA__NO_DEFAULT_LIB=1 luatalk do example.lua show

# Manually add directory of `talk.lua` to Lua path
LUATALK__DO_LUA__ADDTIONAL_PATH="/path/to/luatalk-cli/assets/lua/lib/?.lua;" \
luatalk do example.lua show
```
