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
use mlua::{Lua, Table};
use regex::Regex;
use tap::{Pipe, Tap};

use luatalk::{Article, InLang, IntoAndLang, LuaTalkExt, Msg, lua, momotalk};

use crate::{
    app::state::State,
    cli::{self, Args, AssetArg, Commands, GenerateCommands, LuaInputArgs, OutputFormatArg},
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

    Show {
        lua_input: LuaInput,
        output_dest: FileOrStdout,
    },

    Export {
        lua_input: LuaInput,
        format: OutputFormat,
        output_dest: MultiPurposeWriter,
    },
}

#[derive(Debug, Clone)]
enum GenerateAction {
    Example,
    Completion { shell: clap_complete::Shell },
    Asset { asset: Asset },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Asset {
    LuaInputExample,
    LuaLibTalk,
}

impl From<AssetArg> for Asset {
    fn from(value: AssetArg) -> Self {
        match value {
            AssetArg::LuaInputExample => Self::LuaInputExample,
            AssetArg::LuaLibTalk => Self::LuaLibTalk,
        }
    }
}

#[derive(Debug, Clone)]
struct LuaInput {
    source: String,
    enable_lib_default: bool,
    path_addtion: String,

    lua: Lua,
}

impl From<LuaInputArgs> for LuaInput {
    fn from(value: LuaInputArgs) -> Self {
        let LuaInputArgs {
            input,
            lib_default,
            libs,
        } = value;

        let source = input
            .contents()
            .into_diagnostic()
            .wrap_err("Input file not found")
            .unwrap_or_default();

        let enable_lib_default = lib_default;

        let path_addtion = libs
            .iter()
            .filter(|p| {
                p.is_dir().tap(|&is| {
                    if is {
                        debug!("Add directory path: {}", p.display());
                    } else {
                        warn!("Ignore bad directory path: {}", p.display());
                    }
                })
            })
            .flat_map(|p| {
                [
                    Self::as_lua_path_entry_or_empty(p, "?.lua"),
                    Self::as_lua_path_entry_or_empty(p, "?/init.lua"),
                ]
            })
            .collect::<String>();

        let lua = Lua::new();

        Self {
            source,
            enable_lib_default,
            path_addtion,
            lua,
        }
    }
}

impl LuaInput {
    #[inline]
    fn as_lua_path_entry_or_empty(dir: &Path, file: &str) -> String {
        dir.join(file)
            .to_str()
            .map_or(String::new(), |s| s.to_owned() + ";")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum OutputFormat {
    Momotalk,
}

#[derive(Debug, Clone)]
enum MultiPurposeWriter {
    Single(FileOrStdout),

    Multi(MultiPath),
}

#[derive(Debug, Clone)]
enum MultiPath {
    Fmtstr(String),
    Dir { path: PathBuf, filename: String },
}

impl TryFrom<Args> for App<state::Initial> {
    type Error = miette::Report;

    fn try_from(value: Args) -> Result<Self, Self::Error> {
        let args = value;

        env_logger::Builder::new()
            .filter_level(args.verbose.log_level_filter())
            .init();

        let action = match args.command {
            Commands::Generate { command } => Action::Generate {
                action: match command {
                    GenerateCommands::Example => GenerateAction::Example,
                    GenerateCommands::Completion { shell } => GenerateAction::Completion { shell },
                    GenerateCommands::Asset { asset } => GenerateAction::Asset {
                        asset: asset.into(),
                    },
                },
            },

            Commands::Show { lua_input_args } => Action::Show {
                lua_input: lua_input_args.into(),
                output_dest: FileOrStdout::from_str("-").into_diagnostic()?,
            },

            Commands::Export {
                lua_input_args,
                output,
                format,
                concat_pages,
            } => {
                let format = match format {
                    OutputFormatArg::Momotalk => OutputFormat::Momotalk,
                };
                let output_dest = if concat_pages {
                    if let Some(output) = output {
                        output
                    } else {
                        FileOrStdout::from_str("-").into_diagnostic()?
                    }
                    .pipe(MultiPurposeWriter::Single)
                } else {
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
                        lua_input_args
                            .input
                            .filename_or_default(DEFAULT_OUTPUTNAME)?
                    };

                    if !certain_dir && strfmt_re.is_match(&path) {
                        debug!("Output file pattern: {path}");
                        MultiPath::Fmtstr(path.to_owned())
                    } else {
                        let path = path.tap(|p| debug!("Output dir: {p}")).into();
                        let filename = lua_input_args
                            .input
                            .filename_or_default(DEFAULT_OUTPUTNAME)?;
                        MultiPath::Dir { path, filename }
                    }
                    .pipe(MultiPurposeWriter::Multi)
                };
                let lua_input = lua_input_args.into();
                Action::Export {
                    lua_input,
                    format,
                    output_dest,
                }
            }
        }
        .pipe(Rc::new);

        Ok(Self {
            action,
            state: state::Initial,
        })
    }
}

trait FilenameOrDefault {
    fn filename_or_default(&self, default: &str) -> Result<String>;
}

impl FilenameOrDefault for FileOrStdin {
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

impl Runnable for App<state::Initial> {
    fn run(self) -> Result<()> {
        match self.action.pipe_ref(Rc::clone).as_ref() {
            Action::Generate { action } => match action {
                GenerateAction::Example => Self::print_example(),

                GenerateAction::Completion { shell } => {
                    cli::generate_completion(*shell, &mut io::stdout());
                    Ok(())
                }

                GenerateAction::Asset { asset } => match asset {
                    Asset::LuaInputExample => Self::print_example(),
                    Asset::LuaLibTalk => {
                        Self::print_asset_str(include_str!("../../../assets/lua/lib/talk.lua"))
                    }
                },
            },

            Action::Show { lua_input, .. } => self.lua_input(lua_input)?.run(),

            Action::Export { lua_input, .. } => self.lua_input(lua_input)?.run(),
        }
    }
}

impl App<state::Initial> {
    fn lua_input(self, lua_input: &LuaInput) -> Result<App<state::OfArticle>> {
        let LuaInput {
            source,
            enable_lib_default,
            path_addtion,
            lua,
        } = lua_input;
        if *enable_lib_default {
            debug!("Loading default lib");
            lua.load_default_lib().into_diagnostic()?;
        }

        if !path_addtion.is_empty() {
            const KEY_PATH: &str = "path";
            let pacakges: Table = lua.globals().get("package").into_diagnostic()?;
            let current_path: String = pacakges.get(KEY_PATH).into_diagnostic()?;
            let new_path = format!("{path_addtion}{current_path}");
            debug!("Lua package path will update to {new_path}");
            pacakges.set(KEY_PATH, new_path).into_diagnostic()?;
        }

        let article = lua::Article::try_from_chunk(source, lua)
            .into_diagnostic()?
            .pipe(Article::from);

        debug!("Build article success");

        App {
            state: state::OfArticle { article },
            action: self.action,
        }
        .pipe(Ok)
    }

