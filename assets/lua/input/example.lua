-- Example input file.
-- For "talk", see `assets/talk.lua`.

local talk = require("talk")

-- Aliases

local page = talk.page

local guest = talk.msg_guest
local host = talk.msg_host
local system = talk.msg_system
local reply = talk.msg_reply
local bond_story = talk.msg_bond_story

-- local text = talk.body_text -- You may do not need this for most time
local image = talk.body_image -- Remember this returns `Body`, not `Image`

-- Main
-- Hint: You may use `func { ... }`-like syntax instead of `func({ ... })`.

local her = { name = [[Her]], avatar = { url = [[<placeholder-0>]] } }

return {
	pages = {
		page({
			guest(her, [[Example guest message]]),
			guest(her, image({ url = [[<placeholder-1>]] })),
			host([[Example host message]]),
			host(image({ url = [[<placeholder-2>]] })),
		}),
		page({
			system([[Example system message]]),
			reply([[Example reply message]]),
			bond_story([[Example bond story message]]),
		}),
	},
}
