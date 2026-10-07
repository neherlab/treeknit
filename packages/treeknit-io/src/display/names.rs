//! Node names as the drawings show them: unique within a drawing, and shortened to the label
//! length of the drawing rules.

use std::collections::{BTreeMap, BTreeSet};
use unicode_segmentation::UnicodeSegmentation;

/// Replaces the middle of a shortened label (U+2026).
const ELLIPSIS: char = '\u{2026}';

/// `name` shortened in the middle to at most `max` characters, with an ellipsis in place of the
/// removed characters: the first half of the kept characters (rounded up), the ellipsis, then the
/// rest from the end. Empty for `max` 0, where not even the ellipsis fits. A character is an
/// extended grapheme cluster of Unicode Standard Annex #29, what a reader sees as one character,
/// so a cut keeps a combining mark with its letter and an emoji sequence whole. The figures,
/// `DrawNode.short_name`, and `ArgNodeView.short_label` use this rule.
pub fn shorten(name: &str, max: usize) -> String {
  let graphemes: Vec<&str> = name.graphemes(true).collect();
  if graphemes.len() <= max {
    return name.to_owned();
  }
  if max == 0 {
    return String::new();
  }
  let kept = max.saturating_sub(1);
  let head = kept.div_ceil(2);
  let tail = kept - head;
  let mut text = graphemes[..head].concat();
  text.push(ELLIPSIS);
  text.push_str(&graphemes[graphemes.len() - tail..].concat());
  text
}

/// The number of characters of `text` as [`shorten`] counts them: extended grapheme clusters.
pub(crate) fn grapheme_count(text: &str) -> usize {
  text.graphemes(true).count()
}

/// `labels` made unique and non-empty, because views select nodes by label. Leaves keep their
/// labels, which are distinct taxon names. An internal node whose label is empty or taken by a
/// leaf or an earlier node gets the first `<label>_<k>`, k ≥ 2 (`NODE_<k>` for an empty label),
/// that no node has: a leaf can be named like an internal node, such as an ARG node
/// (`ARGNode_3`), the synthetic ARG root, or an internal node of another tree that imputation
/// grafted a leaf of the same name next to.
pub(super) fn unique_labels(labels: Vec<String>, leaf: &[bool]) -> Vec<String> {
  let all: BTreeSet<String> = labels.iter().cloned().collect();
  let mut taken: BTreeSet<String> = labels
    .iter()
    .zip(leaf)
    .filter(|(_, l)| **l)
    .map(|(s, _)| s.clone())
    .collect();
  // The next k to try for each base, so that renaming n nodes of one base tries each k once.
  let mut next: BTreeMap<String, usize> = BTreeMap::new();
  labels
    .into_iter()
    .zip(leaf)
    .map(|(label, &is_leaf)| {
      if is_leaf || (!label.is_empty() && taken.insert(label.clone())) {
        return label;
      }
      let base = if label.is_empty() { "NODE".to_owned() } else { label };
      let k = next.entry(base.clone()).or_insert(2);
      let free = loop {
        let candidate = format!("{base}_{k}");
        *k += 1;
        if !all.contains(&candidate) && !taken.contains(&candidate) {
          break candidate;
        }
      };
      taken.insert(free.clone());
      free
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  #[rustfmt::skip]
  #[rstest]
  #[case::short(     "A/New York/392/2004", 40, "A/New York/392/2004")]
  #[case::at_max(    "abcdef",              6,  "abcdef")]
  // Oracle: 5 kept characters, 3 from the start and 2 from the end.
  #[case::odd_kept(  "abcdefgh",            6,  "abc\u{2026}gh")]
  #[case::even_kept( "abcdefgh",            5,  "ab\u{2026}gh")]
  #[case::multibyte( "αβγδεζηθ",            4,  "αβ\u{2026}θ")]
  #[case::one(       "abc",                 1,  "\u{2026}")]
  #[case::none(      "abc",                 0,  "")]
  // Oracle: e and the combining acute accent U+0301 form one grapheme cluster, which the cut
  // after two characters keeps whole.
  #[case::combining( "ae\u{301}xyz",          4,  "ae\u{301}\u{2026}z")]
  // Oracle: the flag of the United States is the regional indicators U and S, one cluster.
  #[case::flag(      "a\u{1F1FA}\u{1F1F8}cde", 4,  "a\u{1F1FA}\u{1F1F8}\u{2026}e")]
  #[case::empty(     "",                    0,  "")]
  #[trace]
  fn shorten_keeps_the_start_and_the_end(#[case] name: &str, #[case] max: usize, #[case] expected: &str) {
    assert_eq!(expected, shorten(name, max));
    assert!(grapheme_count(&shorten(name, max)) <= max);
  }

  #[test]
  fn shorten_of_a_long_label_has_the_rule_length() {
    let name = "x".repeat(50);
    assert_eq!(40, grapheme_count(&shorten(&name, 40)));
  }

  #[test]
  fn unique_labels_rename_taken_and_empty_internal_labels() {
    let labels = ["x", "a", "x", "", "a_2", "a"].map(String::from).to_vec();
    let leaf = [false, true, false, false, false, false];
    let expected = ["x", "a", "x_2", "NODE_2", "a_2", "a_3"].map(String::from).to_vec();
    assert_eq!(expected, unique_labels(labels, &leaf));
  }

  #[test]
  fn unique_labels_give_repeated_labels_the_next_free_numbers() {
    // Oracle: the leaf x_3 takes its number, so the five internal nodes named x get x, x_2, x_4,
    // x_5, and x_6.
    let labels = ["x", "x", "x_3", "x", "x", "x"].map(String::from).to_vec();
    let leaf = [false, false, true, false, false, false];
    let expected = ["x", "x_2", "x_3", "x_4", "x_5", "x_6"].map(String::from).to_vec();
    assert_eq!(expected, unique_labels(labels, &leaf));
  }
}
