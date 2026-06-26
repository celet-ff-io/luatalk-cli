--- Useful Lua module helps generate input file for luatalk crate.
--- See the crate homepage `https://github.com/celet-ff-io/luatalk-cli`.
--- @module talk
--- @author celet-ff-io
--- @copyright 2026-present celet-ff-io

local talk = {}

-- Models

--- @class Article
--- @field pages Page[]

--- @class Page
--- @field msgs Msg[]

--- @class Msg
--- @field role string
--- @field body Body
--- @field profile Profile?

--- @class Body
--- @field type string
--- @field value Text | Image

--- @class Profile
--- @field name string
--- @field avatar Image

--- @class Text
--- @field content string

--- @class Image
--- @field url string

-- Checkers

--- @param obj any
--- @return boolean
local function is_text(obj)
	return type(obj) == "table" and type(obj.content) == "string"
end

--- @param obj any
--- @return boolean
local function is_image(obj)
	return type(obj) == "table" and type(obj.url) == "string"
end

--- @param obj any
--- @return boolean
local function is_profile(obj)
	return type(obj) == "table" and type(obj.name) == "string" and is_image(obj.avatar)
end

--- @param obj any
--- @return boolean
local function is_body(obj)
	return type(obj) == "table" and type(obj.type) == "string" and type(obj.value) == "table"
end

--- @param obj any
--- @return boolean
local function is_msg(obj)
	return type(obj) == "table"
		and type(obj.role) == "string"
		and is_body(obj.body)
		and (obj.profile == nil or is_profile(obj.profile))
end

-- Constants

talk.Role = {
	Guest = "guest",
	Host = "host",
	System = "system",
	Reply = "reply",
	BondStory = "bond_story",
}

talk.Type = {
	Text = "text",
	Image = "image",
}

-- Functions

--- Create a page.
--- Checks only the first entry of list for quick content validation.
--- @param msgs Msg[]
--- @return Page
function talk.page(msgs)
	assert(type(msgs) == "table", "Msg list must be a table")
	assert(#msgs > 0, "Msg list must not be empty")
	assert(is_msg(msgs[1]), "First msg of list is invalid. Please check your msg list")
	return { msgs = msgs }
end

--- Create a message with role, body, and optional profile.
--- @param role string
--- @param body Body | string Body or Text content
--- @param profile Profile?
--- @return Msg
local function role_msg(role, body, profile)
	assert(type(role) == "string", "Invaild msg role")
	if type(body) == "string" then
		body = talk.body_text({ content = body })
	else
		assert(is_body(body), "Invaild msg body")
	end
	if profile then
		assert(is_profile(profile), "Invaild msg profile")
	end
	return { role = role, body = body, profile = profile }
end

--- Guest message.
--- @param profile Profile
--- @param body Body | string Body or Text content
--- @return Msg
function talk.msg_guest(profile, body)
	return role_msg(talk.Role.Guest, body, profile)
end

--- Host message.
--- @param body Body | string Body or Text content
--- @return Msg
function talk.msg_host(body)
	return role_msg(talk.Role.Host, body)
end

--- System message.
--- @param body Body | string Body or Text content
--- @return Msg
function talk.msg_system(body)
	return role_msg(talk.Role.System, body)
end

--- Reply message.
--- @param body Body | string Body or Text content
--- @return Msg
function talk.msg_reply(body)
	return role_msg(talk.Role.Reply, body)
end

--- Bond story message.
--- @param body Body | string Body or Text content
--- @return Msg
function talk.msg_bond_story(body)
	return role_msg(talk.Role.BondStory, body)
end

--- Text body.
--- @param value Text | string Text or Text content
--- @return Body
function talk.body_text(value)
	if type(value) == "string" then
		value = { content = value }
	else
		assert(is_text(value), "Invaild body value as text")
	end
	return { type = talk.Type.Text, value = value }
end

--- Image body.
--- @param value Image
--- @return Body
function talk.body_image(value)
	assert(is_image(value), "Invaild body value as image")
	return { type = talk.Type.Image, value = value }
end

return talk
