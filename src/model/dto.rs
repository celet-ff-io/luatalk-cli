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

use crate::{error::LuaParseError, model::domain};

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

impl From<domain::Article> for Article {
    fn from(article: domain::Article) -> Self {
        let domain::Article { lang, pages } = article;
        Article {
            lang: lang.into(),
            pages: pages.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<Article> for domain::Article {
    type Error = DtoError;

    fn try_from(article: Article) -> Result<Self, Self::Error> {
        let Article { lang, pages } = article;
        domain::Article {
            lang: lang.into(),
            pages: pages
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<domain::Page>, Self::Error>>()?,
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

impl From<domain::Lang> for Lang {
    fn from(lang: domain::Lang) -> Self {
        match lang {
            domain::Lang::En => Lang::En,
            domain::Lang::Ja => Lang::Ja,
            domain::Lang::Ko => Lang::Ko,
            domain::Lang::ZhHans => Lang::ZhHans,
            domain::Lang::ZhHant => Lang::ZhHant,
        }
    }
}

impl From<Lang> for domain::Lang {
    fn from(lang: Lang) -> Self {
        match lang {
            Lang::En => domain::Lang::En,
            Lang::Ja => domain::Lang::Ja,
            Lang::Ko => domain::Lang::Ko,
            Lang::ZhHans => domain::Lang::ZhHans,
            Lang::ZhHant => domain::Lang::ZhHant,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Page {
    pub msgs: Vec<Msg>,
}

impl From<domain::Page> for Page {
    fn from(page: domain::Page) -> Self {
        let domain::Page { msgs } = page;
        Page {
            msgs: msgs.into_iter().map(Into::into).collect(),
        }
    }
}

impl TryFrom<Page> for domain::Page {
    type Error = DtoError;

    fn try_from(page: Page) -> Result<Self, Self::Error> {
        let Page { msgs } = page;
        domain::Page {
            msgs: msgs
                .into_iter()
                .map(TryInto::try_into)
                .collect::<Result<Vec<domain::Msg>, Self::Error>>()?,
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

impl From<domain::Msg> for Msg {
    fn from(msg: domain::Msg) -> Self {
        let domain::Msg {
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

impl TryFrom<Msg> for domain::Msg {
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
        domain::Msg {
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

impl From<domain::Role> for Role {
    fn from(role: domain::Role) -> Self {
        match role {
            domain::Role::Guest => Role::Guest,
            domain::Role::Host => Role::Host,
            domain::Role::System => Role::System,
            domain::Role::Reply => Role::Reply,
            domain::Role::BondStory => Role::BondStory,
        }
    }
}

impl From<Role> for domain::Role {
    fn from(role: Role) -> Self {
        match role {
            Role::Guest => domain::Role::Guest,
            Role::Host => domain::Role::Host,
            Role::System => domain::Role::System,
            Role::Reply => domain::Role::Reply,
            Role::BondStory => domain::Role::BondStory,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "snake_case")]
pub enum Body {
    Text(TextValue),
    Image(ImageValue),
}

impl From<domain::Body> for Body {
    fn from(body: domain::Body) -> Self {
        match body {
            domain::Body::Text(text_value) => Body::Text(text_value.into()),
            domain::Body::Image(image_value) => Body::Image(image_value.into()),
        }
    }
}

impl TryFrom<Body> for domain::Body {
    type Error = DtoError;

    fn try_from(body: Body) -> Result<Self, Self::Error> {
        match body {
            Body::Text(text_value) => domain::Body::Text(text_value.into()),
            Body::Image(image_value) => domain::Body::Image(image_value.try_into()?),
        }
        .pipe(Ok)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TextValue {
    pub content: String,
}

impl From<domain::TextValue> for TextValue {
    fn from(text_value: domain::TextValue) -> Self {
        let domain::TextValue { content } = text_value;
        TextValue { content }
    }
}

impl From<TextValue> for domain::TextValue {
    fn from(text_value: TextValue) -> Self {
        let TextValue { content } = text_value;
        domain::TextValue { content }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageValue {
    pub path: Option<String>,
    pub url: Option<String>,
}

impl From<domain::ImageValue> for ImageValue {
    fn from(image_value: domain::ImageValue) -> Self {
        let domain::ImageValue { path, url } = image_value;
        ImageValue {
            path: Some(path),
            url,
        }
    }
}

impl TryFrom<ImageValue> for domain::ImageValue {
    type Error = DtoError;
    fn try_from(image_value: ImageValue) -> Result<Self, Self::Error> {
        let ImageValue { path, url } = image_value;
        let path = if let Some(path) = path {
            path
        } else {
            let url: &str = url.as_ref().ok_or(DtoError::NeitherPathNorUrl)?;
            let prefix = BASE32HEX_NOPAD.encode(url.as_bytes());
            if let Some(filename) = url.pipe(Url::parse).ok().pipe(|url| {
                let seg = url?.path_segments()?.next_back()?.to_owned();
                if seg.is_empty() { None } else { Some(seg) }
            }) {
                format!("{prefix}-{filename}")
            } else {
                prefix
            }
        };
        domain::ImageValue { path, url }.pipe(Ok)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Profile {
    pub name: String,
    pub avatar: ImageValue,
}

impl From<domain::Profile> for Profile {
    fn from(profile: domain::Profile) -> Self {
        let domain::Profile { name, avatar } = profile;
        Profile {
            name,
            avatar: avatar.into(),
        }
    }
}

impl TryFrom<Profile> for domain::Profile {
    type Error = DtoError;

    fn try_from(profile: Profile) -> Result<Self, Self::Error> {
        let Profile { name, avatar } = profile;
        domain::Profile {
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
