//! Capture of the `log` records of the Rust code: the diagnostics and `log.txt` of a run.

use js_sys::Date;
use log::{LevelFilter, Log, Metadata, Record};
use std::cell::RefCell;
use treeknit_io::summary::Diagnostic;

/// The logger of the module: it keeps records of level Debug and above in `RECORDS`.
struct Capture;

static CAPTURE: Capture = Capture;

thread_local! {
  /// Records since the last `take`. Browsers run the module on one thread.
  static RECORDS: RefCell<Vec<Diagnostic>> = const { RefCell::new(Vec::new()) };
}

impl Log for Capture {
  fn enabled(&self, metadata: &Metadata<'_>) -> bool {
    metadata.level() <= log::Level::Debug
  }

  fn log(&self, record: &Record<'_>) {
    if !self.enabled(record.metadata()) {
      return;
    }
    let diagnostic = Diagnostic {
      level: record.level().into(),
      message: record.args().to_string(),
      // `toISOString` writes UTC with milliseconds, such as `2026-10-05T12:00:00.000Z`: RFC 3339.
      time: Date::new_0().to_iso_string().into(),
    };
    RECORDS.with_borrow_mut(|r| r.push(diagnostic));
  }

  fn flush(&self) {}
}

/// Install the logger; a second call keeps the installed one.
pub(crate) fn install() {
  if log::set_logger(&CAPTURE).is_ok() {
    log::set_max_level(LevelFilter::Debug);
  }
}

/// The records since the last call, in the order they occurred; clears the buffer.
pub(crate) fn take() -> Vec<Diagnostic> {
  RECORDS.with_borrow_mut(std::mem::take)
}

/// Guard of an export that is not a run; see [`discard`].
#[must_use = "the records are discarded when the guard is dropped"]
pub(crate) struct Discard(());

/// Discard the records of an export: clear the buffer now and when the guard drops, so they never
/// mix into the log of a run. Installs the logger too, because the start function that installs
/// it in the web app does not run under `wasm-bindgen-test`.
pub(crate) fn discard() -> Discard {
  install();
  take();
  Discard(())
}

impl Drop for Discard {
  fn drop(&mut self) {
    take();
  }
}
