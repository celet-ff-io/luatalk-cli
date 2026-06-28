use std::{
    cell::RefCell,
    io::{BufWriter, Write},
    path::Path,
    rc::Rc,
};

use log::{debug, warn};
use miette::{IntoDiagnostic, Result, WrapErr};
use mlua::{Lua, Table};
use tap::{Pipe, Tap};

use luatalk::{Article, LuaTalkExt, lua};

use crate::{
    app::state::State,
    cli::{Args, Commands, LuaInputArgs},
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
        writer: RefCell<BufWriter<Box<dyn Write>>>,
    },
}

struct LuaInput {
    source: String,
    enable_lib_default: bool,
    path_addtion: String,

    lua: Lua,
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

        let path_addtion;

        let action = match args.command {
            Commands::Show {
                lua_input_args,
                output,
            } => {
                let LuaInputArgs {
                    input,
                    lib_default,
                    libs,
                } = lua_input_args;

                let source = {
                    debug!("Input file: {}", input.filename());
                    input
                        .contents()
                        .into_diagnostic()
                        .wrap_err("Input file not found")?
                };
                let enable_lib_default = lib_default;
                let writer = {
                    const INITIAL_CAPACITY: usize = 128;
                    path_addtion = libs
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
                                Self::as_lua_path_entry_or_empty(p, "?.lua"),
                                Self::as_lua_path_entry_or_empty(p, "?/init.lua"),
                            ]
                        })
                        .fold(String::with_capacity(INITIAL_CAPACITY), |mut acc, s| {
                            acc.push_str(&s);
                            acc
                        });

                    debug!("Output file: {}", output.filename());
                    output
                        .into_writer()
                        .into_diagnostic()?
                        .pipe(Box::new)
                        .pipe(|w| w as Box<dyn Write>)
                        .pipe(BufWriter::new)
                        .pipe(RefCell::new)
                };
                Action::Show {
                    lua_input: LuaInput {
                        source,
                        enable_lib_default,
                        path_addtion,
                        lua: Lua::new(),
                    },
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
        match &*self.action.pipe_ref(Rc::clone) {
            Action::Show { lua_input, .. } => {
                App::<state::Initial>::lua_input(self, lua_input)?.run()
            }
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

        Ok(App {
            state: state::OfArticle { article },
            action: self.action,
        })
    }
}

impl Runnable for App<state::OfArticle> {
    fn run(self) -> Result<()> {
        match &*self.action {
            Action::Show { writer, .. } => {
                let mut writer = writer.borrow_mut();
                writeln!(writer, "{:#?}", self.state.article).into_diagnostic()?;
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
