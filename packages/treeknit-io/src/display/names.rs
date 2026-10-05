//! Node names as the drawings show them: unique within a drawing, and shortened to the label
//! length of the drawing rules.

use std::collections::BTreeSet;

/// Replaces the middle of a shortened label (U+2026).
const ELLIPSIS: char = '\u{2026}';

/// `name` shortened in the middle to at most `max` characters (Unicode scalar values), with an
/// ellipsis in place of the removed characters: the first half of the kept characters (rounded
/// up), the ellipsis, then the rest from the end. Empty for `max` 0, where not even the ellipsis
/// fits. The standard library has no grapheme clusters, so a cut can separate a combining mark
/// from its letter or split an emoji sequence; the interactive views show these shortened
/// labels, so both surfaces cut alike.
pub fn shorten(name: &str, max: usize) -> String {
  let chars: Vec<char> = name.chars().collect();
  if chars.len() <= max {
    return name.to_owned();
  }
  if max == 0 {
    return String::new();
  }
  let kept = max.saturating_sub(1);
  let head = kept.div_ceil(2);
  let tail = kept - head;
  let mut text: String = chars[..head].iter().collect();
  text.push(ELLIPSIS);
  text.extend(&chars[chars.len() - tail..]);
  text
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
  labels
    .into_iter()
    .zip(leaf)
    .map(|(label, &is_leaf)| {
      if is_leaf || (!label.is_empty() && taken.insert(label.clone())) {
        return label;
      }
      let base = if label.is_empty() { "NODE" } else { label.as_str() };
      // At most `all.len() + taken.len()` candidates are in use, so one of these is free.
      #[expect(clippy::expect_used, reason = "the range has more candidates than labels are in use")]
      let free = (2..=all.len() + taken.len() + 2)
        .map(|k| format!("{base}_{k}"))
        .find(|c| !all.contains(c) && !taken.contains(c))
        .expect("a free label");
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
  // Oracle: the combining acute accent U+0301 is a character of its own, so the cut after two
  // characters drops it from its letter.
  #[case::combining( "ae\u{301}xyz",          4,  "ae\u{2026}z")]
  #[case::empty(     "",                    0,  "")]
  #[trace]
  fn shorten_keeps_the_start_and_the_end(#[case] name: &str, #[case] max: usize, #[case] expected: &str) {
    assert_eq!(expected, shorten(name, max));
    assert!(shorten(name, max).chars().count() <= max);
  }

  #[test]
  fn shorten_of_a_long_label_has_the_rule_length() {
    let name = "x".repeat(50);
    assert_eq!(40, shorten(&name, 40).chars().count());
  }

  #[test]
  fn unique_labels_rename_taken_and_empty_internal_labels() {
    let labels = ["x", "a", "x", "", "a_2", "a"].map(String::from).to_vec();
    let leaf = [false, true, false, false, false, false];
    let expected = ["x", "a", "x_2", "NODE_2", "a_2", "a_3"].map(String::from).to_vec();
    assert_eq!(expected, unique_labels(labels, &leaf));
  }
}
