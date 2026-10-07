//! Core algorithms of TreeKnit: maximally compatible clades (MCCs) between trees of
//! different genome segments, and ancestral reassortment graphs (ARGs) for two segments.
//!
//! This crate performs no file IO; see `treeknit-io` for formats.

pub mod anneal;
pub mod arg;
pub mod bits;
pub mod impute;
pub mod mcc_map;
pub mod naive;
pub mod options;
pub mod pair;
pub mod pipeline;
pub mod progress;
pub mod resolve;
pub mod splitgraph;
pub mod tree;

pub use naive::{Mcc, naive_mccs};
pub use options::{Cooling, Method, Options, Resolution};
pub use pipeline::{
  PairResult, arg_inputs, imputed_trees, keeps_run_order, last_sorting_pair, run, run_observed, sort_for_pair,
  sort_strictness, unmatched_mccs,
};
pub use progress::{Phase, Progress};
pub use tree::{NodeId, Taxa, Tree};

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
