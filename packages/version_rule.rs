// The version rule of TreeKnit, included by the build scripts of `treeknit-cli` and
// `treeknit-wasm` (`include!`), so the command line and the web app report the same version.

/// Environment variable that sets the version at build time.
const VERSION_ENV: &str = "TREEKNIT_VERSION";

/// The version that TreeKnit reports: `TREEKNIT_VERSION` when set (the crate version, or the
/// crate version with a `-` suffix), otherwise the crate version with the suffix `-dev`.
fn treeknit_version() -> Result<String, Box<dyn std::error::Error>> {
  println!("cargo:rerun-if-env-changed={VERSION_ENV}");
  let crate_version = env!("CARGO_PKG_VERSION");
  let version = match std::env::var(VERSION_ENV) {
    Ok(version) if !version.is_empty() => version,
    Ok(_) | Err(std::env::VarError::NotPresent) => format!("{crate_version}-dev"),
    Err(error) => return Err(error.into()),
  };
  let matches_crate = version
    .strip_prefix(crate_version)
    .is_some_and(|rest| rest.is_empty() || rest.starts_with('-'));
  if !matches_crate {
    return Err(format!("{VERSION_ENV}={version} does not start with the crate version {crate_version}").into());
  }
  Ok(version)
}
