# Usage

|[README](../README.md)|USAGE|
|---|---|

See `README` for more information,
including how to install.

## Quick view

### Work with [MomoTalk Editor](https://github.com/U1805/momotalk/)

1. `luatalk generate example en > article.lua`
  to generate your `article.lua` from example.

2. Replace `her` profile avatar URL placeholder,
  with a real student avatar image URL,
  like the one in your JSON file exported from MomoTalk Editor.
  e.g. `https://BlueArcbox.github.io/resources/Avatars/Kivo/Released/10000.webp`
  (Upload by [U1805](https://github.com/U1805) the author of MomoTalk Editor).

3. Edit your article in `article.lua` following the hints in the file.
  For detailed `talk.lua` definition,
  use `luatalk generate asset lua/lib/talk.lua > talk.lua`.

4. `luatalk do article.lua show` to check if your article file got expected structure.

5. `luatalk do article.lua momotalk -o article`.
  This will create `article/`
  and export your pages in `article/article_1.json` and so on.
  (See [Output](#output) for more details).

6. Upload any of the exported JSON files to [MomoTalk website](https://u1805.github.io/momotalk).

## Detailed

Use `luatalk help` and `luatalk help <COMMAND>` for detailed CLI usage.

### Do process

Process your article input to output in different formats.

#### Input

|Format|Description|
|---|---|
|`lua`|Lua file with DSL defined in `talk.lua` or raw Lua table returning `luatalk::Article`|
|`json`|LuaTalk article dumped in JSON format|

The program uses the extension of input file to determine the format.
If the extension is not recognized,
you have to specify the input format with `--format` option.

For inputs from stdin, the input format defaults to `json`,
which makes it easy for piping.

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

#### Output

Use subcommand for formats to specify the output format.

|Format|Description|
|---|---|
|`show`|`luatalk::Article` shown by Rust debug|
|`json`|LuaTalk article dumped in JSON format|
|`momotalk`|JSON format for [MomoTalk Editor](https://github.com/U1805/momotalk)|

Output file(s) generally specified by `-o/--output`,
defaults to `-` as output for stdout for single file.

##### To single file

```bash
luatalk do example.lua json -o output.json
luatalk do example.lua json > output.json
```

##### To multiple files in pages

For some formats which do not support multiple pages in one file,
e.g. `momotalk`,
you have to set the output destination as either **directory**
or **file path in format string**.

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

However, for articles have only one page,
it will act as same as single file output.
You can use `--pl` to manually specify
whether to write to multiple files or not.

```bash
# To single file
luatalk do example.lua momotalk --pl=single -o output.json
# To multiple files
luatalk do example.lua momotalk --pl=multi -o output/e_{i}.json
```

##### Concatenate all pages to a single file

Use `-c/--concat-pages` to concatenate all pages to a single file.

```bash
luatalk do -c example.lua momotalk -o output.json
luatalk do -c example.lua json | luatalk do - momotalk -o output.json
```

### Generate

To generate useful files.
For the assets,
they are nothing different from the original files in `assets/` directory.

```bash
# Write `example.lua` to your file
luatalk generate example en > article.lua
# Use bash completion script in this session
source <(luatalk generate completion bash)
# Print content of `talk.lua`
luatalk generate asset lua/lib/talk.lua
```

### More details

Use `luatalk generate config-help` for more details about configuration.

#### About DSL from [`talk.lua`](../assets/lua/lib/talk.lua)

A file like [`example.lua`](../assets/lua/input/example.lua)
uses DSL features defined in [`talk.lua`](../assets/lua/lib/talk.lua).
However a file like [`raw_example.lua`](../assets/lua/input/raw_example.lua)
does not need that at all.

You may disable the default auto loading of `talk.lua`
via configuration from environment variables.

```bash
LUATALK__DO_LUA__NO_DEFAULT_LIB=1 luatalk do raw_example.lua show

# Manually copy `talk.lua` to your Lua path
cp /path/to/luatalk-cli/assets/lua/lib/talk.lua talk.lua && \
LUATALK__DO_LUA__NO_DEFAULT_LIB=1 luatalk do example.lua show

# Manually add directory of `talk.lua` to Lua path
LUATALK__DO_LUA__ADDTIONAL_PATH="/path/to/luatalk-cli/assets/lua/lib/?.lua;" \
luatalk do example.lua show
```
