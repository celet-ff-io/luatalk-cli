-- This file shows what input data should finally be like.
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
						avatar = { path = "/path/to/image", url = nil },
					},
					role = "guest",
				},
				{
					body = {
						type = "image",
						value = { path = "/path/to/image", url = nil },
					},
					profile = {
						name = "Her",
						avatar = { path = "/path/to/image", url = nil },
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
						value = { path = "/path/to/image", url = nil },
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
