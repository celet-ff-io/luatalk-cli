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
pub(crate) struct App {
    /// Input Lua file. '-' for stdin.
    input: FileOrStdin,

    // #[arg(long = "lib", action = ArgAction::Append)]
    // libs: Vec<PathBuf>,

    // Load the module `talk.lua`.
    #[arg(long)]
    lib_default: bool,

    /// Ouptut file. '-' for stdout. Defaults to stdout.
    #[arg(short, long, default_value = "-")]
    output: FileOrStdout,

    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

impl App {
    pub(crate) fn run(self) -> Result<()> {
        let source;
        // let lib: Vec<PathBuf>;
        let mut writer;
        {
            let input = self.input;
            debug!("Input file: {}", input.filename());

            source = input
                .contents()
                .into_diagnostic()
                .wrap_err("Input file not found")?;
            let output = self.output;
            debug!("Output file: {}", output.filename());

            writer = output.into_writer().into_diagnostic()?.pipe(BufWriter::new);
        };

        let lua = Lua::new();

        if self.lib_default {
            debug!("Loading default lib");
            lua.load_default_lib().into_diagnostic()?;
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
