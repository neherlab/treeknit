# File formats: parity checklist

Counterpart: [`v0/formats.md`](v0/formats.md). `treeknit-io` reads and writes all formats. TreeKnit.jl uses TreeTools.jl for Newick.

## Input: Newick

`fn parse` and `fn parse_first` [[src](../../packages/treeknit-io/src/newick.rs#L18-L59)] read the classic dialect of `util-newick` ([README](../../packages/util-newick/README.md)): standard Newick, with comments as text:

- [/] **Several trees in one file**: the first tree is used, with the warning "<label>: more than one tree in file, using the first". A tree ends at the first `;` outside quoted labels and `[...]` comments, and any text after it other than comments counts as more trees, so `('a;b',C);[x;y]` is one tree and `(A,B);x` gives the warning. The validation logs the warning and the tree inspection of the web app reports it, also when the first tree has an unnamed or a duplicate leaf; a first tree that does not parse gives its parse error alone. TreeKnit.jl reads a vector, which its command line does not handle
- [x] **Terminator**: `;` is required. Whitespace, `\r`, and comments after it are accepted; TreeTools.jl rejects them. A byte order mark is a parse error
- [/] **Branch lengths**: numbers with an optional sign, a leading point (`.5`), and an exponent, read as `f64`. A length that is not a number (`0.R`) or too large for `f64` (`1e999`) is a parse error with its line and column. TreeTools.jl reads an invalid length as missing; this differs on purpose ([`docs/user/treeknit-jl.md`](../../docs/user/treeknit-jl.md#deliberate-differences-from-treeknitjl))
- [x] **Root**: the root branch length is dropped. A root polytomy stays a polytomy
- [x] **Unnamed internal nodes**: `NODE_<k>`, numbered in pre-order for each tree [[src](../../packages/treeknit-io/src/newick.rs#L187-L218)]
- [/] **Unnamed leaves**: an error "unnamed leaf". TreeTools.jl names them `NODE_<k>`
- [/] **Numeric internal labels**: support values such as `87`, `0.95`, or `80.5/95`, and quoted numbers such as `'95'`, become `NODE_<k>`, so the output has the same names in every run and loses the support values. TreeTools.jl renames them `<label>__<random>`
- [x] **Duplicate leaf names**: an error, as in TreeTools.jl
- [/] **Duplicate internal names**: renamed `NODE_<k>`. TreeTools.jl raises an error
- [x] **Quoted labels (new)**: `'a b'`, with `''` for a quote character. `#` is part of a label
- [x] **Comments (new)**: `[...]` is skipped, including annotations such as `[&x=1]`. Brackets inside a comment nest, so `[a[;]b]` is one comment
- [x] **Whitespace (new)**: allowed between all tokens
- [x] **Deep trees (new)**: the reader keeps its own stack of open nodes; a test reads and writes back a caterpillar tree of depth 200,000
- [x] **Nodes with one child**: accepted without a warning

The differences in this list need a decision ([`N-undocumented-differences-from-treeknit-jl.md`](../issues/N-undocumented-differences-from-treeknit-jl.md)).

## `MCCs.json`

`fn to_json` [[src](../../packages/treeknit-io/src/mccs.rs#L6-L37)]:

- [x] **Structure**: `{"MCC_dict": {"1": {"trees": [a, b], "mccs": [[...]]}, ...}}`, keys in pair order. `serde_json` keeps the insertion order, so the keys appear as `"1"`, `"2"`, and so on
- [x] **`imputed` (new)**: for a pair with leaves that only one tree has, a list of `{leaf, tree, mcc, ambiguous}`, with `mcc` 0-based. Omitted when empty (see [`partial-overlap.md`](partial-overlap.md))

## MCC list as text

- [x] **Writer**: `to_lines`, one MCC per line, labels joined by `,`, no newline after the last line [[src](../../packages/treeknit-io/src/mccs.rs#L39-L42)]
- [x] **`MCCs.dat` (new as output)**: the command line writes this format again, as TreeKnit.jl 0.4 did. Downstream tools such as TreeTime `arg` read it. With more than two trees: `MCCs_<a>_<b>.dat` per pair
- [/] **Labels with `,`**: cannot be stored, as in TreeKnit.jl

## Resolved trees

`fn write` [[src](../../packages/treeknit-io/src/newick.rs#L63-L78)], in the classic dialect of `util-newick`:

- [x] **Order**: children in stored order
- [x] **Labels**: every node label, including internal nodes
- [x] **Lengths**: shortest exact text, with `.0` on whole numbers: `1.0`, `0.25`, `19329.779261588275`. Exponent notation below 1e-4 and from 1e16, with a point in the mantissa: `1.0e-20`, `2.5e16`. Missing lengths are omitted. A length that is not a finite number, which arithmetic on lengths can produce, stops the output with an error that names the file
- [/] **Root**: no length. TreeTools.jl always writes `:0` after the root
- [x] **Quoting (new)**: labels that the reader would read differently without quotes: with `(),:;[]'` or whitespace, and labels that end like an extended Newick hybrid tag, so `EPI_ISL#402124` is written `'EPI_ISL#402124'`
- [x] **Line end**: one tree, then a newline
- [x] **Deep trees (new)**: the writer keeps its own stack, so deep ladder trees cannot overflow the stack

## `parameters.json`

`fn parameters_file` [[src](../../packages/treeknit-io/src/output.rs#L596-L598)]:

- [x] **Time of writing**: before the inference, so the file exists when a run fails
- [x] **Layout**: compact, as in TreeKnit.jl
- [x] **Line end**: no newline after the closing brace
- [/] **Fields**: `gamma`, `itmax`, `likelihood_sort`, `resolution`, `seq_lengths`, `pre_resolve`, `rounds`, `final_unresolved_round`, `nMCMC`, `sa_rep`, `Tmin`, `Tmax`, `nT`, `cooling_schedule`, `naive`, and `seed`. TreeKnit.jl writes the `OptArgs` fields: `γ`, `resolve`, `strict`, `final_no_resolve`, `parallel`, and `Trange`, which the port does not write. A reader of the TreeKnit.jl file cannot read this file
- [x] **Values**: the values the run uses. TreeKnit.jl shows defaults for the five fields it ignores

## `log.txt`

- [/] **Format**: `<RFC 3339 time> [LEVEL] <message>` (see [`cli.md`](cli.md#logging))
- [x] **Web app (new)**: `fn log_file` writes the records of a run in the same layout, without a thread ID, with the time of the JavaScript clock in UTC with milliseconds [[src](../../packages/treeknit-io/src/output.rs#L602-L609)]

## `ARG/arg.nwk`

See [`arg.md`](arg.md#extended-newick-output).

- [x] **Syntax**: `<label>[&segments={...}]:<length>`, hybrids `label#H<i>`, extra `GlobalRoot`
- [/] **Lengths**: without the `eps()` offset of TreeKnit.jl
- [x] **Labels**: deterministic, from counters
- [x] **Quoting (new)**: labels quoted as in the resolved trees. TreeKnit.jl writes them as they are
- [x] **Line end**: a newline after the tree

## `ARG/nodes.dat`

`fn node_table` [[src](../../packages/treeknit-io/src/arg.rs#L47-L62)]:

- [x] **Lines**: `<ARG label>,<label in tree 1>,<label in tree 2>`, with one space for an absent node, in ARG node order
- [x] **Consistency**: the tree labels name nodes of the trees in `ARG/*_liberal_resolved*`, including the inserted singletons. In TreeKnit.jl, those files lack the singletons. This differs on purpose ([`docs/user/treeknit-jl.md`](../../docs/user/treeknit-jl.md#deliberate-differences-from-treeknitjl))
- [/] **Line end**: a newline after the last line; TreeKnit.jl writes none. Labels with `,` make the table ambiguous ([`M-arg-outputs-unquoted-labels.md`](../issues/M-arg-outputs-unquoted-labels.md))

## `auspice_<label>.json`

- [x] **Structure**: the Auspice v2 subset of TreeKnit.jl (see [`visualization.md`](visualization.md#auspice-json))
- [x] **Line end**: no newline after the closing brace

## Session file (`treeknit_session.json`, new)

The analysis request of the web app: the trees with their labels and Newick texts, and the settings. `fn session_file` writes it [[src](../../packages/treeknit-io/src/output.rs)], and `fn read_session` reads it [[src](../../packages/treeknit-io/src/analysis.rs)]. Numbers read back exactly, because `serde_json` parses with its `float_roundtrip` feature.

- [x] **Structure**: `{"trees": [{"label": ..., "newick": ...}], "settings": {...}}`, pretty JSON with a newline at the end. The settings use the camelCase names of the web app (`gamma`, `seqLengths`, `nMcmcIt`, `resolve`, `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`, `seed`); a missing setting takes its default
- [x] **Reading**: checks the structure only: types, required and unknown fields, and a seed of at most 2^53 - 1, which a JavaScript number holds exactly. A file with a broken tree or an out-of-range setting loads, so the web app can show the errors at their fields; the command line applies the shared validation after reading
- [x] **Writers**: the web app ("Save session file", and the archive of a run), and the command line with `--session`, `--example`, `--link`, or `--print-link`, which writes the request it ran into the results directory

## ZIP archive (new)

`fn zip_archive` packs the output files of a web app run [[src](../../packages/treeknit-io/src/output.rs#L612-L618)].

- [x] **Entries**: every file under `treeknit_results/`, the default results directory of the command line, at its path in that directory (`treeknit_results/ARG/arg.nwk`)
- [x] **Reproducible bytes**: every entry is deflated and dated 1980-01-01 00:00, the earliest ZIP time, so equal files give a byte-identical archive. `log.txt` carries clock times, so the archives of two runs differ in that entry
- [x] **Reproduction**: `treeknit --session treeknit_results/treeknit_session.json --impute --auspice-view`, run in the directory where the archive was extracted, writes the same file set with the command line

## Links (new)

A link of the web app names trees, settings, a run, and a view in its query, and inline data in its fragment. `treeknit_io::launch` reads it (`fn parse_launch`) and writes it (`fn launch_pairs`, `fn inline_session`) for both surfaces, so the web app and the command line (`--link`, `--print-link`) share one grammar [[src](../../packages/treeknit-io/src/launch.rs)].

- [x] **Input keys**: one kind per link: `example=<id>` (an id of the catalog of `treeknit_io::examples`, which the Examples menu and `--example` share; ids stay fixed once published), `tree=[<label>=]<location>` repeated for at least two trees, `session=<location>`, or `from=opener|parent` for a session file that another window posts. A second kind is an error at its key
- [x] **Labels**: the text before the first `=` of a tree is its label when it is not empty and holds no `:`, `/`, or `\`, which a label cannot hold and every location holds before its first `=`. Unlabeled trees take `analysis::tree_labels` of the percent-decoded last path segment of their address, next to the given labels; a `data:` tree has no file name and gets `tree`, `tree_2`
- [x] **Locations**: `https:` addresses without a user name or password, and `data:` texts (RFC 2397, percent-encoded or `;base64` in the standard or URL-safe alphabet), decoded while parsing. A GitHub file page (`github.com/<o>/<r>/blob/<ref>/<path>` or `/raw/`) is read from `raw.githubusercontent.com`, and a Zenodo record file (`zenodo.org/records/<id>/files/<name>`) from its `/api/records/<id>/files/<name>/content` address, because only those send `Access-Control-Allow-Origin`; the link keeps the address as given
- [x] **Downloads**: gzip-compressed bytes are decompressed (several gzip members too, as bgzip writes them) up to 256 MiB of text; the text must be UTF-8, and a web page (`<!doctype html` or `<html` at the start) is rejected. A download stops after 60 seconds or 64 MiB (`FETCH_TIMEOUT_SECONDS`, `MAX_DOWNLOAD_BYTES`), in the web app and in the command line
- [x] **Settings keys**: the long flags of the command line without `--`: `gamma`, `seq-lengths` (comma list), `n-mcmc-it`, `resolve`, `rounds`, `seed`, and the flags `pre-resolve`, `no-final-round`, `no-likelihood`, `naive` with their opposites `no-pre-resolve`, `final-round`, `likelihood`, `no-naive`, from the key table `schema::SETTING_KEYS`. A value of the wrong type is an error that leaves the setting unset; a value out of bounds goes into the settings, so the form shows it at its field. A repeated key with a value is an error; of a flag and its opposite the last wins. The settings apply to the defaults, or to the settings of the session file
- [x] **`run`, `v`**: `run` starts the run once the inputs have loaded. `v` is the format version: absent means 1, and a larger version than the build reads stops the launch with "This link needs a newer version of TreeKnit". Settings or `run` without an input are an error
- [x] **Other keys**: keys of the display are read by the web app (see [`web-app.md`](web-app.md#web-app)); any other key is ignored, with the launch or display key it most likely misspells (edit distance 2, keys of at least 4 characters) and, after a location with a `?`, the hint that `&` inside an address is written `%26`
- [x] **Query text**: pairs joined by `&`, a flag without `=`; only `%`, `&`, `#`, `+`, `=` in keys, spaces, `"`, `<`, `>`, control and non-ASCII characters are percent-encoded, so addresses and Newick texts stay readable. Reading follows `URLSearchParams` (`+` is a space). The fragment holds keys only when it contains `=`, so `#help-cite` stays an anchor
- [x] **Canonical link**: `launch_pairs` writes the input keys, the settings that differ from the defaults in the order of the key table, and `run`: `example=<id>` when the trees are those of the example in its order with its labels, one `tree` per tree otherwise, with `<label>=` only where the label differs from the one the address gives. A tree from a file has no address, and no link is written
- [x] **Inline session**: `data:application/gzip;base64,` and the gzip of the session file, for the fragment: `?run&<view keys>#session=data:...`. The whole session compresses better than the trees one by one, because they share their leaf names. Links longer than 32,000 characters are not written (`MAX_LINK_CHARS`)

## Other formats

- Not ported: `parse_nexus`, which the TreeKnit.jl module does not include and which calls an undefined function
