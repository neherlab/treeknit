//! The labels of the trees: the check that they name output files, and the labels that the web
//! app gives trees loaded from files.

use crate::analysis::{Field, TreeText, ValidationError};
use std::collections::BTreeMap;
use std::collections::btree_map::Entry;

/// Labels of the web app for trees loaded from `file_names`: each file name without its last
/// extension (`ha.nwk` gives `ha`, `ha.tree.nwk` gives `ha.tree`), with every character that the
/// label check rejects (path separators, characters reserved on Windows, control characters)
/// replaced by `_`, or `tree` for a name that leaves no usable label (empty, blank, `.`, or
/// `..`). A label that equals one of `existing_labels` or an earlier new label, ignoring case as
/// the label check does, gets the first free suffix of `_2`, `_3`, ... Every label passes the
/// label check.
///
/// The command line labels its tree files by path instead (the file stem, and the parent
/// directory when stems collide), because it has directories to tell equal file names apart;
/// the web app has file names only.
pub fn tree_labels(file_names: &[String], existing_labels: &[String]) -> Vec<String> {
  let mut taken: std::collections::BTreeSet<String> = existing_labels.iter().map(|l| label_key(l)).collect();
  file_names
    .iter()
    .map(|name| {
      let stem = match name.rsplit_once('.') {
        Some((stem, _)) if !stem.is_empty() => stem,
        _ => name.as_str(),
      };
      let stem: String = stem
        .chars()
        .map(|c| if label_char_rejected(c) { '_' } else { c })
        .collect();
      let base = if stem.trim().is_empty() || stem == "." || stem == ".." {
        "tree"
      } else {
        stem.as_str()
      };
      let mut label = base.to_owned();
      let mut suffix = 1;
      while taken.contains(&label_key(&label)) {
        suffix += 1;
        label = format!("{base}_{suffix}");
      }
      taken.insert(label_key(&label));
      label
    })
    .collect()
}

/// Characters that Windows does not allow in file names, besides the path separators.
const RESERVED_CHARS: [char; 7] = ['<', '>', ':', '"', '|', '?', '*'];

/// Whether the label check rejects a label holding `c`: a path separator, a character reserved
/// on Windows, or a control character.
fn label_char_rejected(c: char) -> bool {
  c == '/' || c == '\\' || RESERVED_CHARS.contains(&c) || c.is_control()
}

/// A label is also a file-name stem of the outputs, so it must name a file in the results
/// directory on every release target: no path separator, no character reserved on Windows, no
/// control character, and neither `.` nor `..`. Labels with the same [`label_key`], such as
/// labels that differ only in case, are repeated labels.
///
/// Return the error of each label, in the order of `trees`.
pub(crate) fn check_labels(trees: &[TreeText]) -> Vec<Option<ValidationError>> {
  let mut errors = Vec::new();
  let mut seen: BTreeMap<String, &str> = BTreeMap::new();
  for (i, t) in trees.iter().enumerate() {
    let field = Field::TreeLabel { index: i };
    let l = t.label.as_str();
    let message = if l.trim().is_empty() {
      Some(format!("tree {} needs a label", i + 1))
    } else if l.contains(['/', '\\']) {
      Some(format!("tree label {l:?} must not contain / or \\"))
    } else if l.contains(RESERVED_CHARS) {
      Some(format!("tree label {l:?} must not contain any of <>:\"|?*"))
    } else if l.chars().any(char::is_control) {
      Some(format!("tree label {l:?} must not contain control characters"))
    } else if l == "." || l == ".." {
      Some(format!("tree label {l:?} is not a file name"))
    } else {
      match seen.entry(label_key(l)) {
        Entry::Vacant(e) => {
          e.insert(l);
          None
        },
        Entry::Occupied(e) if *e.get() == l => Some(format!("tree label {l:?} is used twice")),
        Entry::Occupied(e) => Some(format!(
          "tree label {l:?} differs from {:?} only in case, so their output files get the same name",
          e.get()
        )),
      }
    };
    errors.push(message.map(|m| ValidationError::at(field, m)));
  }
  errors
}

