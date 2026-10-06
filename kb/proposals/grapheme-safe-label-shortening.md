# Label shortening at grapheme boundaries

Long leaf labels are shortened for drawings by keeping the start and the end and putting an ellipsis between them. The cut counts Unicode scalar values, so it can separate a combining accent from its letter or split an emoji sequence. Users see what Unicode calls a grapheme cluster as one character. This proposal moves the cut to grapheme boundaries.

## Problem

- **Code**: `pub fn shorten()` [packages/treeknit-io/src/display/names.rs#L10-L31](../../packages/treeknit-io/src/display/names.rs#L10-L31) splits the label into `char`s and keeps the first half and the last half of the allowed count. Its doc comment states the limit: "The standard library has no grapheme clusters, so a cut can separate a combining mark from its letter or split an emoji sequence"
- **Users of the result**: `short_name` of the tree drawings [packages/treeknit-io/src/display/tree.rs#L33](../../packages/treeknit-io/src/display/tree.rs#L33) and `short_label` of the ARG drawing [packages/treeknit-io/src/display/arg_view.rs#L54](../../packages/treeknit-io/src/display/arg_view.rs#L54), which the SVG figures and the web app draw as they are; the web app does not shorten labels itself
- **Example**: in a label written with a decomposed accent, `e` followed by U+0301, a cut between the two characters leaves a lone `e` on one side and a combining accent after the ellipsis on the other, which renders on top of the ellipsis. A flag emoji consists of two regional-indicator characters, and a cut between them shows two letters in boxes
- **Impact**: influenza strain names are mostly ASCII, where the two counts agree. The defect shows for names with accents in decomposed form, scripts with combining marks, and emoji

## Background

Unicode defines extended grapheme clusters, the units that users see as characters, in Unicode Standard Annex #29, "Unicode Text Segmentation" [[doc](https://www.unicode.org/reports/tr29/#Grapheme_Cluster_Boundaries)]. The crate `unicode-segmentation` 1.13.3 (2026-06-01, MIT OR Apache-2.0) implements these boundaries for Rust (`UnicodeSegmentation::graphemes(true)` for extended clusters [[doc](https://docs.rs/unicode-segmentation/1.13.3/unicode_segmentation/trait.UnicodeSegmentation.html#tymethod.graphemes)]); it is not yet a dependency of the project.

## Design axes

- **Cut at grapheme boundaries with `unicode-segmentation` (recommended)**: count and cut graphemes instead of `char`s; the limit `label_max_chars` then counts what users see. Adding the crate follows "Adding a dependency" of the project rules
- **Keep the current cut and the documented limit**: no dependency; labels with combining marks or emoji can render broken
- **Measure width instead of count**: a grapheme can be wide (CJK, emoji) or narrow; the SVG figures already estimate widths from font metrics, so the count could become a width. Larger change, separate from the boundary question

## Work items

- [`N-label-shortening-splits-graphemes.md`](../issues/N-label-shortening-splits-graphemes.md)
