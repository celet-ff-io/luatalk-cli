mod app;
mod cli;

use clap::Parser;
use miette::Result;
use tap::Pipe;

use crate::{
    app::{App, Runnable},
    cli::CliArgs,
};

fn main() -> Result<()> {
    CliArgs::parse().pipe(App::try_from)?.run()
}
