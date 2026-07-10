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

use crate::model::domain::{self, InLang, IntoAndLang};

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

type MsgList = Vec<domain::Msg>;

impl TryFrom<domain::AndLang<MsgList>> for Vec<TalkHistoryItem> {
    type Error = MomotalkExportError;

    fn try_from(msgs: domain::AndLang<MsgList>) -> Result<Self, Self::Error> {
        let domain::AndLang { value: msgs, lang } = msgs;
        msgs.into_iter()
            .enumerate()
            .map(|(i, msg)| -> Result<TalkHistoryItem, MomotalkExportError> {
                let id = (i + 1)
                    .pipe(i32::try_from)
                    .map_err(MomotalkExportError::TryFromInt)?;
                let name;
                let avatar: String;
                let type_;
                let flag = 0;
                let content;

                let domain::Msg {
                    role,
                    body,
                    profile,
                } = msg;
                match role {
                    domain::Role::Guest => {
                        type_ = Type::Student;
                        match profile {
                            Some(profile) => {
                                name = profile.name.clone();
                                avatar = profile.avatar.url_or_data_url()?;
                            }
                            None => {
                                name = String::new();
                                avatar = String::new();
                            }
                        }
                        content = body.try_into_content()?;
                    }
                    domain::Role::Host => {
                        type_ = Type::Sensei;
                        name = "sensei".to_owned();
                        avatar = String::new();
                        content = body.try_into_content()?;
                    }
                    domain::Role::System => {
                        type_ = Type::Message;
                        name = "systemInfo".to_owned();
                        avatar = String::new();
                        content = body.try_into_text()?;
                    }
                    domain::Role::Reply => {
                        type_ = Type::Choice;
                        name = match lang {
                            domain::Lang::En => "Reply",
                            domain::Lang::Ja => "返信する",
                            domain::Lang::Ko => "답장",
                            domain::Lang::ZhHans => "回复",
                            domain::Lang::ZhHant => "回覆",
                        }
                        .to_owned();
                        avatar = String::new();
                        content = body.try_into_text()?;
                    }
                    domain::Role::BondStory => {
                        type_ = Type::Story;
                        name = match lang {
                            domain::Lang::En => "Story Event",
                            domain::Lang::Ja => "絆イベント",
                            domain::Lang::Ko => "이야기 이벤트",
                            domain::Lang::ZhHans => "羁绊剧情",
                            domain::Lang::ZhHant => "羈絆劇情",
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

trait TryIntoText {
    fn try_into_text(self) -> Result<String, MomotalkExportError>;
}

impl TryIntoText for domain::Body {
    fn try_into_text(self) -> Result<String, MomotalkExportError> {
        match self {
            domain::Body::Text(text) => Ok(text.into_content()),
            domain::Body::Image(..) => Err(MomotalkExportError::InvaildContentType),
        }
    }
}

trait TryIntoContent {
    fn try_into_content(self) -> Result<String, MomotalkExportError>;
}

impl TryIntoContent for domain::Body {
    fn try_into_content(self) -> Result<String, MomotalkExportError> {
        match self {
            domain::Body::Text(text) => text.into_content(),
            domain::Body::Image(image) => image.url_or_data_url()?,
        }
        .pipe(Ok)
    }
}

trait ImageValueExt {
    fn url_or_data_url(&self) -> Result<String, MomotalkExportError>;
}

impl ImageValueExt for domain::ImageValue {
    fn url_or_data_url(&self) -> Result<String, MomotalkExportError> {
        if let Some(url) = self.url() {
            return url.clone().pipe(Ok);
        }
        debug!(
            "ImageValue has no URL, try to generate data URL from path: {:?}",
            self.path()
        );
        self.try_generate_data_url()?.pipe(Ok)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MomotalkExportError {
    #[error("MsgList length is too big for i32 Id for TalkHistoryItem: {0}")]
    TryFromInt(#[from] TryFromIntError),

    #[error("Require text while body type is image")]
    InvaildContentType,

    #[error("ImageValue error: {0}")]
    ImageValueError(#[from] domain::Error),
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

impl TryFrom<domain::Article> for Vec<MomotalkExport> {
    type Error = MomotalkExportError;

    fn try_from(value: domain::Article) -> Result<Self, Self::Error> {
        let article = value;
        let lang = article.lang();
        article
            .into_pages()
            .pipe(|pages| {
                pages
                    .len()
                    .pipe(i32::try_from)
                    .map_err(MomotalkExportError::TryFromInt)
                    .map(|_| pages)
            })?
            .into_iter()
            .zip(1..)
            .map(|(page, i)| -> Result<MomotalkExport, MomotalkExportError> {
                let talk_history = page
                    .into_iter()
                    .collect::<Vec<domain::Msg>>()
                    .into_and_lang(lang)
                    .try_into()?;
                let select_list = Vec::new();
                MomotalkExport {
                    talk_id: i,
                    talk_history,
                    select_list,
                }
                .tap(|_| {
                    debug!("Build MomotalkExport structure success for page {i}");
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
        domain::{Body, ImageValue, Msg, Profile, Role, TextValue},
        *,
    };

    #[test]
    fn try_into_content() -> Result<()> {
        let text_body = Body::Text(TextValue::builder().content("Hello".to_owned()).build());
        assert_eq!(text_body.try_into_content().into_diagnostic()?, "Hello");

        let image_body = Body::Image(
            ImageValue::builder()
                .path("/path/to/image.webp".to_owned())
                .url("https://example.com/image.webp".to_owned())
                .build(),
        );
        assert_eq!(
            image_body.try_into_content().into_diagnostic()?,
            "https://example.com/image.webp"
        );

        Ok(())
    }

    #[test]
    fn try_into_text() {
        let text_body = Body::Text(TextValue {
            content: "Hello".to_owned(),
        });
        assert_eq!(text_body.try_into_text().unwrap(), "Hello");

        let image_body = Body::Image(
            ImageValue::builder()
                .path("/path/to/image.webp".to_owned())
                .url("https://example.com/image.webp".to_owned())
                .build(),
        );
        assert!(image_body.try_into_text().is_err());
    }

    #[test]
    fn try_from_and_lang_msg_list() -> Result<()> {
        let her = Profile::builder()
            .name("Her".to_owned())
            .avatar(
                ImageValue::builder()
                    .path("/path/to/image.webp".to_owned())
                    .url("https://example.com/image.webp".to_owned())
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
                        .path("/path/to/image.webp".to_owned())
                        .url("https://example.com/image.webp".to_owned())
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
                        .path("/path/to/image.webp".to_owned())
                        .url("https://example.com/image.webp".to_owned())
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

        let got: Vec<TalkHistoryItem> = msgs.into_and_lang(domain::Lang::En).try_into().unwrap();

        let expected = vec![
            TalkHistoryItem {
                id: 1,
                name: "Her".to_owned(),
                avatar: "https://example.com/image.webp".to_owned(),
                type_: Type::Student,
                flag: 0,
                content: "Example guest message".to_owned(),
            },
            TalkHistoryItem {
                id: 2,
                name: "Her".to_owned(),
                avatar: "https://example.com/image.webp".to_owned(),
                type_: Type::Student,
                flag: 0,
                content: "https://example.com/image.webp".to_owned(),
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
                content: "https://example.com/image.webp".to_owned(),
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
                    .path("/path/to/image".to_owned())
                    .url("https://example.com/image.webp".to_owned())
                    .build(),
            )
            .build()
            .pipe(Arc::new);
        let article = domain::Article::builder()
            .lang(domain::Lang::En)
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
                                    .path("/path/to/image.webp".to_owned())
                                    .url("https://example.com/image.webp".to_owned())
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
                                    .path("/path/to/image".to_owned())
                                    .url("https://example.com/image.webp".to_owned())
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
                        avatar: "https://example.com/image.webp".to_owned(),
                        type_: Type::Student,
                        flag: 0,
                        content: "Example guest message".to_owned(),
                    },
                    TalkHistoryItem {
                        id: 2,
                        name: "Her".to_owned(),
                        avatar: "https://example.com/image.webp".to_owned(),
                        type_: Type::Student,
                        flag: 0,
                        content: "https://example.com/image.webp".to_owned(),
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
                        content: "https://example.com/image.webp".to_owned(),
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
