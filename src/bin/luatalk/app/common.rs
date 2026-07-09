/// Common codes.
use std::io::Write;

use clap_stdin::FileOrStdout;
use log::debug;
use luatalk::assets;
use miette::{IntoDiagnostic, Result, diagnostic};
use tap::Pipe;

use crate::app::{App, generate::TypstOutputConfig, state};

// Action

impl<S: state::State> App<S> {
    /// * `data` - **File path** of a JSON file
    pub fn output_typst<W: Write>(
        writer: &mut W,
        data: &str,
        config: &TypstOutputConfig,
    ) -> Result<()> {
        let TypstOutputConfig {
            font_size,
            width,
            font_family,
            length_factor,
        } = config;
        if *length_factor <= 0.0 {
            return Err(
                diagnostic!("Length factor must be positive, but got: {length_factor}").into(),
            );
        }
        let font_family: String = font_family.pipe_as_ref(json_escape::escape_str).collect();
        let length_factor_percent = length_factor * 100.0;
        let data: String = data.pipe(json_escape::escape_str).collect();
        writeln!(writer, "{}", assets::typst::output()).into_diagnostic()?;
        writeln!(
            writer,
            "#set text({font_size}pt)
#article-render(
  width: {width}pt,
  font: \"{font_family}\",
  length-factor: {length_factor_percent}%,
  json(\"{data}\"),
)
"
        )
        .into_diagnostic()?;
        writer.flush().into_diagnostic()?;
        Ok(())
    }
}

// Utils

pub trait FileOrStdoutExt {
    fn clone_to_output_writer(&self) -> Result<impl Write>;
}

impl FileOrStdoutExt for FileOrStdout {
    fn clone_to_output_writer(&self) -> Result<impl Write> {
        debug!("Output to: {}", self.filename());
        self.clone().into_writer().into_diagnostic()?.pipe(Ok)
    }
}
