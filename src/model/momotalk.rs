//! This file is according to the export JSON file of [MomoTalk Editor](https://github.com/U1805/momotalk/)

use std::num::TryFromIntError;

use serde::Serialize;
use tap::Pipe;

use crate::model::{
    self,
    lang::{Lang, WithLang},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MomotalkExport {
    #[serde(rename = "talkId")]
    pub talk_id: i32,

    #[serde(rename = "talkHistory")]
    pub talk_history: Vec<TalkHistoryItem>,

    #[serde(rename = "selectList")]
    pub select_list: Vec<SelectListItem>,
}

type MsgList = Vec<model::Msg>;

impl TryFrom<WithLang<MsgList>> for Vec<TalkHistoryItem> {
    type Error = MomotalkExportError;

    fn try_from(msgs: WithLang<MsgList>) -> Result<Self, MomotalkExportError> {
        let WithLang { value: msgs, lang } = msgs;
        msgs.into_iter()
            .enumerate()
            .map(|(i, msg)| -> Result<TalkHistoryItem, MomotalkExportError> {
                let id = (i + 1)
                    .pipe(i32::try_from)
                    .map_err(MomotalkExportError::TryFromInt)?;
                let name;
                let avatar;
                let type_;
                let flag = 0;
                let content;

                let model::Msg {
                    role,
                    body,
                    profile,
                } = msg;
                match role {
                    model::Role::Guest => {
                        type_ = Type::Student;
                        match profile {
                            Some(profile) => {
                                name = profile.name.clone();
                                avatar = profile.avatar.url.clone();
                            }
                            None => {
                                name = String::new();
                                avatar = String::new();
                            }
                        }
                        content = body.into_content();
                    }
                    model::Role::Host => {
                        type_ = Type::Sensei;
                        name = "sensei".to_owned();
                        avatar = String::new();
                        content = body.into_content();
                    }
                    model::Role::System => {
                        type_ = Type::Message;
                        name = "systemInfo".to_owned();
                        avatar = String::new();
                        content = body.try_into_text()?;
                    }
                    model::Role::Reply => {
                        type_ = Type::Choice;
                        name = match lang {
                            Lang::En => "Reply",
                            Lang::Zh => "回复",
                        }
                        .to_owned();
                        avatar = String::new();
                        content = body.try_into_text()?;
                    }
                    model::Role::BondStory => {
                        type_ = Type::Story;
                        name = match lang {
                            Lang::En => "Story Event",
                            Lang::Zh => "羁绊剧情",
                        }
                        .to_owned();
                        avatar = String::new();
                        content = body.try_into_text()?;
                    }
                };

                TalkHistoryItem {
                    id,
                    name,
                    avatar,
                    type_,
                    flag,
                    content,
                }
                .pipe(Ok)
            })
            .collect()
    }
}

trait IntoContent {
    fn into_content(self) -> String;
}

impl IntoContent for model::Body {
    fn into_content(self) -> String {
        match self {
            model::Body::Text(text) => text.content,
            model::Body::Image(image) => image.url,
        }
    }
}

trait TryIntoText {
    fn try_into_text(self) -> Result<String, MomotalkExportError>;
}

impl TryIntoText for model::Body {
    fn try_into_text(self) -> Result<String, MomotalkExportError> {
        match self {
            model::Body::Text(text) => Ok(text.content),
            model::Body::Image(..) => Err(MomotalkExportError::InvaildContentType),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MomotalkExportError {
    #[error("MsgList length is too big for i32 Id for TalkHistoryItem: {0}")]
    TryFromInt(#[from] TryFromIntError),

    #[error("Require text while body type is image")]
    InvaildContentType,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TalkHistoryItem {
    #[serde(rename = "Id")]
    pub id: i32,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Avatar")]
    pub avatar: String,

    #[serde(rename = "type")]
    pub type_: Type,

    #[serde(rename = "flag")]
    pub flag: i32,

    #[serde(rename = "content")]
    pub content: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum Type {
    Student = 0,
    Sensei = 1,
    Story = 2,
    Choice = 3,
    Message = 4,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SelectListItem {
    #[serde(rename = "Id")]
    pub id: i32,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Avatar")]
    pub avatar: String,
}
