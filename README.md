# LuaTalk CLI

Build article from Lua file.

Using Lua version 5.5.

## TODO

- Make article exportable to Typst code
  to make it easy to render as pictures page by page.

## Usage

Use `luatalk help` and `luatalk help <COMMAND>` for detailed CLI usage.

### Export

Export your article to different formats.

|Format|Description|
|---|---|
|`momotalk`|JSON format for [MomoTalk Editor](https://github.com/U1805/momotalk/)|

#### Export in pages to files

You may set the output destination as either **directory**
or **file path in format string**.

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

#### Concatenate all pages to a single file

`-` as output for stdout.

```bash
# Write to stdout
luatalk export --lib-default example.lua -f momotalk --concat-pages
# Write to a file
luatalk export --lib-default example.lua -f momotalk --concat-pages -o output.json
```

### Show

#### With DSL from [`talk.lua`](./assets/lua/talk.lua)

`show` a file like [`example.lua`](./assets/lua/input/example.lua),
which uses DSL features defined in [`talk.lua`](./assets/lua/talk.lua).

You may:

- Use `--lib-default` flag to load hard-coded `talk.lua` in the binary (Recommended).
- Use `--lib` to add the directory of `talk.lua` to Lua package path.
- Copy [`talk.lua`](./assets/lua/talk.lua) to existing Lua package path like `.`,
  or just add its content to your input file.

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

#### With raw Lua table

`show` a file like [`raw_example.lua`](./assets/lua/input/raw_example.lua),
which directly returns the full `Article` table.

```bash
lua show raw_example.lua
```

### Generate

To generate useful assets file hard-coded in the binary.

```bash
luatalk generate example # Output `example.lua`
luatalk generate lib/talk # Output `talk.lua`
```

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
