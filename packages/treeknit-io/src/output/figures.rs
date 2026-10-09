//! The figures of a run and the downloads of the Auspice view, with the names of their files.

use crate::analysis::{self, ValidationError};
use crate::display::{self, AuspiceTrees, Scale, TreeVersion};
use crate::figure::{self, FigureOptions};
use crate::output::kinds::tanglegram_path;
use crate::output::{ARG_FIGURE, Figure, OutputFile};
use crate::run::RunResult;
use crate::wire::wire_name;
use serde::Serialize;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// The SVG text of `figure` of `run` with the default `FigureOptions`, a tanglegram of the
/// resolved trees, as `pair_figure` and `arg_figure` draw it; `None` when `run` lacks the pair or
/// the ARG.
pub fn figure_text(run: &RunResult, figure: Figure) -> Option<String> {
  let options = FigureOptions::default();
  let svg = match figure {
    Figure::Pair { pair } => pair_figure(run, pair, TreeVersion::Resolved, &options),
    Figure::Arg => arg_figure(run, &options),
  };
  #[expect(clippy::expect_used, reason = "the default figure options are valid")]
  svg.expect("default figure options").map(|f| f.text)
}

/// The SVG tanglegram of pair `pair` (pipeline order) of `run` in `version` with `options`;
/// `Ok(None)` when `run` lacks the pair, and the errors of `figure::check_figure_options` when
/// `options` are invalid. The scale is that of the pair view (`PairView.scale`), so the figure
/// shows what the interactive view shows.
pub fn pair_figure(
  run: &RunResult,
  pair: usize,
  version: TreeVersion,
  options: &FigureOptions,
) -> Result<Option<FigureDownload>, Vec<ValidationError>> {
  figure::checked(options)?;
  let Some(view) = display::pair_view(run, pair, version, options.scale) else {
    return Ok(None);
  };
  let p = &run.pairs[pair];
  let path = tanglegram_path(&run.trees[p.i].label, &run.trees[p.j].label);
  Ok(Some(FigureDownload {
    file_name: download_name(&path, Some(version), options, view.scale),
    text: figure::tanglegram_svg(&view, options)?,
  }))
}

/// The SVG figure of the ARG of `run` with `options`; `Ok(None)` for more than two trees or a
/// failed ARG, and the errors of `figure::check_figure_options` when `options` are invalid. The
/// scale is that of the ARG view (`ArgView.scale`).
pub fn arg_figure(run: &RunResult, options: &FigureOptions) -> Result<Option<FigureDownload>, Vec<ValidationError>> {
  figure::checked(options)?;
  let Some(view) = display::arg_view(run, options.scale) else {
    return Ok(None);
  };
  Ok(Some(FigureDownload {
    file_name: download_name(ARG_FIGURE, None, options, view.scale),
    text: figure::arg_svg(&view, options)?,
  }))
}

/// The downloads of the Auspice view of pair `pair` (pipeline order) of `run` in `version` with
/// `scale` and the shown `trees`; `None` when `run` lacks the pair. The datasets are those of
/// `display::auspice_view`, and the names tell the shown scale of that view.
pub fn auspice_files(
  run: &RunResult,
  pair: usize,
  version: TreeVersion,
  scale: Scale,
  trees: AuspiceTrees,
) -> Option<AuspiceFiles> {
  let view = display::auspice_view(run, pair, version, scale)?;
  let p = &run.pairs[pair];
  let [a, b] = [&run.trees[p.i].label, &run.trees[p.j].label];
  let pair_stem = analysis::pair_stem(a, b);
  let stem = if version == TreeVersion::Resolved && view.scale == Scale::Div {
    format!("auspice_{pair_stem}")
  } else {
    format!("auspice_{pair_stem}_{}_{}", wire_name(&version), wire_name(&view.scale))
  };
  let json = [(true, a, &view.left), (false, b, &view.right)]
    .into_iter()
    .filter(|&(left, _, _)| trees.shows(left))
    .map(|(_, label, dataset)| {
      #[expect(clippy::expect_used, reason = "an Auspice dataset serializes to JSON")]
      let text = serde_json::to_string_pretty(dataset).expect("a dataset serializes to JSON");
      OutputFile {
        path: format!("{stem}_{label}.json"),
        media_type: "application/json".to_owned(),
        text,
      }
    })
    .collect();
  let svg_prefix = match trees {
    AuspiceTrees::Both => stem,
    AuspiceTrees::Left => format!("{stem}_{a}"),
    AuspiceTrees::Right => format!("{stem}_{b}"),
  };
  Some(AuspiceFiles { svg_prefix, json })
}

