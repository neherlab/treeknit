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
pub use pipeline::{PairResult, arg_inputs, imputed_trees, run, run_observed, unmatched_mccs};
pub use progress::{Phase, Progress};
pub use tree::{NodeId, Taxa, Tree};
