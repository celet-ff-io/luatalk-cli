/// For subcommand `do`.
use std::{
    cell::LazyCell,
    collections::HashMap,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process,
    rc::Rc,
    result,
    str::FromStr,
};

use clap_stdin::{FileOrStdin, FileOrStdout};
use const_format::formatcp;
use data_encoding::BASE32HEX_NOPAD;
use log::{debug, error, warn};
use miette::{IntoDiagnostic, Result, WrapErr, diagnostic};
use mlua::Lua;
use regex::Regex;
use semver::VersionReq;
use tap::{Pipe, Tap};

use luatalk::{Article, InLang, IntoAndLang, LuaExt, dto, momotalk};
use tempfile::NamedTempFile;
use xxhash_rust::xxh3::xxh3_64;

use crate::{
    app::{
        Action, App, Runnable,
        common::FileOrStdoutExt,
        generate::*,
        state::{self},
    },
    cli::do_::{
        InputFormatArg, OutputCommand, OutputPluralityArg, TypstCompileFormatArg,
        UrlFetchOptionsArgs,
    },
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
        options: TypstOutputOptions,
        url_fetch_options: UrlFetchOptions,
    },
    TypstCompile {
        output: Option<String>,
        format: Option<TypstCompileFormat>,
        options: TypstOutputOptions,
        url_fetch_options: UrlFetchOptions,
        keep_temp: bool,
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
            OutputCommand::Typst {
                stem,
                options,
                url_fetch_options,
            } => Self::Typst {
                stem,
                options: options.into(),
                url_fetch_options: url_fetch_options.into(),
            },
            OutputCommand::TypstCompile {
                output,
                format,
                options,
                url_fetch_options,
                keep_temp,
            } => Self::TypstCompile {
                output,
                format: format.map(Into::into),
                options: options.into(),
                url_fetch_options: url_fetch_options.into(),
                keep_temp,
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

impl From<TypstCompileFormat> for &str {
    fn from(value: TypstCompileFormat) -> &'static str {
        match value {
            TypstCompileFormat::Pdf => "pdf",
            TypstCompileFormat::Png => "png",
        }
    }
}

#[derive(Debug, Clone)]
pub struct UrlFetchOptions {
    pub offline: bool,
}

impl From<UrlFetchOptionsArgs> for UrlFetchOptions {
    fn from(value: UrlFetchOptionsArgs) -> Self {
        Self {
            offline: value.offline,
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
                OutputAction::Typst {
                    stem,
                    options,
                    url_fetch_options,
                } => self.output_typst_and_json(stem.as_ref(), options, url_fetch_options),
                OutputAction::TypstCompile {
                    output,
                    format,
                    options,
                    url_fetch_options,
                    keep_temp,
                } => self.output_typst_compile(
                    output.as_ref(),
                    *format,
                    options,
                    url_fetch_options,
                    *keep_temp,
                ),
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
        config: &TypstOutputOptions,
        url_fetch_options: &UrlFetchOptions,
    ) -> Result<()> {
        let article = self.state.article;
        let ensure_result = if url_fetch_options.offline {
            Ok(())
        } else {
            // Fetch data from URL if needed
            article.try_ensure_path().into_diagnostic()
        };
        let stem = stem
            .cloned()
            .unwrap_or_else(|| self.state.input.file_stem_or(DEFAULT_OUTPUT_STEM));
        let data_filename = {
            let path = stem.pipe_ref(Path::new).with_extension("json");
            let mut writer = path.to_writer()?;
            let article: dto::Article = article.into();
            Self::output_json_to_writer(&mut writer, &article)?;
            eprintln!("Output JSON file: {}", path.display());

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
            eprintln!("Output Typst file: {}", path.display());
        }
        ensure_result
    }

    #[inline]
    fn output_typst_compile(
        self,
        output: Option<&String>,
        format: Option<TypstCompileFormat>,
        config: &TypstOutputOptions,
        url_fetch_options: &UrlFetchOptions,
        keep_temp: bool,
    ) -> Result<()> {
        // Inputs
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
            match ext.to_lowercase().as_str() {
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
        let output = if let Some(ext) = match format {
            TypstCompileFormat::Pdf => format.pipe(Into::<&str>::into).into(),
            TypstCompileFormat::Png => None,
        } {
            if let Some(output) = output {
                output.to_owned()
            } else {
                format!("{}.{ext}", self.input_stem())
            }
        } else {
            let strfmt_re = Regex::new(formatcp!(r"\{{0?{}(?::.*)?\}}", PAGE_NUMBER_PLACEHOLDER))
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

        // Get typst-cli
        let typst_command = conf::app_config()
            .do_typst_compile()
            .typst_command()
            .pipe(|cmd| if cmd.is_empty() { "typst" } else { cmd });
        // Check which typst-cli
        {
            let path = which::which("typst").into_diagnostic().wrap_err(
                "Cannot find typst command in PATH.
Please install typst-cli (e.g. `cargo install typst-cli`) to PATH,
or specify the command with `LUATALK__DO_TYPST_COMPILE__TYPST_COMMAND` environment variable.",
            )?;
            eprintln!("Run typst-cli: {}", path.display());
        }
        // Check its version. Should never panic here.
        if let Err(err) = Self::check_typst_cli_version(typst_command) {
            warn!("Checking typst-cli version failed: {err:?}");
        }

        let article = self.state.article;
        debug!("Build article of absolute paths success");

        let ensure_result = if url_fetch_options.offline {
            Ok(())
        } else {
            // Fetch data from URL if needed
            debug!("Ensuring res available from URL");
            article.try_ensure_path().into_diagnostic()
        };

        let article: dto::Article = article.into();
        let article_json = article.pipe_ref(Self::output_json_to_vec)?;
        let prefix = {
            let article_hash = article_json
                .pipe_as_ref(xxh3_64)
                .to_be_bytes()
                .pipe_ref(|b| BASE32HEX_NOPAD.encode(b));
            format!("{DEFAULT_OUTPUT_STEM}-{article_hash}-")
        };

        let pwd = std::env::current_dir()
            .into_diagnostic()
            .wrap_err("Failed to get current working directory")?;
        // .json
        let mut kept_tmp_json = KeptTmp::try_new_kept_tmp(keep_temp, &pwd, &prefix, ".json")?;
        let tmp_json_file = kept_tmp_json.as_file_mut();
        tmp_json_file.write_all(&article_json).into_diagnostic()?;
        tmp_json_file.flush().into_diagnostic()?;
        let tmp_json_path = kept_tmp_json.path();
        debug!("Created temp .json file: {}", tmp_json_path.display());
        let tmp_json_file_name = tmp_json_path
            .file_name()
            .ok_or_else(|| {
                diagnostic!(
                    "Failed to get file name of temp .json file: {}",
                    tmp_json_path.display()
                )
            })?
            .to_str()
            .ok_or_else(|| {
                diagnostic!(
                    "Failed to convert file name of temp .json file to UTF-8: {}",
                    tmp_json_path.display()
                )
            })?
            .to_owned();
        // .typ
        let mut kept_tmp_typ = KeptTmp::try_new_kept_tmp(keep_temp, &pwd, &prefix, ".typ")?;
        let tmp_typ_file = kept_tmp_typ.as_file_mut();
        Self::output_typst(tmp_typ_file, &tmp_json_file_name, config)?;
        let tmp_typ_path = kept_tmp_typ.path();
        debug!("Created temp .typ file: {}", tmp_typ_path.display());

        // Do not run typst-cli if error occurred in fetching
        ensure_result?;

        // Run typst-cli
        let output = process::Command::new(typst_command)
            .arg("compile")
            .arg("--format")
            .arg(match format {
                TypstCompileFormat::Pdf => "pdf",
                TypstCompileFormat::Png => "png",
            })
            .arg(tmp_typ_path)
            .arg(&output)
            .output()
            .into_diagnostic()
            .wrap_err("Failed to run typst command")?;
        if !output.status.success() {
            error!(
                "typst command stderr: {}",
                output.stderr.pipe_as_ref(String::from_utf8_lossy)
            );
            return Err(diagnostic!(
                "typst command failed with status: {}. See stderr for details.",
                output.status
            )
            .into());
        }
        let stdout = output.stdout.pipe_as_ref(String::from_utf8_lossy);
        println!("{stdout}");

        Ok(())
    }

    #[inline]
    fn check_typst_cli_version(typst_cmd: &str) -> Result<()> {
        let version = process::Command::new(typst_cmd)
            .arg("--version")
            .output()
            .into_diagnostic()
            .wrap_err("Failed to run typst command")?
            .pipe(|output| {
                output
                    .stdout
                    .pipe(String::from_utf8)
                    .into_diagnostic()
                    .wrap_err("Failed to parse typst command output as UTF-8")
            })?
            .pipe_as_ref(extract_version)?;
        debug!("Using typst-cli version: {version}");
        let req = VersionReq::parse(">=0.15.0").into_diagnostic()?;
        let version = semver::Version::parse(&version).into_diagnostic()?;
        if !req.matches(&version) {
            return Err(diagnostic!(
                "typst-cli version {version} does not satisfy requirement {req}"
            )
            .into());
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

    fn output_json_to_writer(
        mut writer: &mut impl Write,
        value: &impl serde::Serialize,
    ) -> Result<()> {
        if *conf::app_config().do_json().minify() {
            serde_json::to_writer(&mut writer, value)
        } else {
            serde_json::to_writer_pretty(&mut writer, value)
        }
        .into_diagnostic()?;
        writer.write_all(b"\n").into_diagnostic()?;
        writer.flush().into_diagnostic()
    }

    fn output_json_to_vec(value: &impl serde::Serialize) -> Result<Vec<u8>> {
        let mut writer = Vec::with_capacity(128);
        Self::output_json_to_writer(&mut writer, value)?;
        Ok(writer)
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

trait KeptTmpExt {
    fn try_new_kept_tmp(keep_temp: bool, dir: &Path, prefix: &str, suffix: &str)
    -> Result<KeptTmp>;

    fn path(&self) -> &Path;

    fn as_file_mut(&mut self) -> &mut File;
}

type KeptTmp = result::Result<(File, PathBuf), NamedTempFile>;

impl KeptTmpExt for KeptTmp {
    fn try_new_kept_tmp(
        keep_temp: bool,
        dir: &Path,
        prefix: &str,
        suffix: &str,
    ) -> Result<KeptTmp> {
        let tmp = tempfile::Builder::new()
            .prefix(prefix)
            .suffix(suffix)
            .tempfile_in(dir)
            .into_diagnostic()?;

        if keep_temp {
            tmp.keep().into_diagnostic()?.pipe(Ok)
        } else {
            tmp.pipe(Err)
        }
        .pipe(Ok)
    }

    fn path(&self) -> &Path {
        match self {
            Ok((_, path)) => path.as_path(),
            Err(tmp) => tmp.path(),
        }
    }

    fn as_file_mut(&mut self) -> &mut File {
        match self {
            Ok((file, _)) => file,
            Err(tmp) => tmp.as_file_mut(),
        }
    }
}

fn extract_version(output: &str) -> Result<String> {
    let re = Regex::new(
        r"(?i)(?:version\s+|v)?(?P<version>\d+\.\d+\.\d+(?:-[a-zA-Z0-9.]+)?(?:\+[a-zA-Z0-9.]+)?)",
    )
    .into_diagnostic()?;

    re.captures(output)
        .and_then(|caps| caps.name("version").map(|m| m.as_str().to_string()))
        .ok_or_else(|| diagnostic!("Failed to extract version from output: {}", output))
        .into_diagnostic()
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn test_extract_version() {
        let test_cases = vec![
            ("typst 0.15.0 (unknown commit)", Some("0.15.0".to_owned())),
            ("typst (unknown commit)", None),
        ];

        for (input, expected) in test_cases {
            assert_eq!(extract_version(input).ok(), expected);
        }
    }
}
