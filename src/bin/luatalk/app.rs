use std::{
    io::{BufWriter, Write},
    path::Path,
};

use log::{debug, warn};
use miette::{IntoDiagnostic, Result, WrapErr};
use mlua::{Lua, Table};
use tap::{Pipe, Tap};

use luatalk::{Article, LuaTalkExt, lua};

use crate::cli::{CliArgs, CliCommands, CliLuaInputArgs};

pub trait Runnable {
    fn run(self) -> Result<()>;
}

pub struct App {
    action: AppAction,
}

impl App {
    fn as_lua_path_entry_or_empty(dir: &Path, file: &str) -> String {
        dir.join(file)
            .to_str()
            .map_or(String::new(), |s| s.to_owned() + ";")
    }
}

impl TryFrom<CliArgs> for App {
    type Error = miette::Report;

    fn try_from(value: CliArgs) -> Result<Self, Self::Error> {
        let args = value;

        env_logger::Builder::new()
            .filter_level(args.verbose.log_level_filter())
            .init();

        let path_addtion;

        let action = match args.command {
            CliCommands::Show {
                lua_input_args,
                output,
            } => {
                let CliLuaInputArgs {
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
                };
                AppAction::Show {
                    lua_input: AppLuaInput {
                        source,
                        enable_lib_default,
                        path_addtion,
                        lua: Lua::new(),
                    },
                    writer,
                }
            }
        };

        Ok(Self { action })
    }
}

impl Runnable for App {
    fn run(self) -> Result<()> {
        match self.action {
            AppAction::Show {
                lua_input:
                    AppLuaInput {
                        source,
                        enable_lib_default,
                        path_addtion,
                        lua,
                    },
                mut writer,
            } => {
                if enable_lib_default {
                    debug!("Loading default lib");
                    lua.load_default_lib().into_diagnostic()?;
                }

                if !path_addtion.is_empty() {
                    const KEY_PATH: &str = "path";
                    let pacakges: Table = lua.globals().get("package").into_diagnostic()?;
                    let current_path: String = pacakges.get(KEY_PATH).into_diagnostic()?;
                    let new_path = path_addtion + &current_path;
                    debug!("Lua package path will update to {new_path}");
                    pacakges.set(KEY_PATH, new_path).into_diagnostic()?;
                }

                let article = lua::Article::from_chunk(&source, &lua)
                    .into_diagnostic()?
                    .pipe(Article::from);
                debug!("Build article success");

                writeln!(writer, "{article:#?}").into_diagnostic()?;

                writer.flush().into_diagnostic()?;
                Ok(())
            }
        }
    }
}

enum AppAction {
    Show {
        lua_input: AppLuaInput,
        writer: BufWriter<Box<dyn Write>>,
    },
}

struct AppLuaInput {
    source: String,
    enable_lib_default: bool,
    path_addtion: String,

    lua: Lua,
}
