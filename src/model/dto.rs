//! This file is according to `talk.lua` as following.
#![doc = concat!(
    "```lua\n",
    include_str!("../../assets/lua/lib/talk.lua"),
    "```\n"
)]

use std::sync::Arc;

use data_encoding::BASE32HEX_NOPAD;
use mlua::{AsChunk, FromLua, Lua, LuaSerdeExt, Table, Value};
use serde::{Deserialize, Serialize};
use tap::Pipe;
use url::Url;

use crate::{error::LuaParseError, model};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl TryFrom<Article> for model::Article {
    type Error = DtoError;

    fn try_from(article: Article) -> Result<Self, Self::Error> {
        let Article { lang, pages } = article;
        model::Article {
            lang: lang.into(),
            pages: pages
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<model::Page>, Self::Error>>()?,
        }
        .pipe(Ok)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Lang {
    #[serde(rename = "en")]
    En,

    #[serde(rename = "ja")]
    Ja,

    #[serde(rename = "ko")]
    Ko,

    #[serde(rename = "zh-Hans")]
    ZhHans,

    #[serde(rename = "zh-Hant")]
    ZhHant,
}

impl From<model::Lang> for Lang {
    fn from(lang: model::Lang) -> Self {
        match lang {
            model::Lang::En => Lang::En,
            model::Lang::Ja => Lang::Ja,
            model::Lang::Ko => Lang::Ko,
            model::Lang::ZhHans => Lang::ZhHans,
            model::Lang::ZhHant => Lang::ZhHant,
        }
    }
}

impl From<Lang> for model::Lang {
    fn from(lang: Lang) -> Self {
        match lang {
            Lang::En => model::Lang::En,
            Lang::Ja => model::Lang::Ja,
            Lang::Ko => model::Lang::Ko,
            Lang::ZhHans => model::Lang::ZhHans,
            Lang::ZhHant => model::Lang::ZhHant,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl TryFrom<Page> for model::Page {
    type Error = DtoError;

    fn try_from(page: Page) -> Result<Self, Self::Error> {
        let Page { msgs } = page;
        model::Page {
            msgs: msgs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<model::Msg>, Self::Error>>()?,
        }
        .pipe(Ok)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl TryFrom<Msg> for model::Msg {
    type Error = DtoError;

    fn try_from(msg: Msg) -> Result<Self, Self::Error> {
        let Msg {
            role,
            body,
            profile,
        } = msg;
        let profile = if let Some(profile) = profile {
            Some(profile.try_into()?)
        } else {
            None
        }
        .map(Arc::new);
        model::Msg {
            role: role.into(),
            body: body.try_into()?,
            profile,
        }
        .pipe(Ok)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl TryFrom<Body> for model::Body {
    type Error = DtoError;

    fn try_from(body: Body) -> Result<Self, Self::Error> {
        match body {
            Body::Text(text_value) => model::Body::Text(text_value.into()),
            Body::Image(image_value) => model::Body::Image(image_value.try_into()?),
        }
        .pipe(Ok)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageValue {
    pub path: Option<String>,
    pub url: Option<String>,
}

impl From<model::ImageValue> for ImageValue {
    fn from(image_value: model::ImageValue) -> Self {
        let model::ImageValue { path, url } = image_value;
        ImageValue {
            path: Some(path),
            url,
        }
    }
}

impl TryFrom<ImageValue> for model::ImageValue {
    type Error = DtoError;
    fn try_from(image_value: ImageValue) -> Result<Self, Self::Error> {
        let ImageValue { path, url } = image_value;
        let path = if let Some(path) = path {
            path
        } else {
            let url: &str = url.as_ref().ok_or(DtoError::NeitherPathNorUrl)?;
            let prefix = BASE32HEX_NOPAD.encode(&url.as_bytes());
            if let Some(filename) = url.pipe(Url::parse).ok().pipe(|url| {
                let seg = url?.path_segments()?.last()?.to_owned();
                if seg.is_empty() { None } else { Some(seg) }
            }) {
                format!("{prefix}-{filename}")
            } else {
                format!("{prefix}")
            }
        };
        model::ImageValue { path, url }.pipe(Ok)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

impl TryFrom<Profile> for model::Profile {
    type Error = DtoError;

    fn try_from(profile: Profile) -> Result<Self, Self::Error> {
        let Profile { name, avatar } = profile;
        model::Profile {
            name,
            avatar: avatar.try_into()?,
        }
        .pipe(Ok)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DtoError {
    #[error("Neither path nor url is provided")]
    NeitherPathNorUrl,
}
