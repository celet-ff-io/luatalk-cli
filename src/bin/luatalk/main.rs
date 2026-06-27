mod app;
mod cli;

use clap::Parser;
use miette::Result;

use crate::{
    app::{App, Runnable},
    cli::CliArgs,
};

fn main() -> Result<()> {
    App::try_from(CliArgs::parse())?.run()
}
