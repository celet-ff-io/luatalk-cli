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

use luatalk::{Article, lua};

/// Convert your Lua file to something.
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

    /// Ouptut file. '-' for stdout. Defaults to stdout.
    #[arg(short, long, default_value = "-")]
    output: FileOrStdout,

    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

impl App {
    pub(crate) fn run(self) -> Result<()> {
        let source;
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

        let article = lua::Article::from_chunk(&source, &lua)
            .into_diagnostic()?
            .pipe(Article::from);
        debug!("Build article success");
        println!("{source}");

        writeln!(writer, "{article:#?}").into_diagnostic()?;

        writer.flush().into_diagnostic()?;

        Ok(())
    }
}
