//! File formats for TreeKnit.

pub mod analysis;
pub mod arg;
pub mod auspice;
pub mod display;
pub mod examples;
pub mod figure;
pub mod inspect;
pub mod launch;
pub mod mccs;
pub mod newick;
pub mod output;
pub mod palette;
pub mod progress;
pub mod run;
pub mod schema;
pub mod summary;
#[cfg(test)]
mod test_support;
pub mod version;
pub mod wire;

#[cfg(test)]
mod tests {
  use ctor::ctor;

  /// One thread in the global pool: the test runner runs many tests at once, and a test that
  /// needs concurrency installs a local pool.
  #[ctor(unsafe)]
  fn init() {
    rayon::ThreadPoolBuilder::new()
      .num_threads(1)
      .build_global()
      .expect("the global thread pool of the tests is built once");
  }
}
