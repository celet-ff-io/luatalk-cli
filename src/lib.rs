//! `luatalk` (in crate `luatalk-cli`) is a library to eval Lua scripts to domain data,
//! and convert the result to other formats.
//!
//! Get the evaluation result of `example.lua`
//! and compare it with the expected value built by builders:
//!
//! ```
//! use std::sync::Arc;
//! use luatalk::{Article, Body, ImageValue, Lang, LuaTalkExt, Msg, Page, Profile, Role, TextValue, dto};
//! use miette::{IntoDiagnostic, Result, WrapErr};
//! use mlua::Lua;
//! use tap::Pipe;
//!
//! let lua = Lua::new();
//! lua.load_default_lib().unwrap();
//!
//! let chunk = include_str!("../assets/lua/input/example.lua");
//! let got = dto::Article::try_from_chunk(chunk, &lua)
//!     .unwrap()
//!     .pipe(Article::from);
//!
//! let expected = {
//!     let her = Profile::builder()
//!         .name("Her".to_owned())
//!         .avatar(
//!             ImageValue::builder()
//!                 .url("<placeholder-0>".to_owned())
//!                 .build(),
//!         )
//!         .build()
//!         .pipe(Arc::new);
//!     Article::builder()
//!         .lang(Lang::En)
//!         .pages(vec![
//!             Page::builder()
//!                 .msgs(vec![
//!                     Msg::builder()
//!                         .role(Role::Guest)
//!                         .body(Body::Text(
//!                             TextValue::builder()
//!                                 .content("Example guest message".to_owned())
//!                                 .build(),
//!                         ))
//!                         .profile(her.pipe_ref(Arc::clone))
//!                         .build(),
//!                     Msg::builder()
//!                         .role(Role::Guest)
//!                         .body(Body::Image(
//!                             ImageValue::builder()
//!                                 .url("<placeholder-1>".to_owned())
//!                                 .build(),
//!                         ))
//!                         .profile(her.pipe_ref(Arc::clone))
//!                         .build(),
//!                     Msg::builder()
//!                         .role(Role::Host)
//!                         .body(Body::Text(
//!                             TextValue::builder()
//!                                 .content("Example host message".to_owned())
//!                                 .build(),
//!                         ))
//!                         .build(),
//!                     Msg::builder()
//!                         .role(Role::Host)
//!                         .body(Body::Image(
//!                             ImageValue::builder()
//!                                 .url("<placeholder-2>".to_owned())
//!                                 .build(),
//!                         ))
//!                         .build(),
//!                 ])
//!                 .build(),
//!             Page::builder()
//!                 .msgs(vec![
//!                     Msg::builder()
//!                         .role(Role::System)
//!                         .body(Body::Text(
//!                             TextValue::builder()
//!                                 .content("Example system message".to_owned())
//!                                 .build(),
//!                         ))
//!                         .build(),
//!                     Msg::builder()
//!                         .role(Role::Reply)
//!                         .body(Body::Text(
//!                             TextValue::builder()
//!                                 .content("Example reply message".to_owned())
//!                                 .build(),
//!                         ))
//!                         .build(),
//!                     Msg::builder()
//!                         .role(Role::BondStory)
//!                         .body(Body::Text(
//!                             TextValue::builder()
//!                                 .content("Example bond story message".to_owned())
//!                                 .build(),
//!                         ))
//!                         .build(),
//!                 ])
//!                 .build(),
//!         ])
//!         .build()
//! };
//!
//! assert_eq!(got, expected);
//! ```

mod error;
mod model;
mod talk;

pub use error::LuaParseError;

pub use model::{AndLang, InLang, IntoAndLang, Lang};
pub use model::{Article, Body, Msg, Page, Profile};
pub use model::{ImageValue, Role, TextValue};
pub use model::{dto, momotalk};

pub use talk::{LuaExt, LuaTalkLibError};
