/// For subcommand `do`.
use std::{
    cell::LazyCell,
    collections::HashMap,
    fs,
    io::Write,
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
    app::{
        Action, App, Runnable,
        common::FileOrStdoutExt,
        generate::*,
        state::{self},
    },
    cli::do_::{InputFormatArg, OutputCommand, OutputPluralityArg, TypstCompileFormatArg},
    conf,
};

const DEFAULT_OUTPUT_STEM: &str = "output";
const PAGE_NUMBER_PLACEHOLDER: &str = "p";

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum InputFormat {
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
pub enum OutputAction {
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
pub enum TypstCompileFormat {
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
pub enum OutputPlurality {
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

// Action

impl App<state::Initial> {
    pub fn process_input(
        self,
        input: &FileOrStdin,
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
            state: state::OfArticle {
                article,
                input: input.clone(),
            },
            action: self.action,
        }
        .pipe(Ok)
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
                    self.output_typst_compile(output.as_ref(), *format)
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
        let mut writer = output.clone_to_output_writer()?;
        writeln!(writer, "{:#?}", self.state.article).into_diagnostic()?;
        writer.flush().into_diagnostic()
    }

    #[inline]
    fn output_json(self, output: &FileOrStdout) -> Result<()> {
        let article: dto::Article = self.state.article.into();
        Self::output_json_to_writer(&mut output.clone_to_output_writer()?, &article)
    }

    #[inline]
    fn output_typst_and_json(
        self,
        stem: Option<&String>,
        config: &TypstOutputConfig,
    ) -> Result<()> {
        let stem = stem
            .cloned()
            .unwrap_or_else(|| self.state.input.file_stem_or(DEFAULT_OUTPUT_STEM));
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
        format: Option<TypstCompileFormat>,
    ) -> Result<()> {
        let format: TypstCompileFormat = if let Some(format) = format {
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
                "pdf" => TypstCompileFormat::Pdf,
                "png" => TypstCompileFormat::Png,
                _ => {
                    return Err(diagnostic!(
                        help = HELP,
                        "Cannot infer output format from file extension: {ext}"
                    )
                    .into());
                }
            }
        };
        let ext_for_single = match format {
            TypstCompileFormat::Pdf => Some("pdf"),
            TypstCompileFormat::Png => None,
        };
        if let Some(ext) = ext_for_single {
            let output = if let Some(output) = output {
                output.to_owned()
            } else {
                format!("{}.{ext}", self.input_stem())
            };
            self.output_typst_compile_single(output, format)
        } else {
            let output = {
                let strfmt_re =
                    Regex::new(formatcp!(r"\{{0?{}(?::.*)?\}}", PAGE_NUMBER_PLACEHOLDER))
                        .into_diagnostic()?;

                let stem = LazyCell::new(|| self.input_stem());
                let maybe_format_str = output.is_some();
                let path = output.unwrap_or_else(|| &stem);

                if maybe_format_str && strfmt_re.is_match(path) {
                    debug!("Output file pattern: {path}");
                    path.to_owned()
                } else {
                    debug!("Output dir: {path}");
                    let stem: &str = &stem;
                    format!("{path}/{stem}_{{0p}}.png")
                }
            };
            self.ouput_typst_compile_multi(output, &format)
        }
    }

    fn output_typst_compile_single(self, output: String, format: TypstCompileFormat) -> Result<()> {
        let _ = output;
        match format {
            TypstCompileFormat::Pdf => {
                // todo
            }
            _ => {
                return Err(diagnostic!(
                    "Output format is not supported for single-page output: {format:?}"
                )
                .into());
            }
        }
        Ok(())
    }

    fn ouput_typst_compile_multi(self, output: String, format: &TypstCompileFormat) -> Result<()> {
        let _ = output;
        match format {
            TypstCompileFormat::Png => {
                // todo
            }
            _ => {
                return Err(diagnostic!(
                    "Output format is not supported for multi-page output: {format:?}"
                )
                .into());
            }
        }
        Ok(())
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

        Self::output_json_to_writer(&mut output.clone_to_output_writer()?, &momotalk_export)
    }

    fn output_momotalk_multi(self, output: Option<&FileOrStdout>) -> Result<()> {
        let path = {
            let strfmt_re = Regex::new(formatcp!(r"\{{0?{}(?::.*)?\}}", PAGE_NUMBER_PLACEHOLDER))
                .into_diagnostic()?;

            let maybe_format_str = output.is_some();
            let path = if let Some(output) = output {
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
                self.input_stem()
            };

            if maybe_format_str && strfmt_re.is_match(&path) {
                debug!("Output file pattern: {path}");
                MultiPath::Fmtstr(path.to_owned())
            } else {
                debug!("Output dir: {path}");
                let path = path.into();
                let filename = self.input_stem();
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

    #[inline]
    fn input_stem(&self) -> String {
        self.state.input.file_stem_or(DEFAULT_OUTPUT_STEM)
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

#[derive(Debug, Clone)]
enum MultiPath {
    Fmtstr(String),
    Dir { path: PathBuf, filename: String },
}

// Utils

trait PathExt {
    fn to_writer(&self) -> Result<impl Write>;
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
