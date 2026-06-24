use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::{
    Parser,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};
use clap_stdin::FileOrStdin;

/// Convert your Lua file to HTML.
#[derive(Parser, Debug)]
#[command(version)]
#[command(styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Red.on_default())
)]
struct Args {
    /// Input Lua file. Lua 5.5 supported.
    input: FileOrStdin,

    #[arg(short, long, default_value = "index.html")]
    output: PathBuf,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let input = args.input.contents().context("Input file not found")?;

    println!("Input: {input}");
    Ok(())
}
