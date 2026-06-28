use std::{
    cell::RefCell,
    io::{self, Write},
    path::Path,
    rc::Rc,
};

use log::{debug, warn};
use miette::{IntoDiagnostic, Result, WrapErr, diagnostic};
use mlua::{Lua, Table};
use tap::{Pipe, Tap};

use luatalk::{
    Article, LuaTalkExt, Msg,
    lang::{IntoWithLang, Lang},
    lua, momotalk,
};

use crate::{
    app::state::State,
    cli::{Args, Commands, LuaInputArgs, OutputFormatArg},
};

pub trait Runnable {
    fn run(self) -> Result<()>;
}

pub struct App<S: State> {
    /// App state
    state: S,

    /// App action by CLI command
    action: Rc<Action>,
}

enum Action {
    Show {
        lua_input: LuaInput,
        writer: RefCell<Box<dyn Write>>,
    },

    Export {
        lua_input: LuaInput,
        format: OutputFormat,
        concat_pages: bool,
        writer: RefCell<Box<dyn Write>>,
    },
}

struct LuaInput {
    source: String,
    enable_lib_default: bool,
    path_addtion: String,

    lua: Lua,
}

impl From<LuaInputArgs> for LuaInput {
    fn from(value: LuaInputArgs) -> Self {
        let LuaInputArgs {
            input,
            lib_default,
            libs,
        } = value;

        let source = input
            .contents()
            .into_diagnostic()
            .wrap_err("Input file not found")
            .unwrap_or_default();

        let enable_lib_default = lib_default;

        let path_addtion = libs
            .iter()
            .filter(|p| {
                p.is_dir().tap(|&is| {
                    if is {
                        debug!("Add directory path: {}", p.display());
                    } else {
                        warn!("Ignore bad directory path: {}", p.display());
                    }
                })
            })
            .flat_map(|p| {
                [
                    App::<state::Initial>::as_lua_path_entry_or_empty(p, "?.lua"),
                    App::<state::Initial>::as_lua_path_entry_or_empty(p, "?/init.lua"),
                ]
            })
            .collect::<String>();

        let lua = Lua::new();

        Self {
            source,
            enable_lib_default,
            path_addtion,
            lua,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum OutputFormat {
    Momotalk,
}

impl<S: State> App<S> {
    fn as_lua_path_entry_or_empty(dir: &Path, file: &str) -> String {
        dir.join(file)
            .to_str()
            .map_or(String::new(), |s| s.to_owned() + ";")
    }
}

impl TryFrom<Args> for App<state::Initial> {
    type Error = miette::Report;

    fn try_from(value: Args) -> Result<Self, Self::Error> {
        let args = value;

        env_logger::Builder::new()
            .filter_level(args.verbose.log_level_filter())
            .init();

        let action = match args.command {
            Commands::Show { lua_input_args } => {
                let lua_input = lua_input_args.into();
                let writer = io::stdout();
                Action::Show {
                    lua_input,
                    writer: writer
                        .pipe(Box::new)
                        .pipe(|w| w as Box<dyn Write>)
                        .pipe(RefCell::new),
                }
            }

            Commands::Export {
                lua_input_args,
                output,
                format,
                concat_pages,
            } => {
                let lua_input = lua_input_args.into();
                let format = match format {
                    OutputFormatArg::Momotalk => OutputFormat::Momotalk,
                };
                let writer = {
                    debug!("Output file: {}", output.filename());
                    output
                        .into_writer()
                        .into_diagnostic()?
                        .pipe(Box::new)
                        .pipe(|w| w as Box<dyn Write>)
                        .pipe(RefCell::new)
                };
                Action::Export {
                    lua_input,
                    format,
                    concat_pages,
                    writer,
                }
            }
        }
        .pipe(Rc::new);

        Ok(Self {
            action,
            state: state::Initial,
        })
    }
}

impl Runnable for App<state::Initial> {
    fn run(self) -> Result<()> {
        match self.action.pipe_ref(Rc::clone).as_ref() {
            Action::Show { lua_input, .. } => self.lua_input(lua_input)?.run(),

            Action::Export { lua_input, .. } => self.lua_input(lua_input)?.run(),
        }
    }
}
impl App<state::Initial> {
    fn lua_input(self, lua_input: &LuaInput) -> Result<App<state::OfArticle>> {
        let LuaInput {
            source,
            enable_lib_default,
            path_addtion,
            lua,
        } = lua_input;
        if *enable_lib_default {
            debug!("Loading default lib");
            lua.load_default_lib().into_diagnostic()?;
        }

        if !path_addtion.is_empty() {
            const KEY_PATH: &str = "path";
            let pacakges: Table = lua.globals().get("package").into_diagnostic()?;
            let current_path: String = pacakges.get(KEY_PATH).into_diagnostic()?;
            let new_path = format!("{path_addtion}{current_path}");
            debug!("Lua package path will update to {new_path}");
            pacakges.set(KEY_PATH, new_path).into_diagnostic()?;
        }

        let article = lua::Article::from_chunk(source, lua)
            .into_diagnostic()?
            .pipe(Article::from);

        debug!("Build article success");

        App {
            state: state::OfArticle { article },
            action: self.action,
        }
        .pipe(Ok)
    }
}

impl Runnable for App<state::OfArticle> {
    fn run(self) -> Result<()> {
        match self.action.as_ref() {
            Action::Show { writer, .. } => {
                let mut writer = writer.borrow_mut();
                writeln!(writer, "{:#?}", self.state.article).into_diagnostic()?;
                writer.flush().into_diagnostic()
            }

            Action::Export {
                format,
                concat_pages,
                writer,
                ..
            } => {
                if format != &OutputFormat::Momotalk {
                    return Err(diagnostic!("Only Momotalk format is supported").into());
                }
                if !concat_pages {
                    return Err(diagnostic!(
                        "Currently only concatenated single page export is supported. Use flag --concat-pages."
                    )
                    .into());
                }
                let mut writer = writer.borrow_mut();

                let talk_history = self
                    .state
                    .article
                    .into_pages()
                    .into_iter()
                    .flatten()
                    .collect::<Vec<Msg>>()
                    .into_with_lang(Lang::En)
                    .try_into()
                    .into_diagnostic()?;

                let momotalk_export = momotalk::MomotalkExport {
                    talk_id: 1,
                    talk_history,
                    select_list: Vec::new(),
                };

                debug!("Build MomotalkExport structure success");

                serde_json::to_writer_pretty(writer.as_mut(), &momotalk_export)
                    .into_diagnostic()?;

                writer.write_all(b"\n").into_diagnostic()?;
                writer.flush().into_diagnostic()
            }
        }
    }
}

mod state {
    use super::*;

    pub trait State {}

    pub struct Initial;

    impl State for Initial {}

    pub struct OfArticle {
        pub article: Article,
    }

    impl State for OfArticle {}
}
