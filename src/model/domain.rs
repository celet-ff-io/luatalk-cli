use std::{
    fs::{self, File},
    io,
    path::Path,
    sync::Arc,
};

use getset::Getters;
use log::debug;
use typed_builder::TypedBuilder;

use crate::net::agent;

#[derive(Debug, Clone, PartialEq, Eq, Getters, TypedBuilder)]
pub struct Article {
    pub(crate) lang: Lang,

    #[getset(get = "pub")]
    pub(crate) pages: Vec<Page>,
}

impl Article {
    #[inline]
    pub fn into_pages(self) -> Vec<Page> {
        self.pages
    }

    pub fn concat_pages(self) -> Article {
        let Self { lang, pages } = self;

        let pages = {
            let page = {
                let msgs = pages.into_iter().flatten().collect::<Vec<Msg>>();
                Page { msgs }
            };
            vec![page]
        };

        Self { lang, pages }
    }
}

impl InLang for Article {
    #[inline]
    fn lang(&self) -> Lang {
        self.lang
    }
}

pub trait InLang {
    fn lang(&self) -> Lang;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    En,
    Ja,
    Ko,
    ZhHans,
    ZhHant,
}

/// Value with a language tag
pub struct AndLang<T> {
    pub value: T,

    pub lang: Lang,
}

impl InLang for Lang {
    #[inline]
    fn lang(&self) -> Lang {
        *self
    }
}

pub trait IntoAndLang: Sized {
    #[inline]
    fn into_and_lang(self, lang: Lang) -> AndLang<Self> {
        AndLang { value: self, lang }
    }
}

impl<T> IntoAndLang for T {
    #[inline]
    fn into_and_lang(self, lang: Lang) -> AndLang<Self> {
        AndLang { value: self, lang }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, TypedBuilder)]
pub struct Page {
    #[getset(get = "pub")]
    pub(crate) msgs: Vec<Msg>,
}

impl Page {
    #[inline]
    pub fn into_msgs(self) -> Vec<Msg> {
        self.msgs
    }
}

impl IntoIterator for Page {
    type Item = Msg;
    type IntoIter = std::vec::IntoIter<Self::Item>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        self.msgs.into_iter()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, TypedBuilder)]
pub struct Msg {
    #[getset(get = "pub")]
    pub(crate) role: Role,

    #[getset(get = "pub")]
    pub(crate) body: Body,

    #[builder(default, setter(strip_option(fallback = profile_opt)))]
    pub(crate) profile: Option<Arc<Profile>>,
}

impl Msg {
    #[inline]
    pub fn profile(&self) -> Option<&Profile> {
        self.profile.as_deref()
    }

    #[inline]
    pub fn profile_shared(&self) -> Option<Arc<Profile>> {
        self.profile.clone()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Guest,
    Host,
    System,
    Reply,
    BondStory,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Body {
    Text(TextValue),
    Image(ImageValue),
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, TypedBuilder)]
pub struct TextValue {
    #[getset(get = "pub")]
    pub(crate) content: String,
}

impl TextValue {
    #[inline]
    pub fn into_content(self) -> String {
        self.content
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, TypedBuilder)]
pub struct ImageValue {
    #[getset(get = "pub")]
    pub(crate) path: String,

    #[getset(get = "pub")]
    #[builder(default)]
    pub(crate) url: Option<String>,
}

impl ImageValue {
    #[inline]
    pub fn into_path(self) -> String {
        self.path
    }

    #[inline]
    pub fn into_path_and_url(self) -> (String, Option<String>) {
        let Self { path, url } = self;
        (path, url)
    }

    /// Try to fetch the image from URL to the path
    /// if the file does not exist.
    pub fn try_ensure_path(&self) -> Result<(), ImageValueError> {
        let Self { path, url } = self;
        let path = Path::new(path);
        if !path.exists() {
            if let Some(parent) = path.parent()
                && !parent.exists()
            {
                fs::create_dir_all(parent)?;
                debug!("Created directory: {:?}", parent);
            };
            if let Some(url) = url {
                debug!("Fetch image from URL: {:?}", url);
                let mut reader = agent().get(url).call()?.into_body().into_reader();
                let mut writer = File::create(path)?;
                io::copy(&mut reader, &mut writer)?;
                debug!("Image and saved to path: {:?}", path);
            } else {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    format!("File not found when no URL provided: {:?}", path),
                )
                .into());
            }
        }
        Ok(())
    }

    pub fn data_url(&self) -> Option<String> {
        todo!()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ImageValueError {
    #[error("File system IO error occurred: {0}")]
    FsIoError(#[from] std::io::Error),
    #[error("Network IO error occurred: {0}")]
    NetIOError(#[from] ureq::Error),
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, TypedBuilder)]
pub struct Profile {
    #[getset(get = "pub")]
    pub(crate) name: String,

    #[getset(get = "pub")]
    pub(crate) avatar: ImageValue,
}
