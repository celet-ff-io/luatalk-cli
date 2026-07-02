//! This file is according to `talk.lua` as following.
#![doc = concat!(
    "```lua\n",
    include_str!("../../assets/lua/lib/talk.lua"),
    "```\n"
)]

use std::sync::Arc;

use mlua::{AsChunk, FromLua, Lua, LuaSerdeExt, Table, Value};
use serde::Deserialize;
use tap::Pipe;

use crate::{error::LuaParseError, model};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Article {
    pub lang: Lang,
    pub pages: Vec<Page>,
}

impl Article {
    pub fn try_from_chunk(chunk: impl AsChunk, lua: &Lua) -> Result<Self, LuaParseError> {
        lua.load(chunk)
            .eval::<Table>()
            .map_err(LuaParseError::Eval)?
            .pipe(Value::Table)
            .pipe(|v| Article::from_lua(v, lua))
            .map_err(LuaParseError::Deserialize)
    }
}

impl FromLua for Article {
    fn from_lua(value: Value, lua: &Lua) -> mlua::Result<Self> {
        lua.from_value(value)
    }
}

impl From<model::Article> for Article {
    fn from(article: model::Article) -> Self {
        let model::Article { lang, pages } = article;
        Article {
            lang: lang.into(),
            pages: pages.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Article> for model::Article {
    fn from(article: Article) -> Self {
        let Article { lang, pages } = article;
        model::Article {
            lang: lang.into(),
            pages: pages.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
pub enum Lang {
    #[serde(rename = "en")]
    En,

    #[serde(rename = "ja-JP")]
    JaJp,

    #[serde(rename = "zh-CN")]
    ZhCn,
}

impl From<model::Lang> for Lang {
    fn from(lang: model::Lang) -> Self {
        match lang {
            model::Lang::En => Lang::En,
            model::Lang::JaJp => Lang::JaJp,
            model::Lang::ZhCn => Lang::ZhCn,
        }
    }
}

impl From<Lang> for model::Lang {
    fn from(lang: Lang) -> Self {
        match lang {
            Lang::En => model::Lang::En,
            Lang::JaJp => model::Lang::JaJp,
            Lang::ZhCn => model::Lang::ZhCn,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Page {
    pub msgs: Vec<Msg>,
}

impl From<model::Page> for Page {
    fn from(page: model::Page) -> Self {
        let model::Page { msgs } = page;
        Page {
            msgs: msgs.into_iter().map(Into::into).collect(),
        }
    }
}

impl From<Page> for model::Page {
    fn from(page: Page) -> Self {
        let Page { msgs } = page;
        model::Page {
            msgs: msgs.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Msg {
    pub role: Role,
    pub body: Body,
    pub profile: Option<Profile>,
}

impl From<model::Msg> for Msg {
    fn from(msg: model::Msg) -> Self {
        let model::Msg {
            role,
            body,
            profile, // profile: Option<Arc<Profile>>
        } = msg;
        Msg {
            role: role.into(),
            body: body.into(),
            profile: profile.map(|p| p.as_ref().clone().into()),
        }
    }
}

impl From<Msg> for model::Msg {
    fn from(msg: Msg) -> Self {
        let Msg {
            role,
            body,
            profile,
        } = msg;
        model::Msg {
            role: role.into(),
            body: body.into(),
            profile: profile.map(Into::into).map(Arc::new),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Role {
    Guest,
    Host,
    System,
    Reply,
    BondStory,
}

impl From<model::Role> for Role {
    fn from(role: model::Role) -> Self {
        match role {
            model::Role::Guest => Role::Guest,
            model::Role::Host => Role::Host,
            model::Role::System => Role::System,
            model::Role::Reply => Role::Reply,
            model::Role::BondStory => Role::BondStory,
        }
    }
}

impl From<Role> for model::Role {
    fn from(role: Role) -> Self {
        match role {
            Role::Guest => model::Role::Guest,
            Role::Host => model::Role::Host,
            Role::System => model::Role::System,
            Role::Reply => model::Role::Reply,
            Role::BondStory => model::Role::BondStory,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Body {
    Text(TextValue),
    Image(ImageValue),
}

impl From<model::Body> for Body {
    fn from(body: model::Body) -> Self {
        match body {
            model::Body::Text(text_value) => Body::Text(text_value.into()),
            model::Body::Image(image_value) => Body::Image(image_value.into()),
        }
    }
}

impl From<Body> for model::Body {
    fn from(body: Body) -> Self {
        match body {
            Body::Text(text_value) => model::Body::Text(text_value.into()),
            Body::Image(image_value) => model::Body::Image(image_value.into()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct TextValue {
    pub content: String,
}

impl From<model::TextValue> for TextValue {
    fn from(text_value: model::TextValue) -> Self {
        let model::TextValue { content } = text_value;
        TextValue { content }
    }
}

impl From<TextValue> for model::TextValue {
    fn from(text_value: TextValue) -> Self {
        let TextValue { content } = text_value;
        model::TextValue { content }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ImageValue {
    pub url: String,
}

impl From<model::ImageValue> for ImageValue {
    fn from(image_value: model::ImageValue) -> Self {
        let model::ImageValue { url } = image_value;
        ImageValue { url }
    }
}

impl From<ImageValue> for model::ImageValue {
    fn from(image_value: ImageValue) -> Self {
        let ImageValue { url } = image_value;
        model::ImageValue { url }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Profile {
    pub name: String,
    pub avatar: ImageValue,
}

impl From<model::Profile> for Profile {
    fn from(profile: model::Profile) -> Self {
        let model::Profile { name, avatar } = profile;
        Profile {
            name,
            avatar: avatar.into(),
        }
    }
}

impl From<Profile> for model::Profile {
    fn from(profile: Profile) -> Self {
        let Profile { name, avatar } = profile;
        model::Profile {
            name,
            avatar: avatar.into(),
        }
    }
}