/// Pairs whose output files would get the same name, by the `label_key` of `check_labels`:
/// `MCCs_<a>_<b>.dat` joins two labels with `_`, so the labels `a_b`, `c`, `a`, `b_c` give the
/// pairs (0,1) and (2,3) one name. Only pairs of two `valid` labels are checked.
pub(crate) fn check_pair_stems(trees: &[TreeText], valid: &[bool]) -> Vec<ValidationError> {
  let mut first: BTreeMap<String, (usize, usize, String)> = BTreeMap::new();
  let mut errors = Vec::new();
  for i in 0..trees.len() {
    for j in i + 1..trees.len() {
      if !(valid[i] && valid[j]) {
        continue;
      }
      let stem = pair_stem(&trees[i].label, &trees[j].label);
      match first.entry(label_key(&stem)) {
        Entry::Occupied(e) => {
          let (a, b, first_stem) = e.get();
          let names = if *first_stem == stem {
            format!("the same output file names ({stem:?})")
          } else {
            format!("output file names ({first_stem:?} and {stem:?}) that differ only in case")
          };
          errors.push(ValidationError::at(
            Field::Trees,
            format!(
              "tree pairs ({:?}, {:?}) and ({:?}, {:?}) give {names}; rename a tree",
              trees[*a].label, trees[*b].label, trees[i].label, trees[j].label
            ),
          ));
        },
        Entry::Vacant(e) => {
          e.insert((i, j, stem));
        },
      }
    }
  }
  errors
}

/// Key under which labels name the same output files: labels with equal keys are repeated
/// labels, because the file systems of macOS and Windows ignore case. The key lowercases each
/// character on its own, so that a letter has one key wherever it stands in the label (`Σ` and
/// `σ` both give `σ`, while `str::to_lowercase` gives a final `ς`). Unicode normalization and
/// full case folding are not applied (`kb/issues/M-label-check-misses-file-system-name-rules.md`).
pub fn label_key(label: &str) -> String {
  label.chars().flat_map(char::to_lowercase).collect()
}

