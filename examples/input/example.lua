-- Example input file.
-- For "talk", see `assets/talk.lua`.

local talk = require("talk")

-- Aliases

local guest = talk.msg_guest
local host = talk.msg_host
local system = talk.msg_system
local reply = talk.msg_reply
local bond_story = talk.msg_bond_story

local image = talk.image

-- Main

local her = { name = [[Her]], avatar = [[<placeholder-0>]] }

return {
	pages = {
		{
			msgs = {
				guest(her, [[Example guest message]]),
				guest(her, image({ url = [[<placeholder-1>]] })),
				host([[Example host message]]),
				host(image({ url = [[<placeholder-2>]] })),
			},
		},
		{
			msgs = {
				system([[Example system message]]),
				reply([[Example reply message]]),
				bond_story([[Example bond story message]]),
			},
		},
	},
}
