//! Set `TREEKNIT_LONG_VERSION`, the version that the web app reports, with the rule of
//! `packages/version_rule.rs`, so a build with the same `TREEKNIT_VERSION` gives the command line
//! and the web app the same version.

include!("../version_rule.rs");

fn main() -> Result<(), Box<dyn std::error::Error>> {
  println!("cargo:rerun-if-changed=build.rs");
  println!("cargo:rerun-if-changed=../version_rule.rs");
  println!("cargo:rustc-env=TREEKNIT_LONG_VERSION={}", treeknit_version()?);
  Ok(())
}
