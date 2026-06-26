use std::sync::Arc;

use anyhow::{Context, Result};
use luatalk::{Article, Body, ImageValue, Msg, Page, Profile, Role, TextValue, lua};
use mlua::Lua;
use tap::Pipe;

#[test]
fn from_chunk() -> Result<()> {
    let got = {
        let lua = Lua::new();
        let chunk = include_str!("fixtures/raw_example.lua");

        lua::Article::from_chunk(chunk, &lua)
            .context("Failed to parse Lua chunk")?
            .pipe(Article::from)
    };
    let expected = {
        let her = Profile::builder()
            .name("Her".to_owned())
            .avatar(
                ImageValue::builder()
                    .url("<placeholder-0>".to_owned())
                    .build(),
            )
            .build()
            .pipe(Arc::new);
        Article::builder()
            .pages(vec![
                Page::builder()
                    .msgs(vec![
                        Msg::builder()
                            .role(Role::Guest)
                            .body(Body::Text(
                                TextValue::builder()
                                    .content("Example guest message".to_owned())
                                    .build(),
                            ))
                            .profile(her.pipe_ref(Arc::clone))
                            .build(),
                        Msg::builder()
                            .role(Role::Guest)
                            .body(Body::Image(
                                ImageValue::builder()
                                    .url("<placeholder-1>".to_owned())
                                    .build(),
                            ))
                            .profile(her.pipe_ref(Arc::clone))
                            .build(),
                        Msg::builder()
                            .role(Role::Host)
                            .body(Body::Text(
                                TextValue::builder()
                                    .content("Example host message".to_owned())
                                    .build(),
                            ))
                            .build(),
                        Msg::builder()
                            .role(Role::Host)
                            .body(Body::Image(
                                ImageValue::builder()
                                    .url("<placeholder-2>".to_owned())
                                    .build(),
                            ))
                            .build(),
                    ])
                    .build(),
                Page::builder()
                    .msgs(vec![
                        Msg::builder()
                            .role(Role::System)
                            .body(Body::Text(
                                TextValue::builder()
                                    .content("Example system message".to_owned())
                                    .build(),
                            ))
                            .build(),
                        Msg::builder()
                            .role(Role::Reply)
                            .body(Body::Text(
                                TextValue::builder()
                                    .content("Example reply message".to_owned())
                                    .build(),
                            ))
                            .build(),
                        Msg::builder()
                            .role(Role::BondStory)
                            .body(Body::Text(
                                TextValue::builder()
                                    .content("Example bond story message".to_owned())
                                    .build(),
                            ))
                            .build(),
                    ])
                    .build(),
            ])
            .build()
    };

    assert_eq!(got, expected);

    Ok(())
}
