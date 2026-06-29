//! This file is according to the export JSON file of [MomoTalk Editor](https://github.com/U1805/momotalk/).
//!
//! [MomoTalk Editor](https://github.com/U1805/momotalk/) is
//! a web chat generator, style from Blue Archive,
//! made by [U1805 (Dai0v0)](https://github.com/U1805).

use std::num::TryFromIntError;

use log::debug;
use serde::Serialize;
use serde_repr::Serialize_repr;
use tap::{Pipe, Tap};

use crate::model::{self, InLang, IntoAndLang};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct MomotalkExport {
    #[serde(rename = "talkId")]
    pub talk_id: i32,

    #[serde(rename = "talkHistory")]
    pub talk_history: Vec<TalkHistoryItem>,

    #[serde(rename = "selectList")]
    pub select_list: Vec<SelectListItem>,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize_repr)]
#[repr(i32)]
pub enum Type {
    Student = 0,
    Sensei = 1,
    Story = 2,
    Choice = 3,
    Message = 4,
}

type MsgList = Vec<model::Msg>;

impl TryFrom<model::AndLang<MsgList>> for Vec<TalkHistoryItem> {
    type Error = MomotalkExportError;

    fn try_from(msgs: model::AndLang<MsgList>) -> Result<Self, Self::Error> {
        let model::AndLang { value: msgs, lang } = msgs;
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
                            model::Lang::En => "Reply",
                            model::Lang::ZhCn => "回复",
                        }
                        .to_owned();
                        avatar = String::new();
                        content = body.try_into_text()?;
                    }
                    model::Role::BondStory => {
                        type_ = Type::Story;
                        name = match lang {
                            model::Lang::En => "Story Event",
                            model::Lang::ZhCn => "羁绊剧情",
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
pub struct SelectListItem {
    #[serde(rename = "Id")]
    pub id: i32,

    #[serde(rename = "Name")]
    pub name: String,

    #[serde(rename = "Avatar")]
    pub avatar: String,
}

impl TryFrom<model::Article> for Vec<MomotalkExport> {
    type Error = MomotalkExportError;

    fn try_from(value: model::Article) -> Result<Self, Self::Error> {
        let article = value;
        let lang = article.lang();
        article
            .into_pages()
            .into_iter()
            .enumerate()
            .map(|(i, page)| -> Result<MomotalkExport, MomotalkExportError> {
                let n = (i + 1)
                    .pipe(i32::try_from)
                    .map_err(MomotalkExportError::TryFromInt)?;
                let talk_history = page
                    .into_iter()
                    .collect::<Vec<model::Msg>>()
                    .into_and_lang(lang)
                    .try_into()?;
                let select_list = Vec::new();
                MomotalkExport {
                    talk_id: n,
                    talk_history,
                    select_list,
                }
                .tap(|_| {
                    debug!("Build MomotalkExport structure success for page {n}");
                })
                .pipe(Ok)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use miette::{IntoDiagnostic, Result};
    use pretty_assertions::assert_eq;

    use crate::{IntoAndLang, Page};

    use super::{
        model::{Body, ImageValue, Msg, Profile, Role, TextValue},
        *,
    };

    #[test]
    fn into_content() {
        let text_body = model::Body::Text(model::TextValue {
            content: "Hello".to_owned(),
        });
        assert_eq!(text_body.into_content(), "Hello");

        let image_body = model::Body::Image(model::ImageValue {
            url: "http://example.com/image.png".to_owned(),
        });
        assert_eq!(image_body.into_content(), "http://example.com/image.png");
    }

    #[test]
    fn try_into_text() {
        let text_body = model::Body::Text(model::TextValue {
            content: "Hello".to_owned(),
        });
        assert_eq!(text_body.try_into_text().unwrap(), "Hello");

        let image_body = model::Body::Image(model::ImageValue {
            url: "http://example.com/image.png".to_owned(),
        });
        assert!(image_body.try_into_text().is_err());
    }

    #[test]
    fn try_from_and_lang_msg_list() -> Result<()> {
        let her = Profile::builder()
            .name("Her".to_owned())
            .avatar(
                ImageValue::builder()
                    .url("<placeholder-0>".to_owned())
                    .build(),
            )
            .build()
            .pipe(Arc::new);
        let msgs = vec![
            Msg::builder()
                .role(Role::Guest)
                .body(Body::Text(
                    TextValue::builder()
                        .content("Example guest message".to_owned())
                        .build(),
                ))
                .profile(her.pipe_ref(Arc::clone))
                .build(),
            Msg::builder()
                .role(Role::Guest)
                .body(Body::Image(
                    ImageValue::builder()
                        .url("<placeholder-1>".to_owned())
                        .build(),
                ))
                .profile(her.pipe_ref(Arc::clone))
                .build(),
            Msg::builder()
                .role(Role::Host)
                .body(Body::Text(
                    TextValue::builder()
                        .content("Example host message".to_owned())
                        .build(),
                ))
                .build(),
            Msg::builder()
                .role(Role::Host)
                .body(Body::Image(
                    ImageValue::builder()
                        .url("<placeholder-2>".to_owned())
                        .build(),
                ))
                .build(),
            Msg::builder()
                .role(Role::System)
                .body(Body::Text(
                    TextValue::builder()
                        .content("Example system message".to_owned())
                        .build(),
                ))
                .build(),
            Msg::builder()
                .role(Role::Reply)
                .body(Body::Text(
                    TextValue::builder()
                        .content("Example reply message".to_owned())
                        .build(),
                ))
                .build(),
            Msg::builder()
                .role(Role::BondStory)
                .body(Body::Text(
                    TextValue::builder()
                        .content("Example bond story message".to_owned())
                        .build(),
                ))
                .build(),
        ];

        let got: Vec<TalkHistoryItem> = msgs.into_and_lang(model::Lang::En).try_into().unwrap();

        let expected = vec![
            TalkHistoryItem {
                id: 1,
                name: "Her".to_owned(),
                avatar: "<placeholder-0>".to_owned(),
                type_: Type::Student,
                flag: 0,
                content: "Example guest message".to_owned(),
            },
            TalkHistoryItem {
                id: 2,
                name: "Her".to_owned(),
                avatar: "<placeholder-0>".to_owned(),
                type_: Type::Student,
                flag: 0,
                content: "<placeholder-1>".to_owned(),
            },
            TalkHistoryItem {
                id: 3,
                name: "sensei".to_owned(),
                avatar: String::new(),
                type_: Type::Sensei,
                flag: 0,
                content: "Example host message".to_owned(),
            },
            TalkHistoryItem {
                id: 4,
                name: "sensei".to_owned(),
                avatar: String::new(),
                type_: Type::Sensei,
                flag: 0,
                content: "<placeholder-2>".to_owned(),
            },
            TalkHistoryItem {
                id: 5,
                name: "systemInfo".to_owned(),
                avatar: String::new(),
                type_: Type::Message,
                flag: 0,
                content: "Example system message".to_owned(),
            },
            TalkHistoryItem {
                id: 6,
                name: "Reply".to_owned(),
                avatar: String::new(),
                type_: Type::Choice,
                flag: 0,
                content: "Example reply message".to_owned(),
            },
            TalkHistoryItem {
                id: 7,
                name: "Story Event".to_owned(),
                avatar: String::new(),
                type_: Type::Story,
                flag: 0,
                content: "Example bond story message".to_owned(),
            },
        ];

        assert_eq!(got, expected);

        Ok(())
    }

    #[test]
    fn try_from_article() -> Result<()> {
        let her = Profile::builder()
            .name("Her".to_owned())
            .avatar(
                ImageValue::builder()
                    .url("<placeholder-0>".to_owned())
                    .build(),
            )
            .build()
            .pipe(Arc::new);
        let article = model::Article::builder()
            .lang(model::Lang::En)
            .pages(vec![
                Page::builder()
                    .msgs(vec![
                        Msg::builder()
                            .role(Role::Guest)
                            .body(Body::Text(
                                TextValue::builder()
                                    .content("Example guest message".to_owned())
                                    .build(),
                            ))
                            .profile(her.pipe_ref(Arc::clone))
                            .build(),
                        Msg::builder()
                            .role(Role::Guest)
                            .body(Body::Image(
                                ImageValue::builder()
                                    .url("<placeholder-1>".to_owned())
                                    .build(),
                            ))
                            .profile(her.pipe_ref(Arc::clone))
                            .build(),
                        Msg::builder()
                            .role(Role::Host)
                            .body(Body::Text(
                                TextValue::builder()
                                    .content("Example host message".to_owned())
                                    .build(),
                            ))
                            .build(),
                        Msg::builder()
                            .role(Role::Host)
                            .body(Body::Image(
                                ImageValue::builder()
                                    .url("<placeholder-2>".to_owned())
                                    .build(),
                            ))
                            .build(),
                    ])
                    .build(),
                Page::builder()
                    .msgs(vec![
                        Msg::builder()
                            .role(Role::System)
                            .body(Body::Text(
                                TextValue::builder()
                                    .content("Example system message".to_owned())
                                    .build(),
                            ))
                            .build(),
                        Msg::builder()
                            .role(Role::Reply)
                            .body(Body::Text(
                                TextValue::builder()
                                    .content("Example reply message".to_owned())
                                    .build(),
                            ))
                            .build(),
                        Msg::builder()
                            .role(Role::BondStory)
                            .body(Body::Text(
                                TextValue::builder()
                                    .content("Example bond story message".to_owned())
                                    .build(),
                            ))
                            .build(),
                    ])
                    .build(),
            ])
            .build();

        let got: Vec<MomotalkExport> = article.try_into().into_diagnostic()?;

        let expected = vec![
            MomotalkExport {
                talk_id: 1,
                talk_history: vec![
                    TalkHistoryItem {
                        id: 1,
                        name: "Her".to_owned(),
                        avatar: "<placeholder-0>".to_owned(),
                        type_: Type::Student,
                        flag: 0,
                        content: "Example guest message".to_owned(),
                    },
                    TalkHistoryItem {
                        id: 2,
                        name: "Her".to_owned(),
                        avatar: "<placeholder-0>".to_owned(),
                        type_: Type::Student,
                        flag: 0,
                        content: "<placeholder-1>".to_owned(),
                    },
                    TalkHistoryItem {
                        id: 3,
                        name: "sensei".to_owned(),
                        avatar: String::new(),
                        type_: Type::Sensei,
                        flag: 0,
                        content: "Example host message".to_owned(),
                    },
                    TalkHistoryItem {
                        id: 4,
                        name: "sensei".to_owned(),
                        avatar: String::new(),
                        type_: Type::Sensei,
                        flag: 0,
                        content: "<placeholder-2>".to_owned(),
                    },
                ],
                select_list: Vec::new(),
            },
            MomotalkExport {
                talk_id: 2,
                talk_history: vec![
                    TalkHistoryItem {
                        id: 1,
                        name: "systemInfo".to_owned(),
                        avatar: String::new(),
                        type_: Type::Message,
                        flag: 0,
                        content: "Example system message".to_owned(),
                    },
                    TalkHistoryItem {
                        id: 2,
                        name: "Reply".to_owned(),
                        avatar: String::new(),
                        type_: Type::Choice,
                        flag: 0,
                        content: "Example reply message".to_owned(),
                    },
                    TalkHistoryItem {
                        id: 3,
                        name: "Story Event".to_owned(),
                        avatar: String::new(),
                        type_: Type::Story,
                        flag: 0,
                        content: "Example bond story message".to_owned(),
                    },
                ],
                select_list: Vec::new(),
            },
        ];

        assert_eq!(got, expected);

        Ok(())
    }
}
