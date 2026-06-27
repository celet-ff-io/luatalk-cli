# LuaTalk CLI

Convert your Lua file to `luatalk::Article` structure string.

## Usage

### With DSL from [`talk.lua`](./assets/lua/talk.lua)

Convert a file like [`example.lua`](./assets/lua/input/example.lua),
which uses DSL features defined in [`talk.lua`](./assets/lua/talk.lua).

You may:

- Use `--lib-default` flag to load hard-coded `talk.lua` in the binary (Recommended).
- Use `--lib` to add the directory of `talk.lua` to Lua package path.
- Copy [`talk.lua`](./assets/lua/talk.lua) to existing Lua package path like `.`,
  or just add its content to your input file.

```bash
# Read from a file, write to stdout
luatalk --lib-default example.lua
# Read from a stdin, write to stdout
cat example.lua | luatalk --lib-default -
 # Read from a file, write to a file
luatalk --lib-default example.lua -o output.txt

# Add directory of `talk.lua` to Lua path
luatalk --lib /path/to/luatalk-cli/assets/lua/lib example.lua

# Manually copy `talk.lua` to your Lua path
cp /path/to/luatalk-cli/assets/lua/lib/talk.lua talk.lua && luatalk example.lua
```

Convert a file like [`raw_example.lua`](./assets/lua/input/raw_example.lua),
which directly returns the full `Article` table.

```bash
lua luatalk.lua raw_example.lua
```
