/// For subcommand `generate`.
use std::io::{self, Write, stdout};

use clap_stdin::FileOrStdout;
use miette::{IntoDiagnostic, Result};

use luatalk::assets;

use crate::{
    app::{App, common::FileOrStdoutExt, state},
    cli::generate::{self, AssetArg, Command, ExampleLangArg, LicenseArg, TypstOutputConfigArgs},
};

#[derive(Debug, Clone, PartialEq)]
pub enum GenerateAction {
    Example {
        lang: ExampleLang,
    },
    Typst {
        data: String,
        config: TypstOutputConfig,
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
            Command::Example { lang } => Self::Example { lang: lang.into() },
            Command::Typst { data, config } => Self::Typst {
                data,
                config: config.into(),
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
        match value {
            ExampleLangArg::En => Self::En,
            ExampleLangArg::ZhHans => Self::ZhHans,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TypstOutputConfig {
    pub font_size: u32,
    pub width: u32,
    pub font_family: String,
    pub length_factor: f32,
}

impl From<TypstOutputConfigArgs> for TypstOutputConfig {
    fn from(value: TypstOutputConfigArgs) -> Self {
        let TypstOutputConfigArgs {
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
        match value {
            AssetArg::LuaInputExampleEn => Self::LuaInputExampleEn,
            AssetArg::LuaInputExampleZhHans => Self::LuaInputExampleZhHans,
            AssetArg::LuaLibTalk => Self::LuaLibTalk,
            AssetArg::TypstOutput => Self::TypstOutput,
            AssetArg::LicenseNotice => Self::LicenseNotice,
            AssetArg::LicenseApache => Self::LicenseApache,
            AssetArg::LicenseMit => Self::LicenseMit,
            AssetArg::LicenseHtml => Self::LicenseHtml,
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
        match value {
            LicenseArg::Notice => Self::Notice,
            LicenseArg::Apache => Self::Apache,
            LicenseArg::Mit => Self::Mit,
            LicenseArg::ThirdPartyLicenses => Self::ThirdPartyLicenses,
        }
    }
}

// Action

impl App<state::Initial> {
    pub fn run_generate(self, output: &FileOrStdout, action: &GenerateAction) -> Result<()> {
        let mut w = output.clone_to_output_writer()?;
        match action {
            GenerateAction::Example { lang } => match lang {
                ExampleLang::En => w.output_str(assets::lua::input::example_en())?,

                ExampleLang::ZhHans => w.output_str(assets::lua::input::example_zh_hans())?,
            },

            GenerateAction::Typst { data, config } => {
                Self::output_typst(&mut stdout(), data, config)?
            }

            GenerateAction::Completion { shell } => generate::completion(*shell, &mut io::stdout()),

            GenerateAction::Asset { asset } => match asset {
                Asset::LuaInputExampleEn => w.output_str(assets::lua::input::example_en())?,
                Asset::LuaInputExampleZhHans => {
                    w.output_str(assets::lua::input::example_zh_hans())?
                }
                Asset::LuaLibTalk => w.output_str(assets::lua::lib::talk())?,

                Asset::TypstOutput => w.output_str(assets::typst::output())?,

                Asset::LicenseNotice => w.output_str(assets::license::notice())?,
                Asset::LicenseApache => w.output_str(assets::license::license_apache())?,
                Asset::LicenseMit => w.output_str(assets::license::license_mit())?,
                Asset::LicenseHtml => w.output_str(assets::license::license_html())?,
            },

            GenerateAction::ConfigHelp => generate::help_config(),

            GenerateAction::License { license } => match license {
                License::Notice => w.output_str(assets::license::notice())?,
                License::Apache => w.output_str(assets::license::license_apache())?,
                License::Mit => w.output_str(assets::license::license_mit())?,
                License::ThirdPartyLicenses => w.output_str(assets::license::license_html())?,
            },
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
