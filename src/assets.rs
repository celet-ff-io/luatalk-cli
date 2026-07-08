use include_flate::flate;

pub mod lua {
    use super::*;

    pub mod input {
        use super::*;

        flate!(static EXAMPLE_EN: str from "./assets/lua/input/example_en.lua");
        flate!(static EXAMPLE_ZH_HANS: str from "./assets/lua/input/example_zh-hans.lua");

        #[inline]
        pub fn example_en() -> &'static str {
            &EXAMPLE_EN
        }

        #[inline]
        pub fn example_zh_hans() -> &'static str {
            &EXAMPLE_ZH_HANS
        }
    }

    // We need them updload quickly
    pub mod lib {
        const TALK: &str = include_str!("../assets/lua/lib/talk.lua");

        #[inline]
        pub fn talk() -> &'static str {
            TALK
        }
    }
}

pub mod typst {
    use super::*;

    flate!(static OUTPUT: str from "./assets/typst/output.typ");

    #[inline]
    pub fn output() -> &'static str {
        &OUTPUT
    }
}
