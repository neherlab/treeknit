//! Output files of a run, as the command line writes them and the web app lists them.

use serde::Serialize;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// A result file with its text, at its path in the results directory of the command line.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct OutputFile {
  /// Path relative to the results directory, with `/` separators, such as `ARG/arg.nwk`.
  pub path: String,
  /// Media type of the text, such as `application/json`.
  pub media_type: String,
  /// The bytes the command line writes, newline rule included.
  pub text: String,
}

/// An entry of the file list of a run, without its text.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct FileEntry {
  /// Path relative to the results directory, as in `OutputFile`.
  pub path: String,
  pub media_type: String,
  /// Size in bytes; `None` for a figure not rendered yet.
  pub size: Option<usize>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use serde_json::json;

  #[test]
  fn file_entry_serializes_camel_case_with_null_size() {
    let entry = FileEntry {
      path: "tanglegram_ha_na.svg".into(),
      media_type: "image/svg+xml".into(),
      size: None,
    };
    let expected = json!({"path": "tanglegram_ha_na.svg", "mediaType": "image/svg+xml", "size": null});
    assert_eq!(expected, serde_json::to_value(&entry).unwrap());
  }
}
