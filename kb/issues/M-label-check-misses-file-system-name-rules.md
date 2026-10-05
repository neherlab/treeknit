# Label check misses file-system name rules beyond case

A tree label is the file-name stem of the tree's output files, so `fn check_labels` in `packages/treeknit-io/src/analysis.rs` rejects labels that cannot name a file, or that name the same file as another label, on a release target. Labels name the same file when their `fn label_key` is equal: each character lowercased on its own. The command line (`fn path_labels` in `packages/treeknit-cli/src/main.rs`) and the web app (`fn tree_labels`) use the same key. File systems apply rules that this key and the check do not cover:

- **Unicode normalization**: APFS on macOS treats the composed `é` (U+00E9) and the decomposed `e` + U+0301 as one name. The two labels pass the check, and the output files of the second tree replace those of the first
- **Case folding beyond lowercase**: NTFS compares names with its upper-case table, and Unicode case folding maps some characters that lowercasing keeps apart, such as the final sigma `ς` and `σ`, or `ſ` and `s`
- **Windows device names**: `CON`, `PRN`, `AUX`, `NUL`, `COM1` to `COM9`, and `LPT1` to `LPT9`, also with an extension, cannot be file names on Windows
- **Trailing dots and spaces**: Windows removes them from file names, so `ha.` and `ha` name one file

The standard library covers part of these rules:

- **Per-character case mapping**: NTFS maps each character through its upper-case table on its own. Mapping each character with `char::to_uppercase` and then `char::to_lowercase` puts `ς` and `σ`, and `ſ` and `s`, under one key
- **Windows device names, trailing dots and spaces**: plain string checks

> [!IMPORTANT]
> **Decision required.** Unicode normalization (NFC) and full case folding, where one character folds to several (`ß` to `ss`), need a library, such as `unicode-normalization` and `caseless`, and the project rules require the user's approval for a new library. The options are to add these libraries and compare labels by NFC plus full case folding, or to keep the key of the standard library and document the limit for decomposed accents and multi-character folds.

## Fix direction

- Compute `label_key` per character as the lowercase of the uppercase, with the standard library
- Reject Windows device names and labels that end with a dot or a space in `check_labels`, on every platform, as the reserved characters are
- With the approved libraries only: compute `label_key` as the case folding of the NFC form of the label

## Validation

- Unit tests of `check_labels`: `ς` and `σ`, `CON`, `con.x`, and `ha.` next to `ha` each give an error; with the libraries, `é` in both forms and `ß` next to `ss` too
