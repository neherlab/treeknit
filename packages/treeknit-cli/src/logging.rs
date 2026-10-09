//! The log of a run: the terminal, and `log.txt` in the results directory.

use crate::{Cli, stderr_has_colors};
use eyre::Result;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger};
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::sync::{Arc, OnceLock};

/// Configure the log: the terminal at the verbosity of `cli`, colored only when
/// `stderr_has_colors`, and `file` with debug records too.
pub(crate) fn setup_logging(cli: &Cli, file: &LogFile) -> Result<()> {
  let v = if cli.verbose && cli.verbosity_level == 0 {
    1
  } else {
    cli.verbosity_level
  };
  let term_level = match v {
    i32::MIN..=-1 => LevelFilter::Off,
    0 => LevelFilter::Info,
    1 => LevelFilter::Debug,
    _ => LevelFilter::Trace,
  };
  let config = ConfigBuilder::new()
    .set_time_format_rfc3339()
    .set_target_level(LevelFilter::Off)
    .build();
  let colors = if stderr_has_colors() {
    ColorChoice::Auto
  } else {
    ColorChoice::Never
  };
  CombinedLogger::init(vec![
    TermLogger::new(term_level, config.clone(), TerminalMode::Stderr, colors),
    WriteLogger::new(LevelFilter::Debug.max(term_level), config, file.writer()),
  ])?;
  Ok(())
}

/// The file `log.txt`, named once the input has passed validation.
#[derive(Default)]
pub(crate) struct LogFile(Arc<OnceLock<fs::File>>);

impl LogFile {
  /// Create the file at `path`. The records so far reach it before the next record.
  pub(crate) fn open(&self, path: &Path) -> io::Result<()> {
    let file = fs::File::create(path)?;
    self
      .0
      .set(file)
      .map_err(|_unused_file| io::Error::other("the log file is already open"))
  }

  /// The writer of the logger: it keeps the records in memory until the file is open.
  fn writer(&self) -> LogWriter {
    LogWriter {
      file: Arc::clone(&self.0),
      memory: Vec::new(),
    }
  }
}

/// Writer of `log.txt` for the logger; see `LogFile`.
struct LogWriter {
  file: Arc<OnceLock<fs::File>>,
  /// The records written before the file was open.
  memory: Vec<u8>,
}

impl Write for LogWriter {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match self.file.get() {
      Some(mut file) => {
        if !self.memory.is_empty() {
          file.write_all(&std::mem::take(&mut self.memory))?;
        }
        file.write(buf)
      },
      None => self.memory.write(buf),
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    self.file.get().map_or(Ok(()), |mut file| file.flush())
  }
}
