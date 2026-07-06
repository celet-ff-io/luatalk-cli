use include_flate::flate;

pub mod lua {
    use super::*;

    pub mod input {
        use super::*;

        flate!(pub static EXAMPLE_EN: str from "./assets/lua/input/example_en.lua");
        flate!(pub static EXAMPLE_ZH_HANS: str from "./assets/lua/input/example_zh-hans.lua");
    }

    pub mod lib {
        pub const TALK: &str = include_str!("../assets/lua/lib/talk.lua");
    }
}
