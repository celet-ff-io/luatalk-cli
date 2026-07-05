-- Example input file.
-- For "talk" definition, see `assets/lib/talk.lua`.

-- Although `talk.lua` is loaded to LuaTalk by default,
-- you may still generate or get copy of `talk.lua` to the same directory as this file
-- to get code completion and type checking in your IDE.
local talk = require("talk")

-- Aliases

local page = talk.page

-- All `body` parameters of the following functions
-- requries `Body` table or text content string.
-- `Body` table is like { type = type, value = value },
-- where `type` is either `"text"` or `"image"`, `value` is either `Text` table or `Image` table.

local guest = talk.msg_guest
local host = talk.msg_host
local system = talk.msg_system
local reply = talk.msg_reply
local bond_story = talk.msg_bond_story

-- `talk.body_text` requires `Text` table or text content string.
-- `Text` table is like { content = [[Example content]] }.

-- `talk.body_image` requires `Image` table or path string.
-- `Image` table is like { path = "/path/to/image", url = "https://example.com/path/to/image" }.
-- In `Image` table, you provide either `url` or `path`, or both.
-- `path` is the path to file which either exists or not, or nil.
-- If file does not exist, LuaTalk should try fetch it from URL.
-- If is nil, LuaTalk should set it to `<dir>/<sha256>-<filename>` as fallback,
-- where <dir> could be anywhere, <sha256> is hash of URL, and <filename> is the filename of URL.
-- Do not set it to nil,
-- unless you are sure that your article won't be exported to any file which requires local image file.
-- `url` is the URL to fetch image from, optional.
-- This field is mainly for some exported formats could use URL for image instead of local file.
-- You may use it to mark the image source as addition to path, but not alternative to it.

-- local text = talk.body_text -- You may do not need this for most time
local image = talk.body_image -- Remember this returns `Body`, not `Image`

------------
--- MAIN ---
------------

-- Hint: You may use `func { ... }`-like syntax instead of `func({ ... })`.

local her = { name = [[Her]], avatar = { path = "/path/to/image" } }
-- local her = { name = [[Her]], avatar = { path = "/path/to/image", url = "https://example.com/path/to/image" } }
-- Generally, don't use the following one
-- unless you export your article only to `momotalk` format to upload to `https://u1805.github.io/momotalk/`.
-- local her = { name = [[Her]], avatar = { url = "https://example.com/path/to/image" } }

return {
	lang = "en", -- Available: `en`, `ja`, `ko`, `zh-Hans`, `zh-Hant`
	pages = {
		page({
			guest(her, [[Example guest message]]),
			guest(her, image("/path/to/image")),
			host([[Example host message]]),
			host(image("/path/to/image")),
		}),
		page({
			-- host({ content = [[Example host message]] }),
			-- host(image({ path = "/path/to/image" })),
			system([[Example system message]]),
			reply([[Example reply message]]),
			bond_story([[Example bond story message]]),
		}),
	},
}
