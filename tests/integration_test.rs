use std::sync::{Arc, OnceLock};

use luatalk::{
    Article, Body, ImageValue, InLang, IntoAndLang, Lang, LuaExt, Msg, Page, Profile, Role,
    TextValue, dto, momotalk,
};
use miette::{IntoDiagnostic, Result, diagnostic};
use mlua::Lua;
use pretty_assertions::assert_eq;
use tap::Pipe;

static EXAMPLE_ARTICALE_RESULT: OnceLock<Result<Article>> = OnceLock::new();

fn try_get_example_article() -> Result<&'static Article> {
    EXAMPLE_ARTICALE_RESULT
        .get_or_init(|| {
            let lua =
                Lua::new().pipe(|lua| lua.load_default_lib().into_diagnostic().map(|_| lua))?;
            let chunk = include_str!("fixtures/example.lua");

            dto::Article::try_from_chunk(chunk, &lua)
                .into_diagnostic()?
                .pipe(Article::from)
                .pipe(Ok)
        })
        .as_ref()
        .map_err(|e| miette::miette!("{e}"))
}

#[test]
fn check_article() -> Result<()> {
    let got = try_get_example_article()?;
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
            .lang(Lang::En)
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

    assert_eq!(got, &expected);

    Ok(())
}

#[test]
fn check_article_raw() -> Result<()> {
    let example = try_get_example_article()?;
    let raw_example = {
        let lua = Lua::new();
        let chunk = include_str!("fixtures/raw_example.lua");

        dto::Article::try_from_chunk(chunk, &lua)
            .into_diagnostic()?
            .pipe(Article::from)
    };

    assert_eq!(example, &raw_example);

    Ok(())
}

#[test]
fn export_page_to_momotalk_export_json() -> Result<()> {
    let got = {
        let talk_history: Vec<momotalk::TalkHistoryItem> = {
            let lua =
                Lua::new().pipe(|lua| lua.load_default_lib().into_diagnostic().map(|_| lua))?;
            let chunk = include_str!("fixtures/example_Momotalk-export.lua");
            let article = dto::Article::try_from_chunk(chunk, &lua)
                .into_diagnostic()?
                .pipe(Article::from);
            let lang = article.lang();
            article
                .into_pages()
                .first()
                .ok_or_else(|| diagnostic!("Article has no pages"))?
                .msgs()
                .clone()
                .into_and_lang(lang)
                .try_into()
                .into_diagnostic()?
        };
        let select_list = vec![momotalk::SelectListItem {
            id: 10000,
            name: "Aru".to_owned(),
            avatar: "https://BlueArcbox.github.io/resources/Avatars/Kivo/Released/10000.webp"
                .to_owned(),
        }];
        momotalk::MomotalkExport {
            talk_id: 1,
            talk_history,
            select_list,
        }
    }
    .pipe_ref(serde_json::to_value)
    .into_diagnostic()?;

    let expected = include_str!("fixtures/example_Momotalk-export.json")
        .pipe(serde_json::from_str::<serde_json::Value>)
        .into_diagnostic()?;

    assert_eq!(got, expected);

    Ok(())
}
