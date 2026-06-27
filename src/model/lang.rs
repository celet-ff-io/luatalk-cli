/// Value with a language tag
pub struct WithLang<T> {
    pub value: T,

    pub lang: Lang,
}

pub enum Lang {
    En,
    Zh,
}

pub trait IntoWithLang: Sized {
    fn into_with_lang(self, lang: Lang) -> WithLang<Self> {
        WithLang { value: self, lang }
    }
}

impl<T> IntoWithLang for T {
    fn into_with_lang(self, lang: Lang) -> WithLang<Self> {
        WithLang { value: self, lang }
    }
}
