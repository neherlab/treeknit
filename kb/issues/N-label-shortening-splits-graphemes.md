# Label shortening splits grapheme clusters

`pub fn shorten()` [packages/treeknit-io/src/display/names.rs#L10-L31](../../packages/treeknit-io/src/display/names.rs#L10-L31) cuts long labels by Unicode scalar value. A cut can separate a combining accent from its letter or split an emoji sequence, so the shortened label renders broken in the SVG figures and in the web app, which draw `short_name` and `short_label` as they are. The doc comment of the function states the limit. The options are in [`grapheme-safe-label-shortening.md`](../proposals/grapheme-safe-label-shortening.md).

> [!IMPORTANT]
> **Decision required.** Cutting at grapheme boundaries needs the crate `unicode-segmentation`, a new dependency. The alternative is to keep the current cut and its documented limit.

## Fix direction

- Count and cut extended grapheme clusters with `unicode-segmentation`, keeping the rule "first half, ellipsis, last half"
- Update the doc comment and the test that counts `chars` in `packages/treeknit-io/src/display/tree.rs`

## Validation

- Unit tests of `shorten`: a label with `e` followed by U+0301 at the cut position keeps the pair together; a label with a flag emoji at the cut position keeps both regional indicators together; ASCII labels give the same result as before
