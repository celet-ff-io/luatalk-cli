mod app;
mod cli;
mod conf;
mod locale;

use clap::{CommandFactory, FromArgMatches};
use miette::Result;
use tap::Pipe;

use crate::{
    app::{App, Runnable},
    cli::AppArgs,
    locale::Localize,
};

fn main() -> Result<()> {
    AppArgs::command()
        .localize(conf::lang())
        .get_matches()
        .pipe_ref_mut(AppArgs::from_arg_matches_mut)
        .map_err(|e| e.format(&mut AppArgs::command()))
        .unwrap_or_else(|e| e.exit())
        .pipe(App::try_from)?
        .run()
}
