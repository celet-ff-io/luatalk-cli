/// Common utils.
use std::io::Write;

use clap_stdin::FileOrStdout;
use log::debug;
use miette::{IntoDiagnostic, Result};
use tap::Pipe;

pub trait FileOrStdoutExt {
    fn clone_to_output_writer(&self) -> Result<impl Write>;
}

impl FileOrStdoutExt for FileOrStdout {
    fn clone_to_output_writer(&self) -> Result<impl Write> {
        debug!("Output to: {}", self.filename());
        self.clone().into_writer().into_diagnostic()?.pipe(Ok)
    }
}
