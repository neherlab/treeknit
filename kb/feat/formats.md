# File formats: parity checklist

Counterpart: [`v0/formats.md`](v0/formats.md). `treeknit-io` reads and writes all formats. TreeKnit.jl uses TreeTools.jl for Newick.

## Input: Newick

`fn parse` and `fn parse_first` [[src](../../packages/treeknit-io/src/newick.rs#L257-L288)]:

- [/] **Several trees in one file**: the first tree is used, with the warning "<label>: more than one tree in file, using the first". A tree ends at the first `;` outside quoted labels and `[...]` comments, so `('a;b',C);[x;y]` is one tree. The validation logs the warning and the tree inspection of the web app reports it, also when the first tree does not parse. TreeKnit.jl reads a vector, which its command line does not handle
- [x] **Terminator**: `;` is required. Whitespace and `\r` after it are accepted; TreeTools.jl rejects them
- [x] **Branch lengths**: parsed as `f64`. An invalid length becomes missing with the warning "<label>: ignoring invalid branch length '<x>'" [[src](../../packages/treeknit-io/src/newick.rs#L171-L192)]. TreeTools.jl gives no warning
- [x] **Root**: the root branch length is dropped. A root polytomy stays a polytomy
- [x] **Unnamed internal nodes**: `NODE_<k>`, numbered in pre-order for each tree [[src](../../packages/treeknit-io/src/newick.rs#L305-L336)]
- [/] **Unnamed leaves**: an error "unnamed leaf". TreeTools.jl names them `NODE_<k>`
- [/] **Numeric internal labels**: support values such as `87` or `0.95` become `NODE_<k>`, so the output has the same names in every run and loses the support values. TreeTools.jl renames them `<label>__<random>`
- [x] **Duplicate leaf names**: an error, as in TreeTools.jl
- [/] **Duplicate internal names**: renamed `NODE_<k>`. TreeTools.jl raises an error
- [x] **Quoted labels (new)**: `'a b'`, with `''` for a quote character [[src](../../packages/treeknit-io/src/newick.rs#L137-L169)]
- [x] **Comments (new)**: `[...]` is skipped, including annotations such as `[&x=1]`
- [x] **Whitespace (new)**: allowed between all tokens
- [x] **Nodes with one child**: accepted without a warning

The differences in this list need a decision ([`N-undocumented-differences-from-treeknit-jl.md`](../issues/N-undocumented-differences-from-treeknit-jl.md)).

## `MCCs.json`

`fn to_json` [[src](../../packages/treeknit-io/src/mccs.rs#L6-L37)]:

- [x] **Structure**: `{"MCC_dict": {"1": {"trees": [a, b], "mccs": [[...]]}, ...}}`, keys in pair order. `serde_json` keeps the insertion order, so the keys appear as `"1"`, `"2"`, and so on
- [x] **`imputed` (new)**: for a pair with leaves that only one tree has, a list of `{leaf, tree, mcc, ambiguous}`, with `mcc` 0-based. Omitted when empty (see [`partial-overlap.md`](partial-overlap.md))
- [/] **Reading back (new)**: `fn from_json` reads the format [[src](../../packages/treeknit-io/src/mccs.rs#L42-L55)]. No code calls it and no test covers it. TreeKnit.jl has no reader

## MCC list as text

- [x] **Writer and reader**: `to_lines` and `from_lines`, one MCC per line, labels joined by `,`, no newline after the last line [[src](../../packages/treeknit-io/src/mccs.rs#L57-L67)]
- [x] **`MCCs.dat` (new as output)**: the command line writes this format again, as TreeKnit.jl 0.4 did. Downstream tools such as TreeTime `arg` read it. With more than two trees: `MCCs_<a>_<b>.dat` per pair
- [/] **Labels with `,`**: cannot be stored, as in TreeKnit.jl. `from_lines` also trims whitespace around each label

## Resolved trees

`fn write` [[src](../../packages/treeknit-io/src/newick.rs#L338-L391)]:

- [x] **Order**: children in stored order
- [x] **Labels**: every node label, including internal nodes
- [x] **Lengths**: shortest round-trip form, for example `1.0` and `1e-20`. Missing lengths are omitted
- [/] **Root**: no length. TreeTools.jl always writes `:0` after the root
- [x] **Quoting (new)**: labels with `(),:;[]'` or whitespace are quoted
- [x] **Line end**: one tree, then a newline
- [x] **Deep trees (new)**: an iterative writer, so deep ladder trees cannot overflow the stack

## `parameters.json`

`fn parameters_file` [[src](../../packages/treeknit-io/src/output.rs#L559-L563)]:

- [x] **Time of writing**: before the inference, so the file exists when a run fails
- [x] **Line end**: no newline after the closing brace
- [/] **Fields**: `gamma`, `itmax`, `likelihood_sort`, `resolution`, `seq_lengths`, `pre_resolve`, `rounds`, `final_unresolved_round`, `nMCMC`, `sa_rep`, `Tmin`, `Tmax`, `nT`, `cooling_schedule`, `naive`, and `seed`. TreeKnit.jl writes the `OptArgs` fields: `γ`, `resolve`, `strict`, `final_no_resolve`, `parallel`, and `Trange`, which the port does not write. A reader of the TreeKnit.jl file cannot read this file
- [x] **Values**: the values the run uses. TreeKnit.jl shows defaults for the five fields it ignores

## `log.txt`

- [/] **Format**: `<RFC 3339 time> [LEVEL] <message>` (see [`cli.md`](cli.md#logging))
- [x] **Web app (new)**: `fn log_file` writes the records of a run in the same layout, without a thread ID, with the time of the JavaScript clock in UTC with milliseconds [[src](../../packages/treeknit-io/src/output.rs#L565-L574)]

## `ARG/arg.nwk`

See [`arg.md`](arg.md#extended-newick-output).

- [x] **Syntax**: `<label>[&segments={...}]:<length>`, hybrids `label#H<i>`, extra `GlobalRoot`
- [/] **Lengths**: without the `eps()` offset of TreeKnit.jl
- [x] **Labels**: deterministic, from counters
- [/] **Quoting**: none ([`M-arg-outputs-unquoted-labels.md`](../issues/M-arg-outputs-unquoted-labels.md))
- [x] **Line end**: a newline after the tree

## `ARG/nodes.dat`

`fn node_table` [[src](../../packages/treeknit-io/src/arg.rs#L89-L105)]:

- [x] **Lines**: `<ARG label>,<label in tree 1>,<label in tree 2>`, with one space for an absent node, in ARG node order
- [x] **Consistency**: the tree labels name nodes of the trees in `ARG/*_liberal_resolved*`, including the inserted singletons. In TreeKnit.jl, those files lack the singletons. This differs on purpose ([README](../../README.md#deliberate-differences-from-treeknitjl))
- [/] **Line end**: a newline after the last line; TreeKnit.jl writes none. Labels with `,` make the table ambiguous ([`M-arg-outputs-unquoted-labels.md`](../issues/M-arg-outputs-unquoted-labels.md))

## `auspice_<label>.json`

- [x] **Structure**: the Auspice v2 subset of TreeKnit.jl (see [`visualization.md`](visualization.md#auspice-json))
- [x] **Line end**: no newline after the closing brace

## Session file (`treeknit_request.json`, new)

The analysis request of the web app: the trees with their labels and Newick texts, and the settings. `fn request_file` writes it [[src](../../packages/treeknit-io/src/output.rs#L685-L694)], and `fn read_request` reads it [[src](../../packages/treeknit-io/src/analysis.rs#L361-L384)].

- [x] **Structure**: `{"trees": [{"label": ..., "newick": ...}], "settings": {...}}`, pretty JSON with a newline at the end. The settings use the camelCase names of the web app (`gamma`, `seqLengths`, `nMcmcIt`, `resolve`, `preResolve`, `rounds`, `finalRound`, `likelihood`, `naive`, `seed`); a missing setting takes its default
- [x] **Reading**: checks the structure only: types, required and unknown fields, and a seed of at most 2^53 - 1, which a JavaScript number holds exactly. A file with a broken tree or an out-of-range setting loads, so the web app can show the errors at their fields; the command line applies the shared validation after reading
- [x] **Writers**: the web app ("Save session file", and the archive of a run), and the command line with `--request`, which writes the request it ran into the results directory

## ZIP archive (new)

`fn zip_archive` packs the output files of a web app run [[src](../../packages/treeknit-io/src/output.rs#L576-L683)].

- [x] **Entries**: every file under `treeknit_results/`, the default results directory of the command line, at its path in that directory (`treeknit_results/ARG/arg.nwk`)
- [x] **Reproducible bytes**: every entry is deflated and dated 1980-01-01 00:00, the earliest ZIP time, so equal files give a byte-identical archive. `log.txt` carries clock times, so the archives of two runs differ in that entry
- [x] **Reproduction**: `treeknit --request treeknit_results/treeknit_request.json --impute --auspice-view`, run in the directory where the archive was extracted, writes the same file set with the command line

## Other formats

- Not ported: `parse_nexus`, which the TreeKnit.jl module does not include and which calls an undefined function
