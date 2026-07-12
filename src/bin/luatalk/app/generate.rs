/// For subcommand `generate`.
use std::io::{self, Write, stdout};

use clap_stdin::FileOrStdout;
use miette::{IntoDiagnostic, Result};

use luatalk::assets;

use crate::{
    app::{App, common::FileOrStdoutExt, state},
    cli::generate::{self, AssetArg, Command, ExampleLangArg, LicenseArg, TypstOutputOptionsArgs},
    conf,
    locale::SupportedLang,
};

#[derive(Debug, Clone, PartialEq)]
pub enum GenerateAction {
    Example {
        lang: Option<ExampleLang>,
    },
    Typst {
        data: String,
        options: TypstOutputOptions,
    },
    Completion {
        shell: clap_complete::Shell,
    },
    Asset {
        asset: Asset,
    },
    ConfigHelp,
    License {
        license: License,
    },
}

impl From<Command> for GenerateAction {
    fn from(value: Command) -> Self {
        match value {
            Command::Example { lang } => Self::Example {
                lang: lang.map(Into::into),
            },
            Command::Typst { data, options } => Self::Typst {
                data,
                options: options.into(),
            },
            Command::Completion { shell } => Self::Completion { shell },
            Command::Asset { asset } => Self::Asset {
                asset: asset.into(),
            },
            Command::ConfigHelp => Self::ConfigHelp,
            Command::License { license } => Self::License {
                license: license.into(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExampleLang {
    En,
    ZhHans,
}

impl From<ExampleLangArg> for ExampleLang {
    fn from(value: ExampleLangArg) -> Self {
        use ExampleLangArg as Arg;
        match value {
            Arg::En => Self::En,
            Arg::ZhHans => Self::ZhHans,
        }
    }
}

impl From<SupportedLang> for ExampleLang {
    fn from(value: SupportedLang) -> Self {
        use SupportedLang as Lang;
        match value {
            Lang::En => Self::En,
            Lang::ZhHans => Self::ZhHans,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypstOutputOptions {
    pub font_size: u32,
    pub width: u32,
    pub font_family: String,
    pub length_factor: f32,
}

impl From<TypstOutputOptionsArgs> for TypstOutputOptions {
    fn from(value: TypstOutputOptionsArgs) -> Self {
        let TypstOutputOptionsArgs {
            font_size,
            width,
            font_family,
            length_factor,
        } = value;
        Self {
            font_size,
            width,
            font_family,
            length_factor,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Asset {
    LuaInputExampleEn,
    LuaInputExampleZhHans,
    LuaLibTalk,

    TypstOutput,

    LicenseNotice,
    LicenseApache,
    LicenseMit,
    LicenseHtml,
}

impl From<AssetArg> for Asset {
    fn from(value: AssetArg) -> Self {
        use AssetArg as Arg;
        match value {
            Arg::LuaInputExampleEn => Self::LuaInputExampleEn,
            Arg::LuaInputExampleZhHans => Self::LuaInputExampleZhHans,
            Arg::LuaLibTalk => Self::LuaLibTalk,
            Arg::TypstOutput => Self::TypstOutput,
            Arg::LicenseNotice => Self::LicenseNotice,
            Arg::LicenseApache => Self::LicenseApache,
            Arg::LicenseMit => Self::LicenseMit,
            Arg::LicenseHtml => Self::LicenseHtml,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum License {
    Notice,
    Apache,
    Mit,
    ThirdPartyLicenses,
}

impl From<LicenseArg> for License {
    fn from(value: LicenseArg) -> Self {
        use LicenseArg as Arg;
        match value {
            Arg::Notice => Self::Notice,
            Arg::Apache => Self::Apache,
            Arg::Mit => Self::Mit,
            Arg::ThirdPartyLicenses => Self::ThirdPartyLicenses,
        }
    }
}

// Action

impl App<state::Initial> {
    pub fn run_generate(self, output: &FileOrStdout, action: &GenerateAction) -> Result<()> {
        let mut w = output.clone_to_output_writer()?;
        match action {
            GenerateAction::Example { lang } => {
                let lang = lang.unwrap_or_else(|| conf::lang().into());
                use ExampleLang::*;
                match lang {
                    En => w.output_str(assets::lua::input::example_en())?,
                    ZhHans => w.output_str(assets::lua::input::example_zh_hans())?,
                }
            }

            GenerateAction::Typst { data, options } => {
                Self::output_typst(&mut stdout(), data, options)?
            }

            GenerateAction::Completion { shell } => generate::completion(*shell, &mut io::stdout()),

            GenerateAction::Asset { asset } => {
                use Asset::*;
                match asset {
                    LuaInputExampleEn => w.output_str(assets::lua::input::example_en())?,
                    LuaInputExampleZhHans => w.output_str(assets::lua::input::example_zh_hans())?,
                    LuaLibTalk => w.output_str(assets::lua::lib::talk())?,

                    TypstOutput => w.output_str(assets::typst::output())?,

                    LicenseNotice => w.output_str(assets::license::notice())?,
                    LicenseApache => w.output_str(assets::license::license_apache())?,
                    LicenseMit => w.output_str(assets::license::license_mit())?,
                    LicenseHtml => w.output_str(assets::license::license_html())?,
                }
            }

            GenerateAction::ConfigHelp => generate::help_config(),

            GenerateAction::License { license } => {
                use License::*;
                match license {
                    Notice => w.output_str(assets::license::notice())?,
                    Apache => w.output_str(assets::license::license_apache())?,
                    Mit => w.output_str(assets::license::license_mit())?,
                    ThirdPartyLicenses => w.output_str(assets::license::license_html())?,
                }
            }
        }
        Ok(())
    }
}

// Utils

trait WriteExt {
    fn output_str(&mut self, s: &str) -> Result<()>;
}

impl<W: Write> WriteExt for W {
    fn output_str(&mut self, s: &str) -> Result<()> {
        writeln!(self, "{s}").into_diagnostic()?;
        self.flush().into_diagnostic()
    }
}
