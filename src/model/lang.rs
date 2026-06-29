pub trait InLang {
    fn lang(&self) -> Lang;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Lang {
    En,
    ZhCn,
}

/// Value with a language tag
pub struct AndLang<T> {
    pub value: T,

    pub lang: Lang,
}

impl InLang for Lang {
    fn lang(&self) -> Lang {
        *self
    }
}

pub trait IntoWithLang: Sized {
    fn into_with_lang(self, lang: Lang) -> AndLang<Self> {
        AndLang { value: self, lang }
    }
}

impl<T> IntoWithLang for T {
    fn into_with_lang(self, lang: Lang) -> AndLang<Self> {
        AndLang { value: self, lang }
    }
}
