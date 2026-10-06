//! Capture of the `log` records of the Rust code: the diagnostics and `log.txt` of a run.

use js_sys::Date;
use log::{LevelFilter, Log, Metadata, Record};
use std::cell::{Cell, RefCell};
use std::sync::atomic::{AtomicBool, Ordering};
use treeknit_io::summary::{Diagnostic, Level};
use wasm_bindgen::prelude::*;

/// The logger of the module: it keeps records of level Debug and above in `RECORDS`.
struct Capture;

static CAPTURE: Capture = Capture;

/// Whether `install` installed `CAPTURE`. A flag of its own, because the address of a zero-sized
/// static such as `CAPTURE` need not differ from that of another logger.
static INSTALLED: AtomicBool = AtomicBool::new(false);

/// Why a run has no diagnostics and no `log.txt` lines when another logger is installed.
const UNAVAILABLE: &str =
  "another logger is installed, so runs record no diagnostics and no log.txt lines besides this one";

thread_local! {
  /// Records since the last `take`. Browsers run the module on one thread.
  static RECORDS: RefCell<Vec<Diagnostic>> = const { RefCell::new(Vec::new()) };
  /// Whether a run holds a [`RunGuard`]. A progress callback of the run may call other exports,
  /// whose records then belong to the run's log instead of clearing it.
  static RUNNING: Cell<bool> = const { Cell::new(false) };
}

#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console, js_name = error)]
  fn console_error(message: &str);
}

impl Log for Capture {
  fn enabled(&self, metadata: &Metadata<'_>) -> bool {
    metadata.level() <= log::Level::Debug
  }

  fn log(&self, record: &Record<'_>) {
    if !self.enabled(record.metadata()) {
      return;
    }
    RECORDS.with_borrow_mut(|r| r.push(diagnostic(record.level().into(), record.args().to_string())));
  }

  fn flush(&self) {}
}

/// Install the logger; a second call keeps the installed one. When another logger is installed,
/// write the failure to the console instead: the module still works, and each run reports the
/// failure as a diagnostic (see `unavailable`).
pub(crate) fn install() {
  if INSTALLED.load(Ordering::Relaxed) {
    return;
  }
  if log::set_logger(&CAPTURE).is_ok() {
    log::set_max_level(LevelFilter::Debug);
    INSTALLED.store(true, Ordering::Relaxed);
  } else {
    console_error(UNAVAILABLE);
  }
}

/// The warning that a run records when the logger is not installed; `None` when it is.
pub(crate) fn unavailable() -> Option<Diagnostic> {
  (!INSTALLED.load(Ordering::Relaxed)).then(|| diagnostic(Level::Warn, UNAVAILABLE.to_owned()))
}

/// The records since the last call, in the order they occurred; clears the buffer.
pub(crate) fn take() -> Vec<Diagnostic> {
  RECORDS.with_borrow_mut(std::mem::take)
}

/// Guard of a run: while it lives, [`discard`] keeps the buffer, so the records of exports that a
/// progress callback calls do not erase the records of the run.
#[must_use = "the run's records are unprotected once the guard is dropped"]
pub(crate) struct RunGuard(());

/// The guard of a new run; `None` while another run holds one, because a nested run would take
/// the outer run's records and release the guard of the outer run when it ends.
pub(crate) fn begin_run() -> Option<RunGuard> {
  (!RUNNING.replace(true)).then(|| RunGuard(()))
}

impl Drop for RunGuard {
  fn drop(&mut self) {
    RUNNING.set(false);
  }
}

/// Guard of an export that is not a run; see [`discard`].
#[must_use = "the records are discarded when the guard is dropped"]
pub(crate) struct Discard(());

/// Discard the records of an export: clear the buffer now and when the guard drops, so they never
/// mix into the log of a run. During a run (see [`begin_run`]), keep the buffer instead.
pub(crate) fn discard() -> Discard {
  clear_outside_run();
  Discard(())
}

impl Drop for Discard {
  fn drop(&mut self) {
    clear_outside_run();
  }
}

/// Clear the buffer unless a run holds a [`RunGuard`].
fn clear_outside_run() {
  if !RUNNING.get() {
    take();
  }
}

/// A record of `level` with `message`, timed now.
fn diagnostic(level: Level, message: String) -> Diagnostic {
  Diagnostic {
    level,
    message,
    // `toISOString` writes UTC with milliseconds, such as `2026-10-05T12:00:00.000Z`: RFC 3339.
    time: Date::new_0().to_iso_string().into(),
  }
}
