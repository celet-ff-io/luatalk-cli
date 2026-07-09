use std::{
    collections::HashMap,
    fs,
    io::{self, Write, stdout},
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

use luatalk::{Article, InLang, IntoAndLang, LuaExt, assets, dto, momotalk};

use crate::{
    app::state::State,
    cli::{
        AppArgs, AppCommand,
        do_::{InputFormatArg, OutputCommand, OutputPluralityArg, TypstCompileFormatArg},
        generate::{self, AssetArg, ExampleLangArg, LicenseArg},
    },
    conf,
};

const DEFAULT_OUTPUTNAME: &str = "article";
const PAGE_NUMBER_PLACEHOLDER: &str = "p";

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
        output: FileOrStdout,
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
            AppCommand::Generate { output, command } => Self::Generate {
                output,
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
                let output = output_commands.into();
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

#[derive(Debug, Clone, PartialEq)]
enum GenerateAction {
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

impl From<generate::Command> for GenerateAction {
    fn from(value: generate::Command) -> Self {
        match value {
            generate::Command::Example { lang } => Self::Example { lang: lang.into() },
            generate::Command::Typst { data, config } => Self::Typst {
                data,
                config: config.into(),
            },
            generate::Command::Completion { shell } => Self::Completion { shell },
            generate::Command::Asset { asset } => Self::Asset {
                asset: asset.into(),
            },
            generate::Command::ConfigHelp => Self::ConfigHelp,
            generate::Command::License { license } => Self::License {
                license: license.into(),
            },
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

#[derive(Debug, Clone, PartialEq)]
struct TypstOutputConfig {
    font_size: u32,
    width: u32,
    font_family: String,
    length_factor: f32,
}

impl From<generate::TypstOutputConfigArgs> for TypstOutputConfig {
    fn from(value: generate::TypstOutputConfigArgs) -> Self {
        let generate::TypstOutputConfigArgs {
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
enum Asset {
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
enum License {
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
    Typst {
        stem: Option<String>,
        config: TypstOutputConfig,
    },
    TypstCompile {
        output: Option<String>,
        format: Option<TypstCompileFormat>,
    },
    Momotalk {
        output: Option<FileOrStdout>,
        plurality: OutputPlurality,
    },
}

impl From<OutputCommand> for OutputAction {
    fn from(value: OutputCommand) -> Self {
        match value {
            OutputCommand::Show { output } => Self::Show { output },
            OutputCommand::Json { output } => Self::Json { output },
            OutputCommand::Typst { stem, config } => Self::Typst {
                stem,
                config: config.into(),
            },
            OutputCommand::TypstCompile { output, format } => Self::TypstCompile {
                output,
                format: format.map(Into::into),
            },
            OutputCommand::Momotalk { output, pl } => Self::Momotalk {
                output,
                plurality: pl.into(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum TypstCompileFormat {
    Pdf,
    Png,
}

impl From<TypstCompileFormatArg> for TypstCompileFormat {
    fn from(value: TypstCompileFormatArg) -> Self {
        match value {
            TypstCompileFormatArg::Pdf => Self::Pdf,
            TypstCompileFormatArg::Png => Self::Png,
        }
    }
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

impl<S: state::State> App<S> {
    /// * `data` - **File path** of a JSON file
    fn output_typst<W: Write>(
        writer: &mut W,
        data: &str,
        config: &TypstOutputConfig,
    ) -> Result<()> {
        let TypstOutputConfig {
            font_size,
            width,
            font_family,
            length_factor,
        } = config;
        if *length_factor <= 0.0 {
            return Err(
                diagnostic!("Length factor must be positive, but got: {length_factor}").into(),
            );
        }
        let font_family: String = font_family.pipe_as_ref(json_escape::escape_str).collect();
        let length_factor_percent = length_factor * 100.0;
        let data: String = data.pipe(json_escape::escape_str).collect();
        writeln!(writer, "{}", assets::typst::output()).into_diagnostic()?;
        writeln!(
            writer,
            "#set text({font_size}pt)
#article-render(
  width: {width}pt,
  font: \"{font_family}\",
  length-factor: {length_factor_percent}%,
  json(\"{data}\"),
)
"
        )
        .into_diagnostic()?;
        writer.flush().into_diagnostic()?;
        Ok(())
    }
}

impl Runnable for App<state::Initial> {
    fn run(self) -> Result<()> {
        match self.action.pipe_ref(Rc::clone).as_ref() {
            Action::Generate { output, action } => {
                let mut w = output.clone_to_ouptut_writer()?;
                match action {
                    GenerateAction::Example { lang } => match lang {
                        ExampleLang::En => w.ouput_str(assets::lua::input::example_en())?,

                        ExampleLang::ZhHans => w.ouput_str(assets::lua::input::example_zh_hans())?,
                    },

                    GenerateAction::Typst { data, config } => {
                        Self::output_typst(&mut stdout(), data, config)?
                    }

                    GenerateAction::Completion { shell } => {
                        generate::completion(*shell, &mut io::stdout())
                    }

                    GenerateAction::Asset { asset } => match asset {
                        Asset::LuaInputExampleEn => w.ouput_str(assets::lua::input::example_en())?,
                        Asset::LuaInputExampleZhHans => {
                            w.ouput_str(assets::lua::input::example_zh_hans())?
                        }
                        Asset::LuaLibTalk => w.ouput_str(assets::lua::lib::talk())?,

                        Asset::TypstOutput => w.ouput_str(assets::typst::output())?,

                        Asset::LicenseNotice => w.ouput_str(assets::license::notice())?,
                        Asset::LicenseApache => w.ouput_str(assets::license::license_apache())?,
                        Asset::LicenseMit => w.ouput_str(assets::license::license_mit())?,
                        Asset::LicenseHtml => w.ouput_str(assets::license::license_html())?,
                    },

                    GenerateAction::ConfigHelp => generate::help_config(),

                    GenerateAction::License { license } => match license {
                        License::Notice => w.ouput_str(assets::license::notice())?,
                        License::Apache => w.ouput_str(assets::license::license_apache())?,
                        License::Mit => w.ouput_str(assets::license::license_mit())?,
                        License::ThirdPartyLicenses => {
                            w.ouput_str(assets::license::license_html())?
                        }
                    },
                }
            }

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
}

trait WriteExt {
    fn ouput_str(&mut self, s: &str) -> Result<()>;
}

impl<W: Write> WriteExt for W {
    fn ouput_str(&mut self, s: &str) -> Result<()> {
        writeln!(self, "{s}").into_diagnostic()?;
        self.flush().into_diagnostic()
    }
}

impl Runnable for App<state::OfArticle> {
    fn run(self) -> Result<()> {
        match self.action.pipe_ref(Rc::clone).as_ref() {
            Action::Do { output, .. } => match output {
                OutputAction::Show { output } => self.output_show(output),
                OutputAction::Json { output } => self.output_json(output),
                OutputAction::Typst { stem, config } => {
                    self.output_typst_and_json(stem.as_ref(), config)
                }
                OutputAction::TypstCompile { output, format } => {
                    self.output_typst_compile(output.as_ref(), format.as_ref())
                }
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
    fn output_typst_and_json(
        self,
        stem: Option<&String>,
        config: &TypstOutputConfig,
    ) -> Result<()> {
        let stem = stem
            .cloned()
            .unwrap_or_else(|| self.state.input.file_stem_or(DEFAULT_OUTPUTNAME));
        let data_filename = {
            let article: dto::Article = self.state.article.into();
            let path = stem.pipe_ref(Path::new).with_extension("json");
            let mut writer = path.to_writer()?;
            Self::output_json_to_writer(&mut writer, &article)?;

            let filename = path
                .file_name()
                .ok_or_else(|| diagnostic!("File name of path is not valid: {}", path.display()))?;
            filename
                .to_str()
                .ok_or_else(|| diagnostic!("File name is not valid UTF-8: {}", filename.display()))?
                .to_owned()
        };
        {
            let path = stem.pipe_ref(Path::new).with_extension("typ");
            let mut writer = path.to_writer()?;
            Self::output_typst(&mut writer, &data_filename, config)?;
        }
        Ok(())
    }

    #[inline]
    fn output_typst_compile(
        self,
        output: Option<&String>,
        format: Option<&TypstCompileFormat>,
    ) -> Result<()> {
        let format: &TypstCompileFormat = if let Some(format) = format {
            format
        } else {
            const HELP: &str = "Use `--format` to specify output format";
            let ext = output
                .ok_or_else(|| {
                    diagnostic!(
                        help = HELP,
                        "Cannot infer output format without file extension."
                    )
                })?
                .pipe(Path::new)
                .extension()
                .ok_or_else(|| {
                    diagnostic!(
                        help = HELP,
                        "Cannot infer output format without file extension."
                    )
                })?
                .to_str()
                .ok_or_else(|| {
                    diagnostic!(
                        help = HELP,
                        "Cannot infer output format from a file extension not valid UTF-8",
                    )
                })?;
            match ext {
                "pdf" => &TypstCompileFormat::Pdf,
                "png" => &TypstCompileFormat::Png,
                _ => {
                    return Err(diagnostic!(
                        help = HELP,
                        "Cannot infer output format from file extension: {ext}"
                    )
                    .into());
                }
            }
        };
        match format {
            TypstCompileFormat::Pdf => self.output_typst_compile_single(output, format),
            TypstCompileFormat::Png => self.ouput_typst_compile_multi(output, format),
        }
    }

    fn output_typst_compile_single(
        self,
        output: Option<&String>,
        format: &TypstCompileFormat,
    ) -> Result<()> {
        todo!()
    }

    fn ouput_typst_compile_multi(
        self,
        output: Option<&String>,
        format: &TypstCompileFormat,
    ) -> Result<()> {
        todo!()
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
            let strfmt_re = Regex::new(formatcp!(r"\{{{}(?::.*)?\}}", PAGE_NUMBER_PLACEHOLDER))
                .into_diagnostic()?;

            let certain_dir;
            let path = if let Some(output) = &output {
                certain_dir = false;
                if output.is_file() {
                    output.filename()
                } else {
                    return Err(diagnostic!(
                        "Output file cannot be stdout to export article in pages."
                    )
                    .into());
                }
                .to_owned()
            } else {
                certain_dir = true;
                input().file_stem_or(DEFAULT_OUTPUTNAME)
            };

            if !certain_dir && strfmt_re.is_match(&path) {
                debug!("Output file pattern: {path}");
                MultiPath::Fmtstr(path.to_owned())
            } else {
                let path = path.tap(|p| debug!("Output dir: {p}")).into();
                let filename = input().file_stem_or(DEFAULT_OUTPUTNAME);
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
                    vars.insert(PAGE_NUMBER_PLACEHOLDER.to_owned(), i.to_string());
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

trait PathExt {
    fn to_writer(&self) -> Result<impl Write>;
    fn file_name_or(&self, default: &str) -> String;
}

impl PathExt for Path {
    fn to_writer(&self) -> Result<impl Write> {
        let path_display = self.display();
        let writer = fs::File::create(self)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to create output file: {path_display}",))?;
        debug!("Output to: {path_display}");
        Ok(writer)
    }

    fn file_name_or(&self, default: &str) -> String {
        if let Some(name) = self.file_name() {
            name.to_string_lossy().into_owned()
        } else {
            default.to_owned()
        }
    }
}

trait FileOrStdinExt {
    fn file_stem_or(&self, default: &str) -> String;
}

impl FileOrStdinExt for FileOrStdin {
    fn file_stem_or(&self, default: &str) -> String {
        if self.is_file()
            && let Some(stem) = self.filename().pipe(Path::new).file_stem()
        {
            stem.to_string_lossy().into_owned()
        } else {
            default.to_owned()
        }
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
