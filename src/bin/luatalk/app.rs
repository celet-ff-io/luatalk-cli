use std::{
    cell::RefCell,
    io::{self, Write},
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
};

use clap_stdin::FileOrStdout;
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
        output_dest: FileOrStdout,
    },

    Export {
        lua_input: LuaInput,
        format: OutputFormat,
        output_dest: MultiPurposeWriter,
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

enum MultiPurposeWriter {
    Single(FileOrStdout),
    Multi(PathBuf),
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
            Commands::Show { lua_input_args } => Action::Show {
                lua_input: lua_input_args.into(),
                output_dest: FileOrStdout::from_str("-").into_diagnostic()?,
            },

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
                let output_dest = if concat_pages {
                    debug!("Output file: {}", output.filename());
                    output.pipe(MultiPurposeWriter::Single)
                } else {
                    if output.is_stdout() {
                        return Err(diagnostic!(
                            "Output file cannot be stdout to export article in pages."
                        )
                        .into());
                    }
                    output
                        .filename()
                        .pipe(PathBuf::from)
                        .pipe(MultiPurposeWriter::Multi)
                };
                Action::Export {
                    lua_input,
                    format,
                    output_dest,
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
            Action::Show { output_dest, .. } => {
                let mut writer = output_dest.clone().into_writer().into_diagnostic()?;
                writeln!(writer, "{:#?}", self.state.article).into_diagnostic()?;
                writer.flush().into_diagnostic()
            }

            Action::Export {
                format,
                output_dest,
                ..
            } => {
                if format != &OutputFormat::Momotalk {
                    return Err(diagnostic!("Only Momotalk format is supported").into());
                }
                match output_dest {
                    MultiPurposeWriter::Single(output_dest) => {
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

                        let mut writer = output_dest.clone().into_writer().into_diagnostic()?;
                        serde_json::to_writer_pretty(&mut writer, &momotalk_export)
                            .into_diagnostic()?;

                        writer.write_all(b"\n").into_diagnostic()?;
                        writer.flush().into_diagnostic()
                    }

                    MultiPurposeWriter::Multi(_) => {
                        todo!()
                    }
                }
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
