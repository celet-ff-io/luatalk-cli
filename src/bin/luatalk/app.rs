use std::io::{BufWriter, Write};

use clap::{
    Parser,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};
use clap_stdin::{FileOrStdin, FileOrStdout};
use clap_verbosity_flag::{InfoLevel, Verbosity};
use log::debug;
use miette::{IntoDiagnostic, Result, WrapErr};
use mlua::Lua;
use tap::Pipe;

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

    // #[arg(long = "lib", action = ArgAction::Append)]
    // libs: Vec<PathBuf>,

    // Load the module `talk.lua` hard-coded in program.
    #[arg(long)]
    lib_default: bool,

    /// Ouptut file. '-' for stdout. Defaults to stdout.
    #[arg(short, long, default_value = "-")]
    output: FileOrStdout,

    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

pub struct App {
    source: String,
    lib_default: bool,
    writer: BufWriter<Box<dyn Write>>,

    lua: Lua,
}

impl App {
    pub fn new(args: Args) -> Result<Self> {
        let source;
        // let lib: Vec<PathBuf>;
        let lib_default = args.lib_default;
        let writer;
        {
            let input = args.input;
            debug!("Input file: {}", input.filename());

            source = input
                .contents()
                .into_diagnostic()
                .wrap_err("Input file not found")?;
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
            writer,

            lua: Lua::new(),
        })
    }

    pub fn run(self) -> Result<()> {
        let lua = self.lua;

        if self.lib_default {
            debug!("Loading default lib");
            lua.load_default_lib().into_diagnostic()?;
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
