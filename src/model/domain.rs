use std::sync::Arc;

use getset::Getters;
use typed_builder::TypedBuilder;

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

    #[builder(default, setter(strip_option))]
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

#[derive(Debug, Clone, PartialEq, Eq, Getters, TypedBuilder)]
pub struct ImageValue {
    #[getset(get = "pub")]
    pub(crate) url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, TypedBuilder)]
pub struct Profile {
    #[getset(get = "pub")]
    pub(crate) name: String,

    #[getset(get = "pub")]
    pub(crate) avatar: ImageValue,
}
