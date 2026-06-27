pub mod lua;
pub mod momotalk;

use std::sync::Arc;

use getset::Getters;
use typed_builder::TypedBuilder;

#[derive(Debug, Clone, PartialEq, Eq, Getters, TypedBuilder)]
pub struct Article {
    #[getset(get = "pub")]
    pub(crate) pages: Vec<Page>,
}

#[derive(Debug, Clone, PartialEq, Eq, Getters, TypedBuilder)]
pub struct Page {
    #[getset(get = "pub")]
    pub(crate) msgs: Vec<Msg>,
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
