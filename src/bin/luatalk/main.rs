use std::io::{BufWriter, Write};

use anyhow::{Context, Result};
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
struct Args {
    /// Input Lua file. '-' for stdin.
    input: FileOrStdin,

    /// Ouptut file. '-' for stdout. Defaults to stdout.
    #[arg(short, long, default_value = "-")]
    output: FileOrStdout,

    #[command(flatten)]
    verbose: Verbosity<InfoLevel>,
}

fn main() -> Result<()> {
    let source;
    let mut writer;
    {
        let args = Args::parse();
        env_logger::Builder::new()
            .filter_level(args.verbose.log_level_filter())
            .init();
        let input = args.input;
        debug!("Input file: {}", input.filename());
        source = input.contents().context("Input file not found")?;
        let output = args.output;
        debug!("Output file: {}", output.filename());
        writer = output.into_writer()?.pipe(BufWriter::new);
    };

    let lua = Lua::new();

    let article = lua::Article::from_chunk(source, &lua)?.pipe(Article::from);
    debug!("Build article success");

    writeln!(writer, "{article:#?}")?;

    writer.flush()?;

    Ok(())
}
