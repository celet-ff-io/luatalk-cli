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

pub mod license {
    use super::*;

    flate!(static NOTICE: str from "./assets/license/NOTICE");
    flate!(static LICENSE_APACHE: str from "./assets/license/LICENSE-APACHE");
    flate!(static LICENSE_MIT: str from "./assets/license/LICENSE-MIT");
    flate!(static LICENSE_HTML: str from "./assets/license/license.html");

    #[inline]
    pub fn notice() -> &'static str {
        &NOTICE
    }

    #[inline]
    pub fn license_apache() -> &'static str {
        &LICENSE_APACHE
    }

    #[inline]
    pub fn license_mit() -> &'static str {
        &LICENSE_MIT
    }

    #[inline]
    pub fn license_html() -> &'static str {
        &LICENSE_HTML
    }
}
