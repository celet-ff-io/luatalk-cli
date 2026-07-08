use std::{
    collections::HashMap,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
    rc::Rc,
    str::FromStr,
};

use clap_stdin::{FileOrStdin, FileOrStdout};
use const_format::formatcp;
use log::{debug, warn};
use miette::{IntoDiagnostic, Result, WrapErr, diagnostic};
use mlua::Lua;
use regex::Regex;
use tap::{Pipe, Tap};

use luatalk::{Article, InLang, IntoAndLang, LuaExt, dto, momotalk};

use crate::{
    app::state::State,
    cli::{
        AppArgs, AppCommand,
        do_::{InputFormatArg, OutputCommand, OutputPluralityArg},
        generate::{self, AssetArg, ExampleLangArg},
    },
    conf,
};

const DEFAULT_OUTPUTNAME: &str = "article";
const INDEX_KEY: &str = "i";

pub trait Runnable {
    fn run(self) -> Result<()>;
}

pub struct App<S: State> {
    /// App state
    state: S,

    /// App action by CLI command
    action: Rc<Action>,
}

#[derive(Debug, Clone)]
enum Action {
    Generate {
        action: GenerateAction,
    },

    Do {
        input: FileOrStdin,
        input_format: InputFormat,
        concat_pages: bool,
        output: OutputAction,
    },
}

impl TryFrom<AppCommand> for Action {
    type Error = miette::Report;

