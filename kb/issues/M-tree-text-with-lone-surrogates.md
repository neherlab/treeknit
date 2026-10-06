# Tree text with a lone UTF-16 surrogate parses in the tree list but fails validation

JavaScript strings are UTF-16 and may hold a lone surrogate, a code unit from `U+D800` to `U+DFFF` without its partner. Text pasted into the tree dialog (`PasteTreeDialog`) can hold one; a loaded file cannot, because the app decodes files as UTF-8. The two ways a tree text enters Rust treat such a string differently:

- `inspectTree` takes the text as `&str`. wasm-bindgen encodes it with `TextEncoder`, which replaces a lone surrogate with `U+FFFD`, so the tree parses and the tree list shows it as valid
- `overlap`, `validate`, and `Session.run` take JSON arguments. `serde_json` 1.0.151 rejects the escape of a lone surrogate (`fn parse_unicode_escape` in `read.rs`): a lone leading surrogate (`\ud800`) gives "unexpected end of hex escape", and a lone trailing surrogate (`\udc00`) gives "lone leading surrogate in hex escape"

The app reports the second case as invalid input, such as `invalid request: trees[0].newick: unexpected end of hex escape at line 1 column 37`. The message names a JSON escape that the user never wrote, while the tree list shows the same tree as parsed.

> [!IMPORTANT]
> **Decision required.** Where should a lone surrogate be handled?
>
> - Replace lone surrogates with `U+FFFD` in `from_js` (`packages/treeknit-wasm/src/lib.rs`) before `serde_json` parses the text, as `inspectTree` does, so every export reads the same tree
> - Replace them in the pasted text in `PasteTreeDialog`, so a tree text in the workspace never holds one; the exports keep rejecting them
> - Reject the text in both paths with a message that names the character position in the tree text

## Validation

- A tree text with `\ud800` and one with `\udc00` give the same result from `inspectTree` and from `validate`, in a test of `packages/treeknit-wasm/tests/wasm.rs`