/// The download name of a figure whose listed file is at `path`, drawn in `version` (`None` for
/// the ARG) with `options` and the shown `scale`; see `FigureDownload.file_name`.
#[expect(
  clippy::float_cmp,
  reason = "any other width or row height gives another figure, so the name must tell it"
)]
fn download_name(path: &str, version: Option<TreeVersion>, options: &FigureOptions, scale: Scale) -> String {
  let listed = FigureOptions::default();
  let name = path.rsplit_once('/').map_or(path, |(_, name)| name);
  if version.is_none_or(|v| v == TreeVersion::Resolved) && *options == listed {
    return name.to_owned();
  }
  let stem = name.strip_suffix(".svg").unwrap_or(name);
  let mut parts: Vec<String> = vec![stem.to_owned()];
  parts.extend(version.as_ref().map(wire_name));
  parts.push(wire_name(&scale));
  if options.width != listed.width {
    parts.push(format!("w{}", options.width));
  }
  if options.row_height != listed.row_height {
    parts.push(format!("row{}", options.row_height));
  }
  if options.labels != listed.labels {
    parts.push(format!("labels-{}", wire_name(&options.labels)));
  }
  format!("{}.svg", parts.join("_"))
}

/// An SVG figure with the name of its file as a download.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct FigureDownload {
  /// The name of the listed figure file (`tanglegram_ha_na.svg`, `arg.svg`) when the figure has
  /// its version and options; otherwise its stem with the version, the shown scale, and every
  /// option that differs from the default, such as `tanglegram_ha_na_imputed_depth_w800.svg`, so
  /// that a figure of other options never takes the name of the listed one.
  file_name: String,
  /// The SVG text.
  text: String,
}

