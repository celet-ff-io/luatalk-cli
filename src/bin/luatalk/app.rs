use std::{
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use clap::{
    ArgAction, Parser,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};
use clap_stdin::{FileOrStdin, FileOrStdout};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::{debug, warn};
use miette::{IntoDiagnostic, Result, WrapErr};
use mlua::{Lua, Table};
use tap::{Pipe, Tap};

use luatalk::{Article, LuaTalkExt, lua};

/// Convert your Lua file to `luatalk::Article` structure string.
/// Supports Lua 5.5.
#[derive(Debug, Parser)]
#[command(version)]
#[command(styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Red.on_default())
)]
pub struct Args {
    /// Input Lua file. '-' for stdin.
    input: FileOrStdin,

    // Load the module `talk.lua` hard-coded in program.
    #[arg(long)]
    lib_default: bool,

    /// Additional search directories for Lua modules. Can be specified multiple times.
    #[arg(long = "lib", action = ArgAction::Append)]
    libs: Vec<PathBuf>,

    /// Ouptut file. '-' for stdout. Defaults to stdout.
    #[arg(short, long, default_value = "-")]
    output: FileOrStdout,

    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

pub struct App {
    source: String,

    lib_default: bool,

    path_addtion: String,

    writer: BufWriter<Box<dyn Write>>,

    lua: Lua,
}

impl App {
    pub fn new(args: Args) -> Result<Self> {
        env_logger::Builder::new()
            .filter_level(args.verbose.log_level_filter())
            .init();

        let source;
        let lib_default = args.lib_default;
        let path_addtion: String;
        let writer;

        {
            let input = args.input;
            debug!("Input file: {}", input.filename());
            source = input
                .contents()
                .into_diagnostic()
                .wrap_err("Input file not found")?;

            const INITIAL_CAPACITY: usize = 128;
            path_addtion = args
                .libs
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

            let output = args.output;
            debug!("Output file: {}", output.filename());
            writer = output
                .into_writer()
                .into_diagnostic()?
                .pipe(Box::new)
                .pipe(|w| w as Box<dyn Write>)
                .pipe(BufWriter::new);
        };

        Ok(Self {
            source,
            lib_default,
            path_addtion,
            writer,

            lua: Lua::new(),
        })
    }

    fn as_lua_path_entry_or_empty(dir: &Path, file: &str) -> String {
        dir.join(file)
            .to_str()
            .map_or(String::new(), |s| s.to_owned() + ";")
    }

    pub fn run(self) -> Result<()> {
        let lua = self.lua;

        if self.lib_default {
            debug!("Loading default lib");
            lua.load_default_lib().into_diagnostic()?;
        }

        if !self.path_addtion.is_empty() {
            const KEY_PATH: &str = "path";
            let pacakges: Table = lua.globals().get("package").into_diagnostic()?;
            let current_path: String = pacakges.get(KEY_PATH).into_diagnostic()?;
            let new_path = self.path_addtion + &current_path;
            debug!("Lua package path will update to {new_path}");
            pacakges.set(KEY_PATH, new_path).into_diagnostic()?;
        }

        let article = lua::Article::from_chunk(&self.source, &lua)
            .into_diagnostic()?
            .pipe(Article::from);
        debug!("Build article success");

        let mut writer = self.writer;

        writeln!(writer, "{article:#?}").into_diagnostic()?;

        writer.flush().into_diagnostic()?;

        Ok(())
    }
}
