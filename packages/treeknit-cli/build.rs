//! Derive the version that `treeknit --version` reports: the crate version with
//! the suffix of `TREEKNIT_VERSION_SUFFIX`, `dev` when unset.

use std::env::{self, VarError};

const VERSION_SUFFIX_ENV: &str = "TREEKNIT_VERSION_SUFFIX";

fn main() -> Result<(), VarError> {
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-env-changed={VERSION_SUFFIX_ENV}");

  let suffix = match env::var(VERSION_SUFFIX_ENV) {
    Ok(suffix) if !suffix.is_empty() => suffix,
    Ok(_) | Err(VarError::NotPresent) => "dev".to_owned(),
    Err(error) => return Err(error),
  };
  let version = env!("CARGO_PKG_VERSION");
  println!("cargo:rustc-env=TREEKNIT_LONG_VERSION={version}-{suffix}");
  Ok(())
}