    fn try_from(value: AppCommand) -> Result<Self, Self::Error> {
        match value {
            AppCommand::Generate { command } => Self::Generate {
                action: command.into(),
            },

            AppCommand::Do {
                input,
                concat_pages,
                format,
                output_commands,
            } => {
                let input_format = match format {
                    Some(format) => format.into(),
                    None => {
                        if input.is_stdin() {
                            Default::default()
                        } else {
                            let filename = input.filename();
                            match filename
                                .pipe(Path::new)
                                .extension()
                                .and_then(|ext| ext.to_str())
                            {
                                Some("lua") => InputFormat::Lua,
                                Some("json") => InputFormat::Json,
                                _ => {
                                    return Err(diagnostic!(
                                        help = "Use `--format` to specify input file format",
                                        "Cannot infer input file format from extension: {filename}",
                                    )
                                    .into());
                                }
                            }
                        }
                    }
                };
                let output = match output_commands {
                    OutputCommand::Show { output } => OutputAction::Show { output },
                    OutputCommand::Json { output } => OutputAction::Json { output },
                    OutputCommand::Momotalk { output, pl } => OutputAction::Momotalk {
                        output,
                        plurality: pl.into(),
                    },
                };

                Self::Do {
                    input,
                    input_format,
                    concat_pages,
                    output,
                }
            }
        }
        .pipe(Ok)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum GenerateAction {
    Example { lang: ExampleLang },
    Completion { shell: clap_complete::Shell },
    Asset { asset: Asset },
    ConfigHelp,
}

impl From<generate::Command> for GenerateAction {
    fn from(value: generate::Command) -> Self {
        match value {
            generate::Command::Example { lang } => Self::Example { lang: lang.into() },
            generate::Command::Completion { shell } => Self::Completion { shell },
            generate::Command::Asset { asset } => Self::Asset {
                asset: asset.into(),
            },
            generate::Command::ConfigHelp => Self::ConfigHelp,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum ExampleLang {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Asset {
    LuaInputExampleEn,
    LuaInputExampleZhHans,
    LuaLibTalk,

    TypstOutput,
}

impl From<AssetArg> for Asset {
    fn from(value: AssetArg) -> Self {
        match value {
            AssetArg::LuaInputExampleEn => Self::LuaInputExampleEn,
            AssetArg::LuaInputExampleZhHans => Self::LuaInputExampleZhHans,
            AssetArg::LuaLibTalk => Self::LuaLibTalk,
            AssetArg::TypstOutput => Self::TypstOutput,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
enum InputFormat {
    Lua,

    #[default]
    Json,
}

impl From<InputFormatArg> for InputFormat {
    fn from(value: InputFormatArg) -> Self {
        match value {
            InputFormatArg::Lua => Self::Lua,
            InputFormatArg::Json => Self::Json,
        }
    }
}

#[derive(Debug, Clone)]
enum OutputAction {
    Show {
        output: FileOrStdout,
    },
    Json {
        output: FileOrStdout,
    },
    Momotalk {
        output: Option<FileOrStdout>,
        plurality: OutputPlurality,
    },
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
enum OutputPlurality {
    #[default]
    Auto,
    Single,
    Multi,
}

impl From<OutputPluralityArg> for OutputPlurality {
    fn from(value: OutputPluralityArg) -> Self {
        match value {
            OutputPluralityArg::Auto => Self::Auto,
            OutputPluralityArg::Single => Self::Single,
            OutputPluralityArg::Multi => Self::Multi,
        }
    }
}

impl TryFrom<AppArgs> for App<state::Initial> {
    type Error = miette::Report;

    fn try_from(value: AppArgs) -> Result<Self, Self::Error> {
        let args = value;

        miette::set_panic_hook();
        env_logger::Builder::new()
            .filter_level(args.verbose.log_level_filter())
            .init();
        conf::try_init_app_config()?;

        Self {
            action: args.command.pipe(Action::try_from)?.pipe(Rc::new),
            state: state::Initial,
        }
        .pipe(Ok)
    }
}

impl Runnable for App<state::Initial> {
    fn run(self) -> Result<()> {
        match self.action.pipe_ref(Rc::clone).as_ref() {
            Action::Generate { action } => match action {
                GenerateAction::Example { lang } => match lang {
                    ExampleLang::En => Self::print_example_en(),
                    ExampleLang::ZhHans => Self::print_example_zh_hans(),
                },

                GenerateAction::Completion { shell } => {
                    generate::completion(*shell, &mut io::stdout())
                }

                GenerateAction::Asset { asset } => match asset {
                    Asset::LuaInputExampleEn => Self::print_example_en(),
                    Asset::LuaInputExampleZhHans => Self::print_example_zh_hans(),
                    Asset::LuaLibTalk => Self::print_asset_str(luatalk::assets::lua::lib::talk()),
                    Asset::TypstOutput => Self::print_asset_str(luatalk::assets::typst::output()),
                },

                GenerateAction::ConfigHelp => generate::help_config(),
            },

            Action::Do {
                input,
                input_format,
                concat_pages,
                ..
            } => self
                .process_input(input.clone(), *input_format, *concat_pages)?
                .run()?,
        }
        Ok(())
    }
}

impl App<state::Initial> {
    fn process_input(
        self,
        input: FileOrStdin,
        input_format: InputFormat,
        concat_pages: bool,
    ) -> Result<App<state::OfArticle>> {
        let source = input
            .clone()
            .contents()
            .into_diagnostic()
            .wrap_err("Input file not found")?;

        let article = match input_format {
            InputFormat::Lua => {
                let lua = {
                    let lua = Lua::new();
                    let do_lua_input_config = conf::app_config().do_lua();
                    if !do_lua_input_config.no_default_lib() {
                        lua.load_default_lib().into_diagnostic()?;
                    }
                    let additional_path = do_lua_input_config.additional_path();
                    if !additional_path.is_empty() {
                        lua.append_left_additional_path(additional_path)
                            .into_diagnostic()?;
                    }
                    lua
                };

                dto::Article::try_from_chunk(source, &lua)
                    .into_diagnostic()?
                    .pipe(Article::try_from)
                    .into_diagnostic()?
            }

            InputFormat::Json => serde_json::from_str::<dto::Article>(&source)
                .into_diagnostic()
                .wrap_err("Failed to parse JSON input")?
                .try_into()
                .into_diagnostic()?,
        }
        .pipe(|article| {
            if concat_pages {
                article.concat_pages()
            } else {
                article
            }
        })
        .tap(|article| {
            if article.pages().is_empty() {
                warn!("The article has no pages.");
            }
        });

        debug!("Build article success");

        App {
            state: state::OfArticle { article, input },
            action: self.action,
        }
        .pipe(Ok)
    }

    #[inline]
    fn print_example_en() {
        Self::print_asset_str(luatalk::assets::lua::input::example_en())
    }

    #[inline]
    fn print_example_zh_hans() {
        Self::print_asset_str(luatalk::assets::lua::input::example_zh_hans())
    }

    #[inline]
    fn print_asset_str(asset_str: &str) {
        println!("{asset_str}")
    }
}

impl Runnable for App<state::OfArticle> {
    fn run(self) -> Result<()> {
        match self.action.pipe_ref(Rc::clone).as_ref() {
            Action::Do { output, .. } => match output {
                OutputAction::Show { output } => self.output_show(output),
                OutputAction::Json { output } => self.output_json(output),
                OutputAction::Momotalk { output, plurality } => {
                    self.output_momotalk(output.as_ref(), plurality)
                }
            },

            _ => Err(diagnostic!("Invalid action for this state").into()),
        }
    }
}

impl App<state::OfArticle> {
    #[inline]
    fn output_show(self, output: &FileOrStdout) -> Result<()> {
        let mut writer = output.clone_to_ouptut_writer()?;
        writeln!(writer, "{:#?}", self.state.article).into_diagnostic()?;
        writer.flush().into_diagnostic()
    }

    #[inline]
    fn output_json(self, output: &FileOrStdout) -> Result<()> {
        let article: dto::Article = self.state.article.into();
        Self::output_json_to_writer(&mut output.clone_to_ouptut_writer()?, &article)
    }

    #[inline]
    fn output_momotalk(
        self,
        output: Option<&FileOrStdout>,
        plurality: &OutputPlurality,
    ) -> Result<()> {
        match plurality {
            OutputPlurality::Auto => {
                if self.state.article.pages().len() > 1 {
                    self.output_momotalk_multi(output)
                } else {
                    self.output_momotalk_single(output)
                }
            }

            OutputPlurality::Single => self.output_momotalk_single(output),
            OutputPlurality::Multi => self.output_momotalk_multi(output),
        }
    }

    fn output_momotalk_single(self, output: Option<&FileOrStdout>) -> Result<()> {
        let output = if let Some(output) = output {
            output.clone()
        } else {
            FileOrStdout::from_str("-").into_diagnostic()?
        };

        let (lang, pages) = self
            .state
            .article
            .pipe(|article| (article.lang(), article.into_pages()));
        if pages.len() > 1 {
            return Err(diagnostic!(
                "Article has multiple pages, but output plurality is set to single"
            )
            .into());
        }
        let talk_history = if let Some(page) = pages.into_iter().next() {
            page.into_msgs()
                .into_and_lang(lang)
                .try_into()
                .into_diagnostic()?
        } else {
            Vec::new()
        };

        let momotalk_export = momotalk::MomotalkExport {
            talk_id: 1,
            talk_history,
            select_list: Vec::new(),
        };
        debug!("Build MomotalkExport structure success");

        Self::output_json_to_writer(&mut output.clone_to_ouptut_writer()?, &momotalk_export)
    }

    fn output_momotalk_multi(self, output: Option<&FileOrStdout>) -> Result<()> {
        let input = || &self.state.input;
        let path = {
            let strfmt_re =
                Regex::new(formatcp!(r"\{{{}(?::.*)?\}}", INDEX_KEY)).into_diagnostic()?;

            let certain_dir;
            let path = if let Some(output) = &output {
                certain_dir = false;
                if output.is_file() {
                    output.filename()
                } else {
                    return Err(diagnostic!(
                        "Output file cannot be a file to export article in pages."
                    )
                    .into());
                }
                .to_owned()
            } else {
                certain_dir = true;
                input().filename_or_default(DEFAULT_OUTPUTNAME)?
            };

            if !certain_dir && strfmt_re.is_match(&path) {
                debug!("Output file pattern: {path}");
                MultiPath::Fmtstr(path.to_owned())
            } else {
                let path = path.tap(|p| debug!("Output dir: {p}")).into();
                let filename = input().filename_or_default(DEFAULT_OUTPUTNAME)?;
                MultiPath::Dir { path, filename }
            }
        };

        let momotalk_exports: Vec<momotalk::MomotalkExport> =
            self.state.article.try_into().into_diagnostic()?;
        let width_auto = {
            let size = momotalk_exports.len();
            let _: i32 = size
                .try_into()
                .map_err(momotalk::MomotalkExportError::TryFromInt)
                .into_diagnostic()?;
            if size == 0 {
                return Err(diagnostic!("No page to export").into());
            }
            size.ilog10() as usize + 1
        };
        let mut momotalk_exports = momotalk_exports.into_iter().zip(1..);
        match path {
            MultiPath::Dir { path, filename } => {
                Self::check_or_create_dir(&path)?;
                momotalk_exports.try_for_each(|(momotalk_export, i)| {
                    let path = path.join(format!("{filename}_{:width_auto$}.json", i,));
                    Self::ouput_momotalk_page_to_file(&momotalk_export, i, &path)
                })
            }

            MultiPath::Fmtstr(fmtstr) => {
                let mut vars = HashMap::with_capacity(1);
                momotalk_exports.try_for_each(|(momotalk_export, i)| {
                    vars.insert(INDEX_KEY.to_owned(), i.to_string());
                    let path = strfmt::strfmt(&fmtstr, &vars)
                        .into_diagnostic()
                        .wrap_err("Failed to format output file path string")?
                        .pipe(PathBuf::from);
                    path.parent().map(Self::check_or_create_dir).transpose()?;
                    Self::ouput_momotalk_page_to_file(&momotalk_export, i, &path)
                })
            }
        }
    }

    fn output_json_to_writer<W, T>(mut writer: &mut W, value: &T) -> Result<()>
    where
        W: Write,
        T: ?Sized + serde::Serialize,
    {
        serde_json::to_writer_pretty(&mut writer, value).into_diagnostic()?;
        writer.write_all(b"\n").into_diagnostic()?;
        writer.flush().into_diagnostic()
    }

    fn check_or_create_dir(path: &Path) -> Result<()> {
        if let Ok(metadata) = path.metadata() {
            if metadata.is_file() {
                return Err(diagnostic!(
                    "A file exists already has path of output destination directory: {}",
                    path.display()
                )
                .into());
            }
        } else {
            eprintln!("Creating directory: {}", path.display());
            fs::create_dir_all(path).into_diagnostic()?;
        }
        Ok(())
    }

    fn ouput_momotalk_page_to_file(
        momotalk_export: &momotalk::MomotalkExport,
        i: usize,
        path: &Path,
    ) -> Result<()> {
        let path_display = path.display();
        debug!("Output page {i} to: {path_display}");
        let mut writer = fs::File::create(path)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to create output file: {path_display}"))?;
        Self::output_json_to_writer(&mut writer, &momotalk_export)
    }
}

trait FileOrStdinExt {
    fn filename_or_default(&self, default: &str) -> Result<String>;
}

impl FileOrStdinExt for FileOrStdin {
    fn filename_or_default(&self, default: &str) -> Result<String> {
        if self.is_stdin() {
            default.to_owned()
        } else {
            self.filename()
                .pipe(Path::new)
                .file_stem()
                .ok_or_else(|| {
                    diagnostic!(
                        "Input file path does not have a valid file name: {}",
                        self.filename()
                    )
                })?
                .to_string_lossy()
                .into_owned()
        }
        .pipe(Ok)
    }
}

trait FileOrStdoutExt {
    fn clone_to_ouptut_writer(&self) -> Result<impl Write>;
}

impl FileOrStdoutExt for FileOrStdout {
    fn clone_to_ouptut_writer(&self) -> Result<impl Write> {
        debug!("Output to: {}", self.filename());
        self.clone().into_writer().into_diagnostic()?.pipe(Ok)
    }
}

#[derive(Debug, Clone)]
enum MultiPath {
    Fmtstr(String),
    Dir { path: PathBuf, filename: String },
}

mod state {
    use super::*;

    pub trait State {}

    pub struct Initial;

    impl State for Initial {}

    pub struct OfArticle {
        pub article: Article,
        pub input: FileOrStdin,
    }

    impl State for OfArticle {}
}
