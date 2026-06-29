-- This file shows what input data should finnaly be like.
-- The evaluated result should be as same as `example.lua`.

return {
	lang = "en",
	pages = {
		{
			msgs = {
				{
					body = {
						type = "text",
						value = { content = "Example guest message" },
					},
					profile = {
						name = "Her",
						avatar = { url = "<placeholder-0>" },
					},
					role = "guest",
				},
				{
					body = {
						type = "image",
						value = { url = "<placeholder-1>" },
					},
					profile = {
						name = "Her",
						avatar = { url = "<placeholder-0>" },
					},
					role = "guest",
				},
				{
					body = {
						type = "text",
						value = { content = "Example host message" },
					},
					role = "host",
				},
				{
					body = {
						type = "image",
						value = { url = "<placeholder-2>" },
					},
					role = "host",
				},
			},
		},
		{
			msgs = {
				{
					body = {
						type = "text",
						value = { content = "Example system message" },
					},
					role = "system",
				},
				{
					body = {
						type = "text",
						value = { content = "Example reply message" },
					},
					role = "reply",
				},
				{
					body = {
						type = "text",
						value = { content = "Example bond story message" },
					},
					role = "bond_story",
				},
			},
		},
	},
}
