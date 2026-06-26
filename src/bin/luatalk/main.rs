mod app;

use clap::Parser;
use miette::Result;

use crate::app::App;

fn main() -> Result<()> {
    App::parse().run()
}