/// File-name stem of the pair of trees labeled `a` and `b`, as in `MCCs_<a>_<b>.dat`.
pub fn pair_stem(a: &str, b: &str) -> String {
  format!("{a}_{b}")
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::analysis::tests::{label, labeled, tree_errors};
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  #[rustfmt::skip]
  #[rstest]
  #[case::no_tree(        &[],                         Field::Trees,      "need at least two trees")]
  #[case::one_tree(       &["ha"],                     Field::Trees,      "need at least two trees")]
  #[case::empty_label(    &["ha", ""],                 label(1),          "tree 2 needs a label")]
  #[case::blank_label(    &["ha", " "],                label(1),          "tree 2 needs a label")]
  #[case::duplicate_label(&["ha", "na", "ha"],         label(2),          "tree label \"ha\" is used twice")]
  #[case::parent_path(    &["ha", "../x"],             label(1),          "tree label \"../x\" must not contain / or \\")]
  #[case::slash(          &["a/b", "na"],              label(0),          "tree label \"a/b\" must not contain / or \\")]
  #[case::backslash(      &["ha", "a\\b"],             label(1),          "tree label \"a\\\\b\" must not contain / or \\")]
  #[case::control(        &["ha", "a\tb"],             label(1),          "tree label \"a\\tb\" must not contain control characters")]
  #[case::dot(            &["ha", "."],                label(1),          "tree label \".\" is not a file name")]
  #[case::dot_dot(        &["..", "na"],               label(0),          "tree label \"..\" is not a file name")]
  #[case::pair_stems(     &["a_b", "c", "a", "b_c"],   Field::Trees,      "tree pairs (\"a_b\", \"c\") and (\"a\", \"b_c\") give the same output file names (\"a_b_c\"); rename a tree")]
  #[case::case_only(      &["HA", "na", "ha"],         label(2),          "tree label \"ha\" differs from \"HA\" only in case, so their output files get the same name")]
  #[case::final_sigma(    &["ΑΣ", "na", "ασ"],         label(2),          "tree label \"ασ\" differs from \"ΑΣ\" only in case, so their output files get the same name")]
  #[case::pair_stems_case(&["a_b", "c", "A", "B_c"],   Field::Trees,      "tree pairs (\"a_b\", \"c\") and (\"A\", \"B_c\") give output file names (\"a_b_c\" and \"A_B_c\") that differ only in case; rename a tree")]
  #[case::less_than(      &["ha", "a<b"],              label(1),          "tree label \"a<b\" must not contain any of <>:\"|?*")]
  #[case::greater_than(   &["ha", "a>b"],              label(1),          "tree label \"a>b\" must not contain any of <>:\"|?*")]
  #[case::colon(          &["ha", "a:b"],              label(1),          "tree label \"a:b\" must not contain any of <>:\"|?*")]
  #[case::quote(          &["ha", "a\"b"],             label(1),          "tree label \"a\\\"b\" must not contain any of <>:\"|?*")]
  #[case::pipe(           &["ha", "a|b"],              label(1),          "tree label \"a|b\" must not contain any of <>:\"|?*")]
  #[case::question(       &["ha", "a?b"],              label(1),          "tree label \"a?b\" must not contain any of <>:\"|?*")]
  #[case::star(           &["ha", "a*b"],              label(1),          "tree label \"a*b\" must not contain any of <>:\"|?*")]
  #[trace]
  fn invalid_labels_are_rejected(#[case] labels: &[&str], #[case] field: Field, #[case] message: &str) {
    assert_eq!(vec![ValidationError::at(field, message)], tree_errors(&labeled(labels)));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::dots_inside(  &["ha", "a.b", "..c"])]
  #[case::unicode(      &["A-H3N2 HA", "NA ü"])]
  #[case::underscores(  &["a_b", "c", "a"])]
  #[case::case_distinct(&["HA", "ha2"])]
  #[trace]
  fn usable_labels_are_accepted(#[case] labels: &[&str]) {
    assert_eq!(Vec::<ValidationError>::new(), tree_errors(&labeled(labels)));
  }

  #[test]
  fn invalid_label_does_not_hide_pair_name_collisions() {
    let expected = vec![
      ValidationError::at(label(4), "tree label \"bad/label\" must not contain / or \\"),
      ValidationError::at(
        Field::Trees,
        "tree pairs (\"a_b\", \"c\") and (\"a\", \"b_c\") give the same output file names (\"a_b_c\"); rename a tree",
      ),
    ];
    assert_eq!(expected, tree_errors(&labeled(&["a_b", "c", "a", "b_c", "bad/label"])));
  }

  #[test]
  fn repeated_label_gives_no_pair_name_error() {
    assert_eq!(
      vec![ValidationError::at(label(2), "tree label \"ha\" is used twice")],
      tree_errors(&labeled(&["ha", "na", "ha"]))
    );
  }

  fn strings(v: &[&str]) -> Vec<String> {
    v.iter().map(|&s| s.to_owned()).collect()
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::extension(     &["ha.nwk", "na.tree"],          &[],             &["ha", "na"])]
  #[case::last_extension(&["ha.tree.nwk"],                &[],             &["ha.tree"])]
  #[case::no_extension(  &["ha"],                         &[],             &["ha"])]
  #[case::dot_file(      &[".nwk"],                       &[],             &[".nwk"])]
  #[case::empty_name(    &["", ""],                       &[],             &["tree", "tree_2"])]
  #[case::existing(      &["ha.nwk"],                     &["ha"],         &["ha_2"])]
  #[case::existing_case( &["ha.nwk"],                     &["HA"],         &["ha_2"])]
  #[case::new_repeated(  &["ha.nwk", "ha.tree", "ha.nwk"], &[],            &["ha", "ha_2", "ha_3"])]
  #[case::suffix_taken(  &["ha.nwk"],                     &["ha", "ha_2"], &["ha_3"])]
  #[case::no_files(      &[],                             &["ha"],         &[])]
  #[case::reserved(      &["ha:1.nwk", "a<b>|c?*\"d.nwk"],  &[],            &["ha_1", "a_b__c___d"])]
  #[case::separators(    &["a\\b.nwk"],                   &[],             &["a_b"])]
  #[case::control(       &["a\tb.nwk"],                    &[],             &["a_b"])]
  #[case::blank_stem(    &[" .nwk"],                      &[],             &["tree"])]
  #[case::dot_stem(      &["..nwk"],                      &[],             &["tree"])]
  #[case::dot_dot_stem(  &["...nwk", ".."],               &[],             &["tree", "tree_2"])]
  #[trace]
  fn tree_labels_follow_the_web_label_policy(#[case] files: &[&str], #[case] existing: &[&str], #[case] expected: &[&str]) {
    assert_eq!(strings(expected), tree_labels(&strings(files), &strings(existing)));
  }

  #[test]
  fn tree_labels_pass_the_label_check() {
    // Repeated file names and existing labels give labels that parse_trees accepts.
    let existing = strings(&["HA", "na"]);
    let files = [
      "ha.nwk", "ha.nwk", "NA.nwk", "ha:1.nwk", " .nwk", "..nwk", "a\tb", "...nwk",
    ];
    let new = tree_labels(&strings(&files), &existing);
    let all: Vec<&str> = existing.iter().chain(&new).map(String::as_str).collect();
    assert_eq!(Vec::<ValidationError>::new(), tree_errors(&labeled(&all)));
  }

  #[test]
  fn pair_stem_joins_labels() {
    assert_eq!("ha_na", pair_stem("ha", "na"));
  }
}
