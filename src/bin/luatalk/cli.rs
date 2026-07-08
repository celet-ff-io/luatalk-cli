use clap::{
    CommandFactory, Parser, Subcommand, ValueEnum,
    builder::{
        Styles,
        styling::{AnsiColor, Effects},
    },
};
use clap_complete::{Shell, generate};
use clap_stdin::{FileOrStdin, FileOrStdout};
use clap_verbosity_flag::{InfoLevel, Verbosity};

static STYLES: Styles = Styles::styled()
    .header(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .usage(AnsiColor::Yellow.on_default().effects(Effects::BOLD))
    .literal(AnsiColor::Green.on_default().effects(Effects::BOLD))
    .placeholder(AnsiColor::Red.on_default());

fn styles() -> Styles {
    STYLES.clone()
}

/// Build article from Lua file (using Lua 5.5).
#[derive(Debug, Parser)]
#[command(version)]
#[command(styles = styles())]
#[command(
    after_help = "Use the subcommand `generate config-help` for advanced configuration options.

Visit the crate page at `https://crates.io/crates/luatalk-cli` \
or the repository for more information.
"
)]
pub struct AppArgs {
    #[command(flatten)]
    pub verbose: Verbosity<InfoLevel>,

    #[command(subcommand)]
    pub command: AppCommand,
}

#[derive(Debug, Clone, Subcommand)]
pub enum AppCommand {
    /// Output useful files at runtime or hard-coded in the binary.
    Generate {
        #[command(subcommand)]
        command: generate::Command,
    },

    /// Process LuaTalk article input to something.
    Do {
        /// Input file. `-` for stdin.
        input: FileOrStdin,

        /// Concatenate all pages into a single page.
        #[arg(short, long)]
        concat_pages: bool,

        /// Input file format.
        /// Defaults to be inferred from file extension;
        /// for stdin, defaults to `json`.
        #[arg(short, long)]
        format: Option<do_::InputFormatArg>,

        #[command(subcommand)]
        output_commands: do_::OutputCommand,
    },
}

pub mod generate {
    use std::io::{self, IsTerminal};

    use super::*;

    #[derive(Debug, Clone, Subcommand)]
    pub enum Command {
        /// Example input Lua file.
        Example {
            #[arg(default_value = "en")]
            lang: ExampleLangArg,
        },

        /// Generate a Typst file to render article, from the base Typst file in asset.
        Typst {
            /// File path of article data in JSON format, from `do <INPUT> json`.
            #[arg(long, default_value = "output.json")]
            data: String,

            /// Font size in points.
            #[arg(long, default_value_t = 20)]
            font_size: u32,

            /// Page width in points
            #[arg(long, default_value_t = 720)]
            width: u32,

            /// Font family name to use.
            #[arg(long, default_value = "BlueakaBetaGBK")]
            font_family: String,

            /// Length factor for zooming all elements in the page.
            #[arg(long, default_value_t = 1.0)]
            length_factor: f32,
        },

        /// Shell completion script for the specified shell.
        #[command(after_help = "e.g. `source <(luatalk generate completion bash)` \
            for bash users to load the completion script in current session.")]
        Completion { shell: Shell },

        /// Useful assets file.
        /// You may also obtain them from source.
        Asset { asset: AssetArg },

        /// Help about advanced configuration of this program
        ConfigHelp,
    }

    #[derive(Debug, Clone, ValueEnum)]
    pub enum ExampleLangArg {
        /// English
        En,

        /// Simplified Chinese
        #[value(alias("zh-Hans"))]
        ZhHans,
    }

    #[derive(Debug, Clone, ValueEnum)]
    pub enum AssetArg {
        /// Example input Lua file in English
        #[value(name = "lua/input/example_en.lua")]
        LuaInputExampleEn,

        /// Example input Lua file in Simplified Chinese
        #[value(name = "lua/input/example_zh-hans.lua")]
        LuaInputExampleZhHans,

        /// `talk.lua` module in default lib of this program
        #[value(name = "lua/lib/talk.lua")]
        LuaLibTalk,

        /// Base file for Typst output
        #[value(name = "typst/output.typ")]
        TypstOutput,
    }

    pub fn completion(shell: Shell, buf: &mut dyn std::io::Write) {
        let mut cmd = AppArgs::command();
        generate(shell, &mut cmd, "luatalk", buf);
    }

    pub fn help_config() {
        let h;
        let l;
        let p;
        let r;
        if io::stdout().is_terminal() {
            let styles = styles();
            h = styles.get_header().render().to_string();
            l = styles.get_literal().render().to_string();
            p = styles.get_placeholder().render().to_string();
            r = styles.get_header().render_reset().to_string();
        } else {
            h = String::new();
            l = String::new();
            p = String::new();
            r = String::new();
        };
        println!(
            "{h}Via environment variables:{r}
{l}LUATALK{r}
  {l}DO_LUA{r}
    {l}NO_DEFAULT_LIB{r}: To disable loading the `talk.lua` module
      LUATALK__DO_LUA__NO_DEFAULT_LIB={p}1{r}
    {l}ADDITIONAL_PATH{r}: Additional Lua search paths which will be appended before the default search paths
      LUATALK__DO_LUA__ADDITIONAL_PATH={p}'/path/to/lib/?.lua;/path/to/lib/?/init.lua;'{r}"
        );
    }
}

pub mod do_ {
    use super::*;

    #[derive(Debug, Clone, ValueEnum)]
    #[value(rename_all = "lower")]
    pub enum InputFormatArg {
        /// Lua file returns LuaTalk Article
        Lua,

        /// JSON file of LuaTalk Article
        Json,
    }

    #[derive(Debug, Clone, Subcommand)]
    pub enum OutputCommand {
        /// Show LuaTalk article in `luatalk::Article` structure string.
        Show {
            /// Ouptut. Defaults to `stdout`.
            #[arg(short, long, default_value = "-")]
            output: FileOrStdout,
        },

        Json {
            /// Ouptut. Defaults to `stdout`.
            #[arg(short, long, default_value = "-")]
            output: FileOrStdout,
        },

        /// Momotalk export JSON format for 'https://github.com/U1805/momotalk'
        Momotalk {
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

            /// Output plurality.
            #[arg(long, default_value = "auto")]
            pl: OutputPluralityArg,
        },
    }

    #[derive(Debug, Clone, Default, ValueEnum)]
    pub enum OutputPluralityArg {
        /// Use `single` for article of only one page,
        /// `multi` for article of multiple pages.
        #[default]
        Auto,

        /// Output a single file.
        Single,

        /// Output multiple files, one for each page.
        Multi,
    }
}
