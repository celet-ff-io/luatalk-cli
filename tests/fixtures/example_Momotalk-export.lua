-- Example input file in one page.
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

local aru =
	{ name = [[Aru]], avatar = { url = [[https://BlueArcbox.github.io/resources/Avatars/Kivo/Released/10000.webp]] } }

return {
	lang = "en",
	pages = {
		page({
			guest(aru, [[Example guest message]]),
			guest(aru, image({ url = [[https://BlueArcbox.github.io/resources/Stickers/01.webp]] })),
			host([[Example host message]]),
			host(image({ url = [[https://BlueArcbox.github.io/resources/Stickers/01.webp]] })),
			system([[Example system message]]),
			reply([[Example reply message
Reply line 2]]),
			bond_story([[Example bond story message]]),
		}),
	},
}