/// The downloads of the Auspice view: the name of its SVG figure and the datasets of the shown
/// trees.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct AuspiceFiles {
  /// File name of the SVG figure without `.svg`, which Auspice's figure download appends:
  /// `auspice_<a>_<b>` for both trees, with the tree label appended for one tree, and with the
  /// version and the shown scale appended unless they are `resolved` and `div`.
  svg_prefix: String,
  /// One Auspice v2 dataset per shown tree, as pretty-printed JSON named `<stem>_<label>.json`.
  /// The names differ from the listed `auspice_<label>.json` files of the command line, which
  /// hold other fields.
  json: Vec<OutputFile>,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::figure::LabelMode;
  use crate::test_support::{HA, HA_LENGTHS, NA, NA_LENGTHS, run_trees};
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use serde_json::Value;

  #[rustfmt::skip]
  #[rstest]
  #[case::none(         HA,                                      NA,                                      Scale::Depth, Scale::Depth)]
  #[case::both(         "((A:1,B:1):1,(C:1,(D:1,X:1):1):1);",    "((A:1,(B:1,X:1):1):1,(C:1,D:1):1);",    Scale::Div,   Scale::Div)]
  #[case::one_tree(     "((A:1,B:1):1,(C:1,(D:1,X:1):1):1);",    NA,                                      Scale::Depth, Scale::Div)]
  #[case::some_branches("((A:1,B),(C,(D,X)));",                 "((A:1,(B:1,X:1):1):1,(C:1,D:1):1);",    Scale::Div,   Scale::Div)]
  #[trace]
  fn figures_with_the_div_scale_show_the_scale_of_the_views(
    #[case] ha: &str,
    #[case] na: &str,
    #[case] pair_scale: Scale,
    #[case] arg_scale: Scale,
  ) {
    // A tree without branch lengths draws the pair as cladograms; the ARG takes a missing length
    // from the other tree, so it needs lengths in neither tree to become a cladogram.
    let r = run_trees(&[("ha", ha), ("na", na)]);
    let view = display::pair_view(&r, 0, TreeVersion::Resolved, Scale::Div).unwrap();
    let arg = display::arg_view(&r, Scale::Div).unwrap();
    assert_eq!((pair_scale, arg_scale), (view.scale, arg.scale));
    let options = FigureOptions::default();
    let expected = (
      figure::tanglegram_svg(&view, &options).unwrap(),
      figure::arg_svg(&arg, &options).unwrap(),
    );
    assert_eq!(
      (Some(expected.0), Some(expected.1)),
      (figure_text(&r, Figure::Pair { pair: 0 }), figure_text(&r, Figure::Arg))
    );
  }

  #[test]
  fn pair_figure_with_the_depth_scale_keeps_it() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    let depth = FigureOptions {
      scale: Scale::Depth,
      width: 500.0,
      ..FigureOptions::default()
    };
    let view = display::pair_view(&r, 0, TreeVersion::Input, Scale::Depth).unwrap();
    let expected = FigureDownload {
      file_name: "tanglegram_ha_na_input_depth_w500.svg".to_owned(),
      text: figure::tanglegram_svg(&view, &depth).unwrap(),
    };
    assert_eq!(Some(expected), pair_figure(&r, 0, TreeVersion::Input, &depth).unwrap());
    assert_eq!(None, pair_figure(&r, 1, TreeVersion::Input, &depth).unwrap());
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::resolved_div_both(                                ("ha", "na", true),                        TreeVersion::Resolved, Scale::Div,   AuspiceTrees::Both,  ("auspice_ha_na",                                             vec!["auspice_ha_na_ha.json", "auspice_ha_na_na.json"]))]
  #[case::imputed_depth_both(                               ("ha", "na", true),                        TreeVersion::Imputed,  Scale::Depth, AuspiceTrees::Both,  ("auspice_ha_na_imputed_depth",                               vec!["auspice_ha_na_imputed_depth_ha.json", "auspice_ha_na_imputed_depth_na.json"]))]
  #[case::input_div_left(                                   ("ha", "na", true),                        TreeVersion::Input,    Scale::Div,   AuspiceTrees::Left,  ("auspice_ha_na_input_div_ha",                                vec!["auspice_ha_na_input_div_ha.json"]))]
  #[case::resolved_div_right(                               ("ha", "na", true),                        TreeVersion::Resolved, Scale::Div,   AuspiceTrees::Right, ("auspice_ha_na_na",                                          vec!["auspice_ha_na_na.json"]))]
  #[case::div_shown_as_depth_without_lengths_is_in_the_name(("ha", "na", false), TreeVersion::Resolved, Scale::Div, AuspiceTrees::Both, ("auspice_ha_na_resolved_depth", vec!["auspice_ha_na_resolved_depth_ha.json", "auspice_ha_na_resolved_depth_na.json"]))]
  #[case::long_labels(                                      ("segment_4_hemagglutinin", "segment_6_neuraminidase", true), TreeVersion::Resolved, Scale::Div, AuspiceTrees::Both, ("auspice_segment_4_hemagglutinin_segment_6_neuraminidase", vec!["auspice_segment_4_hemagglutinin_segment_6_neuraminidase_segment_4_hemagglutinin.json", "auspice_segment_4_hemagglutinin_segment_6_neuraminidase_segment_6_neuraminidase.json"]))]
  #[trace]
  fn auspice_files_names_tell_the_pair_version_scale_and_shown_trees(
    #[case] (a, b, lengths): (&str, &str, bool),
    #[case] version: TreeVersion,
    #[case] scale: Scale,
    #[case] trees: AuspiceTrees,
    #[case] (svg_prefix, json): (&str, Vec<&str>),
  ) {
    let (ha, na) = if lengths { (HA_LENGTHS, NA_LENGTHS) } else { (HA, NA) };
    let r = run_trees(&[(a, ha), (b, na)]);
    let files = auspice_files(&r, 0, version, scale, trees).unwrap();
    let paths: Vec<&str> = files.json.iter().map(|f| f.path.as_str()).collect();
    assert_eq!((svg_prefix, json), (files.svg_prefix.as_str(), paths));
  }

  #[test]
  fn auspice_files_hold_the_datasets_of_the_auspice_view_as_json() {
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    let view = display::auspice_view(&r, 0, TreeVersion::Resolved, Scale::Div).unwrap();
    let files = auspice_files(&r, 0, TreeVersion::Resolved, Scale::Div, AuspiceTrees::Both).unwrap();
    let parsed: Vec<Value> = files
      .json
      .iter()
      .map(|f| serde_json::from_str(&f.text).unwrap())
      .collect();
    let expected = vec![
      serde_json::to_value(&view.left).unwrap(),
      serde_json::to_value(&view.right).unwrap(),
    ];
    assert_eq!(expected, parsed);
    assert!(files.json.iter().all(|f| f.media_type == "application/json"));
    assert_eq!(
      None,
      auspice_files(&r, 1, TreeVersion::Resolved, Scale::Div, AuspiceTrees::Both)
    );
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::listed( TreeVersion::Resolved, 1200.0, 12.0, Scale::Div,   LabelMode::Auto, "tanglegram_ha_na.svg",                                  "arg.svg")]
  #[case::version(TreeVersion::Imputed,  1200.0, 12.0, Scale::Div,   LabelMode::Auto, "tanglegram_ha_na_imputed_depth.svg",                    "arg.svg")]
  #[case::scale(  TreeVersion::Resolved, 1200.0, 12.0, Scale::Depth, LabelMode::Auto, "tanglegram_ha_na_resolved_depth.svg",                   "arg_depth.svg")]
  #[case::options(TreeVersion::Input,    640.5,  20.0, Scale::Div,   LabelMode::Off,  "tanglegram_ha_na_input_depth_w640.5_row20_labels-off.svg", "arg_depth_w640.5_row20_labels-off.svg")]
  #[trace]
  fn figure_downloads_name_the_version_and_the_options_that_differ_from_the_listed_figure(
    #[case] version: TreeVersion,
    #[case] width: f64,
    #[case] row_height: f64,
    #[case] scale: Scale,
    #[case] labels: LabelMode,
    #[case] pair_name: &str,
    #[case] arg_name: &str,
  ) {
    // The example trees have no branch lengths, so every figure shows the scale `depth`.
    let r = run_trees(&[("ha", HA), ("na", NA)]);
    let options = FigureOptions { width, row_height, scale, labels };
    let pair = pair_figure(&r, 0, version, &options).unwrap().unwrap();
    let arg = arg_figure(&r, &options).unwrap().unwrap();
    assert_eq!((pair_name, arg_name), (pair.file_name.as_str(), arg.file_name.as_str()));
  }

  #[test]
  fn figure_text_of_a_missing_pair_or_arg_is_none() {
    let t = "((A,B),(C,D));";
    let r = run_trees(&[("ha", t), ("na", t), ("pb2", t)]);
    assert_eq!(
      (None, None),
      (figure_text(&r, Figure::Pair { pair: 3 }), figure_text(&r, Figure::Arg))
    );
  }
}
