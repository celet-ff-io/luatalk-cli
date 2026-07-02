mod app;
mod cli;

use clap::Parser;
use miette::Result;
use tap::Pipe;

use crate::{
    app::{App, Runnable},
    cli::AppArgs,
};

fn main() -> Result<()> {
    AppArgs::parse().pipe(App::try_from)?.run()
}
