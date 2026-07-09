mod common;
mod do_;
mod generate;

use std::{path::Path, rc::Rc};

use clap_stdin::{FileOrStdin, FileOrStdout};
use miette::{Result, diagnostic};
use tap::Pipe;

use luatalk::Article;

use crate::{
    app::{do_::*, generate::*, state::State},
    cli::{AppArgs, AppCommand},
    conf,
};

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
            Action::Generate { output, action } => self.run_generate(output, action)?,

            Action::Do {
                input,
                input_format,
                concat_pages,
                ..
            } => self
                .process_input(input, *input_format, *concat_pages)?
                .run()?,
        }
        Ok(())
    }
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
