use clap::Command;

pub trait Localize {
    fn localize(self, lang: SupportedLang) -> Self;
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum SupportedLang {
    #[default]
    En,
    ZhHans,
}

impl From<&str> for SupportedLang {
    fn from(lang: &str) -> Self {
        let lower_lang = lang.to_lowercase();
        let prefix = if let Some(prefix) = lower_lang.get(..2) {
            prefix
        } else {
            return SupportedLang::default();
        };
        use SupportedLang::*;
        match prefix {
            "en" => En,
            "zh" => ZhHans,
            _ => SupportedLang::default(),
        }
    }
}

impl Localize for Command {
    fn localize(self, lang: SupportedLang) -> Self {
        use SupportedLang::*;
        match lang {
            En => self.localize_en(),
            ZhHans => self.localize_zh_hans(),
        }
    }
}

trait CommandExt {
    fn localize_en(self) -> Self;
    fn localize_zh_hans(self) -> Self;
}

impl CommandExt for Command {
    #[inline]
    fn localize_en(self) -> Self {
        self
    }

    #[inline]
    fn localize_zh_hans(self) -> Self {
        // TODO
        self
    }
}
