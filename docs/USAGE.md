# Usage

|[README](../README.md)|USAGE|
|---|---|

See `README` for more information,
including how to install.

## Quick view

### Work with [MomoTalk Editor](https://github.com/U1805/momotalk/)

1. `luatalk generate example > article.lua`
  to generate your `article.lua` from example.

2. Replace `her` profile avatar URL placeholder,
  with a real student avatar image URL,
  like the one in your JSON file exported from MomoTalk Editor.
  e.g. `https://BlueArcbox.github.io/resources/Avatars/Kivo/Released/10000.webp`
  (Upload by [U1805](https://github.com/U1805) the author of MomoTalk Editor).

3. Edit your article in `article.lua` following the hints in the file.
  For detailed `talk.lua` defination,
  use `luatalk generate asset lua/lib/talk.lua > talk.lua`.

4. `luatalk show article.lua` to check if your article file is valid.

5. `luatalk export article.lua -f momotalk -o article`.
  This will create `article/`
  and export your pages in `article/article_1.json` and so on.
  (See [Export](#export) for more details).

6. Upload any of the exported JSON files to [MomoTalk website](https://u1805.github.io/momotalk).

## Detailed

Use `luatalk help` and `luatalk help <COMMAND>` for detailed CLI usage.

### Export

Export your article to different formats.

|Format|Description|
|---|---|
|`momotalk`|JSON format for [MomoTalk Editor](https://github.com/U1805/momotalk)|

#### Export in pages to files

You may set the output destination as either **directory**
or **file path in format string**.

```bash
# Make directory `example/` and write to `example/example_1.json`, ...
luatalk export example.lua -f momotalk
# Make directory `output/dir/` and write to `output/dir/example_1.json`, ...
luatlalk export example.lua -f momotalk -o output/dir
# Make directory `output/` and write to `output/e_1.json`, ...
luatlalk export example.lua -f momotalk -o output/e_{i}.json
# If you really want, the following command will work as expected too
luatlalk export example.lua -f momotalk -o output/{i}/{i}.json
```

#### Concatenate all pages to a single file

`-` as output for stdout.

```bash
# Write to stdout
luatalk export example.lua -f momotalk --concat-pages
# Write to a file
luatalk export example.lua -f momotalk --concat-pages -o output.json
```

### Write your input

`show` is a command to show your input article structure.

#### With DSL from [`talk.lua`](../assets/lua/lib/talk.lua)

A file like [`example.lua`](../assets/lua/input/example.lua)
uses DSL features defined in [`talk.lua`](../assets/lua/lib/talk.lua).

You may:

- **(Default)** Let the program auto load hard-coded `talk.lua` in the binary.
- Use `--lib` to add the directory of `talk.lua` to Lua package path.
- Copy `talk.lua` to existing Lua package path like `.`,
  or just add its content to your input file.

```bash
# Read from a file, write to stdout
luatalk show example.lua
# Read from a stdin, write to stdout
cat example.lua | luatalk show -
 # Read from a file, write to a file
luatalk show example.lua -o output.txt

# Without auto loading `talk.lua`,
# you need to manually add it to Lua path or copy it to your Lua path.

# Manually add directory of `talk.lua` to Lua path
luatalk show --no-default-lib --lib /path/to/luatalk-cli/assets/lua/lib example.lua

# Manually copy `talk.lua` to your Lua path
cp /path/to/luatalk-cli/assets/lua/lib/talk.lua talk.lua && \
luatalk show --no-default-lib example.lua
```

#### With raw Lua table

A file like [`raw_example.lua`](../assets/lua/input/raw_example.lua)
directly returns the full `Article` table.

```bash
lua show raw_example.lua
# Of course you don't need `talk.lua` here
lua show --no-default-lib raw_example.lua
```

### Generate

To generate useful files.
For the asset ones,
they are nothing different from the original files in `assets/` directory.

```bash
# Write `example.lua` to your file
luatalk generate example > article.lua
# Use bash completion script in this session
source <(luatalk generate completion bash)
# Print content of `talk.lua`
luatalk generate asset lua/lib/talk.lua
```
