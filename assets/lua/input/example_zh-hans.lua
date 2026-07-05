-- 示例输入文件。
-- 有关 "talk" 定义，见 `assets/lib/talk.lua`。

-- 虽然 `talk.lua` 默认加载到 LuaTalk，
-- 你仍可以生成或复制一份 `talk.lua` 到与本文件相同的目录，
-- 以便在 IDE 中获得代码补全和类型检查。
local talk = require("talk")

------------
--- 别名 ---
------------

local page = talk.page

-- 以下函数的所有 `body` 参数
-- 需要 `Body` 表或文本内容字符串。
-- `Body` 表格式为 { type = type, value = value }，
-- 其中 `type` 为 "text" 或 "image"，`value` 分别为 `Text` 表或 `Image` 表。

local guest = talk.msg_guest
local host = talk.msg_host
local system = talk.msg_system
local reply = talk.msg_reply
local bond_story = talk.msg_bond_story

-- `talk.body_text` 需要 `Text` 表或文本内容字符串。
-- `Text` 表格式为 { content = [[Example content]] }。

-- `talk.body_image` 需要 `Image` 表或路径字符串。
-- `Image` 表格式为 { path = "/path/to/image", url = "https://example.com/path/to/image" }。
-- 在 `Image` 表中，你可以只提供 `url` 或 `path`，也可以都提供。
-- `path` 是文件路径，可以是已存在或不存在的文件，或为 nil。
-- 如果文件不存在，LuaTalk 应该尝试通过 URL 获取。
-- 如果为 nil，LuaTalk 应将其设置为 `<dir>/<sha256>-<filename>` 作为备用，
-- 其中 <dir> 可为任意目录，<sha256> 为 URL 的哈希值，<filename> 为 URL 的文件名。
-- 不要将其设置为 nil，
-- 除非你确定你的文章不会导出到需要本地图片文件的任何文件中。
-- `url` 是图片的获取地址，可选。
-- 该字段主要用于某些导出格式可以使用图片 URL 而不是本地文件。
-- 你可以将其作为图片来源的补充信息标记，但不是替代项。

-- local text = talk.body_text -- 大多数情况下你可能不需要这个
local image = talk.body_image -- 请注意，这返回的是 `Body`，不是 `Image`

------------
--- 主体 ---
------------

-- 提示：你可以使用 func { ... } 语法代替 func({ ... })。

local her = { name = [[Her]], avatar = { path = "/path/to/image" } }
-- local her = { name = [[Her]], avatar = { path = "/path/to/image", url = "https://example.com/path/to/image" } }
-- 通常情况下不要使用下面这种方式
-- 除非你只导出你的文章到 `momotalk` 格式并上传到 `https://u1805.github.io/momotalk/`。
-- local her = { name = [[Her]], avatar = { url = "https://example.com/path/to/image" } }

return {
	lang = "zh-Hans", -- 可用选项：`en`, `ja`, `ko`, `zh-Hans`, `zh-Hant`
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
