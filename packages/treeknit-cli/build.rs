//! Derive the version that `treeknit --version` reports: `TREEKNIT_VERSION` when
//! set (the crate version, or the crate version with a `-` suffix), otherwise
//! the crate version with the suffix `-dev`.

use std::env::{self, VarError};
use std::error::Error;

const VERSION_ENV: &str = "TREEKNIT_VERSION";

fn main() -> Result<(), Box<dyn Error>> {
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-env-changed={VERSION_ENV}");

  let crate_version = env!("CARGO_PKG_VERSION");
  let version = match env::var(VERSION_ENV) {
    Ok(version) if !version.is_empty() => version,
    Ok(_) | Err(VarError::NotPresent) => format!("{crate_version}-dev"),
    Err(error) => return Err(error.into()),
  };
  let matches_crate = version
    .strip_prefix(crate_version)
    .is_some_and(|rest| rest.is_empty() || rest.starts_with('-'));
  if !matches_crate {
    return Err(format!("{VERSION_ENV}={version} does not start with the crate version {crate_version}").into());
  }

  println!("cargo:rustc-env=TREEKNIT_LONG_VERSION={version}");
  Ok(())
}
