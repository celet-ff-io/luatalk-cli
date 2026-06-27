# LuaTalk CLI

Build article from Lua file.

## Usage

Use `luatalk help` and `luatalk help <COMMAND>` for more information.

### With DSL from [`talk.lua`](./assets/lua/talk.lua)

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

### With raw Lua table

`show` a file like [`raw_example.lua`](./assets/lua/input/raw_example.lua),
which directly returns the full `Article` table.

```bash
lua show raw_example.lua
```

## License

Copyright (c) 2026-present [celet-ff-io](https://github.com/celet-ff-io)

`luatalk-cli` is made available under the terms of
either the MIT License or the Apache License 2.0, at your option.

See the LICENSE-APACHE and LICENSE-MIT files for license details.
