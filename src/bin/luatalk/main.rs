mod app;

use clap::Parser;
use miette::Result;

use crate::app::{App, Args};

fn main() -> Result<()> {
    App::new(Args::parse())?.run()
}
