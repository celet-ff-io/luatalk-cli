use std::path::PathBuf;

use clap::{
    ArgAction, Parser, Subcommand, ValueEnum,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};
use clap_stdin::{FileOrStdin, FileOrStdout};
use clap_verbosity_flag::{InfoLevel, Verbosity};

/// Build article from Lua file (using Lua 5.5).
#[derive(Debug, Parser)]
#[command(version)]
#[command(styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Red.on_default())
)]
#[command(
    after_help = "Visit the crate page at 'https://crates.io/crates/luatalk-cli' \
        or the repository for more information.
"
)]
pub struct Args {
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Generate {
        asset: AssetArg,
    },

    /// Show LuaTalk article in `luatalk::Article` structure string.
    Show {
        #[command(flatten)]
        lua_input_args: LuaInputArgs,
    },

    /// Export LuaTalk article.
    Export {
        #[command(flatten)]
        lua_input_args: LuaInputArgs,

        /// Output format.
        #[arg(short, long)]
        format: OutputFormatArg,

        /// Concatenate all pages into a single page.
        #[arg(long, default_value_t = false)]
        concat_pages: bool,

        /// Ouptut. Defaults to `None`.
        ///
        /// For one file: a file path, or `-` for stdout.
        /// `None` stands for stdout.
        ///
        /// For multiple files: a directory path,
        /// or a format string with placeholders for page index starts from 1.
        /// e.g. `article_{i}.json`.
        /// `None` stands for directory named after stem portion of input file name.
        #[arg(short, long)]
        output: Option<FileOrStdout>,
    },
}

#[derive(Debug, Clone, ValueEnum)]
pub enum AssetArg {
    /// Example input Lua file
    #[value(name = "example")]
    Example,

    /// `talk.lua` module enabled by the flag `--lib-default`
    #[value(name = "lib/talk.lua")]
    LibTalk,
}

#[derive(Debug, clap::Args)]
pub struct LuaInputArgs {
    /// Input Lua file. '-' for stdin.
    pub input: FileOrStdin,

    /// Set this flag to load the `talk.lua` module hard-coded in program.
    #[arg(long)]
    pub lib_default: bool,

    /// Additional search directories for Lua modules. Can be specified multiple times.
    #[arg(long = "lib", action = ArgAction::Append)]
    pub libs: Vec<PathBuf>,
}

#[derive(Debug, Clone, ValueEnum)]
#[value(rename_all = "kebab-case")]
pub enum OutputFormatArg {
    /// Momotalk export JSON format for 'https://github.com/U1805/momotalk'
    Momotalk,
}
