# Label check misses file-system name rules beyond case

A tree label is the file-name stem of the tree's output files, so `fn check_labels` in `packages/treeknit-io/src/analysis.rs` rejects labels that cannot name a file, or that name the same file as another label, on a release target. Labels name the same file when their `fn label_key` is equal: each character lowercased on its own. The command line (`fn path_labels` in `packages/treeknit-cli/src/main.rs`) and the web app (`fn tree_labels`) use the same key. File systems apply rules that this key and the check do not cover:

- **Unicode normalization**: APFS on macOS treats the composed `é` (U+00E9) and the decomposed `e` + U+0301 as one name. The two labels pass the check, and the output files of the second tree replace those of the first
- **Case folding beyond lowercase**: NTFS compares names with its upper-case table, and Unicode case folding maps some characters that lowercasing keeps apart, such as the final sigma `ς` and `σ`, or `ſ` and `s`
- **Windows device names**: `CON`, `PRN`, `AUX`, `NUL`, `COM1` to `COM9`, and `LPT1` to `LPT9`, also with an extension, cannot be file names on Windows
- **Trailing dots and spaces**: Windows removes them from file names, so `ha.` and `ha` name one file

> [!IMPORTANT]
> **Decision required.** Normalization and full case folding need a library, such as `unicode-normalization` for NFC and `caseless` for Unicode default case folding, and the project rules require the user's approval for a new library. The options are to add these libraries and compare labels by NFC plus case folding, or to keep the per-character lowercase key and document the limit.

## Fix direction

- Compute `label_key` as the case folding of the NFC form of the label, with the approved libraries
- Reject Windows device names and labels that end with a dot or a space in `check_labels`, on every platform, as the reserved characters are

## Validation

- Unit tests of `check_labels`: `é` in both forms, `ς` and `σ`, `CON`, `con.x`, and `ha.` next to `ha` each give an error
