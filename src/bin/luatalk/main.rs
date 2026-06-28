mod app;
mod cli;

use clap::Parser;
use miette::Result;
use tap::Pipe;

use crate::{
    app::{App, Runnable},
    cli::Args,
};

fn main() -> Result<()> {
    Args::parse().pipe(App::try_from)?.run()
}
