# LuaTalk CLI

Convert your Lua file to `luatalk::Article` structure string.

## Usage

Convert a file like [`example.lua`](./assets/lua/input/example.lua),
which uses DSL features defined in [`talk.lua`](./assets/lua/talk.lua).

You can use `--lib-default` flag
to load hard-coded `talk.lua` in the binary,
or copy [`talk.lua`](./assets/lua/talk.lua) to your Lua path.

```bash
# Read from a file, write to stdout
luatalk --lib-default example.lua

# Read from a stdin, write to stdout
cat example.lua | luatalk --lib-default -

 # Read from a file, write to a file
luatalk --lib-default example.lua -o output.txt

# Manually add `talk.lua` to your Lua path
cp /path/to/luatalk-cli/assets/lua/talk.lua talk.lua && luatalk example.lua
```

Convert a file like [`raw_example.lua`](./assets/lua/input/raw_example.lua),
which directly returns the full `Article` table.

```bash
lua luatalk.lua raw_example.lua
```