    fn print_example() -> Result<()> {
        Self::print_asset_str(include_str!("../../../assets/lua/input/example.lua"))
    }

    #[inline]
    fn print_asset_str(asset_str: &str) -> Result<()> {
        println!("{asset_str}");
        Ok(())
    }
}

impl Runnable for App<state::OfArticle> {
    fn run(self) -> Result<()> {
        match self.action.as_ref() {
            Action::Show { output_dest, .. } => {
                let mut writer = output_dest.clone().into_writer().into_diagnostic()?;
                writeln!(writer, "{:#?}", self.state.article).into_diagnostic()?;
                writer.flush().into_diagnostic()
            }

            Action::Export {
                format,
                output_dest,
                ..
            } => {
                if format != &OutputFormat::Momotalk {
                    return Err(diagnostic!("Only Momotalk format is supported").into());
                }
                match output_dest {
                    MultiPurposeWriter::Single(output_dest) => {
                        let article = self.state.article;
                        let lang = article.lang();
                        let talk_history = article
                            .into_pages()
                            .into_iter()
                            .flatten()
                            .collect::<Vec<Msg>>()
                            .into_and_lang(lang)
                            .try_into()
                            .into_diagnostic()?;
                        let momotalk_export = momotalk::MomotalkExport {
                            talk_id: 1,
                            talk_history,
                            select_list: Vec::new(),
                        };
                        debug!("Build MomotalkExport structure success");

                        debug!("Output to: {}", output_dest.filename());
                        let mut writer = output_dest.clone().into_writer().into_diagnostic()?;
                        Self::export_momotalk_to_writer(&momotalk_export, &mut writer)
                    }

                    MultiPurposeWriter::Multi(path) => {
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
                                Self::check_or_create_dir(path)?;
                                momotalk_exports.try_for_each(|(momotalk_export, i)| {
                                    let path =
                                        path.join(format!("{filename}_{:width_auto$}.json", i,));
                                    Self::export_momotalk_page_to_file(&momotalk_export, i, &path)
                                })
                            }

                            MultiPath::Fmtstr(fmtstr) => {
                                let mut vars = HashMap::with_capacity(1);
                                momotalk_exports.try_for_each(|(momotalk_export, i)| {
                                    vars.insert(INDEX_KEY.to_owned(), i.to_string());
                                    let path = strfmt::strfmt(fmtstr, &vars)
                                        .into_diagnostic()
                                        .wrap_err("Failed to format output file path string")?
                                        .pipe(PathBuf::from);
                                    path.parent().map(Self::check_or_create_dir).transpose()?;
                                    Self::export_momotalk_page_to_file(&momotalk_export, i, &path)
                                })
                            }
                        }
                    }
                }
            }

            _ => Err(diagnostic!("Invalid action for this state").into()),
        }
    }
}

impl App<state::OfArticle> {
    fn export_momotalk_to_writer<W: Write>(
        momotalk_export: &momotalk::MomotalkExport,
        mut writer: &mut W,
    ) -> Result<()> {
        serde_json::to_writer_pretty(&mut writer, &momotalk_export).into_diagnostic()?;
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

    fn export_momotalk_page_to_file(
        momotalk_export: &momotalk::MomotalkExport,
        i: usize,
        path: &Path,
    ) -> Result<()> {
        let path_display = path.display();
        debug!("Output page {i} to: {path_display}");
        let mut writer = fs::File::create(path)
            .into_diagnostic()
            .wrap_err_with(|| format!("Failed to create output file: {path_display}"))?;
        Self::export_momotalk_to_writer(momotalk_export, &mut writer)
    }
}

mod state {
    use super::*;

    pub trait State {}

    pub struct Initial;

    impl State for Initial {}

    pub struct OfArticle {
        pub article: Article,
    }

    impl State for OfArticle {}
}
