# File formats

TreeKnit reads Newick trees and writes JSON, Newick, extended Newick, and plain-text files. TreeTools.jl 0.6.14 reads and writes the Newick trees. TreeKnit writes all other formats itself.

## Input: Newick

`read_tree(file; label)` from TreeTools reads each input file [[src](https://github.com/PierreBarrat/TreeTools.jl/blob/2fb33ac3a5891a9cf0d25c76d2bc0270e571a6b8/src/reading.jl#L48-L87)]. The parser splits strings on `)`, `,`, and `:`, so it accepts a narrower dialect than the Newick standard.

- **One tree per line**: each non-empty line is one tree. A file with several trees gives a vector, which the command line does not handle
- **Terminator**: each line must end with `;` as its last character. A trailing space or a Windows line end (`\r`) fails the assertion "Newick string does not end with ';'"
- **Branch lengths**: the text after `:` is parsed as a `Float64`. A missing or unparsable length becomes `missing` without a warning [[src](https://github.com/PierreBarrat/TreeTools.jl/blob/2fb33ac3a5891a9cf0d25c76d2bc0270e571a6b8/src/reading.jl#L224-L240)]. A label with two `:` gives a warning
- **Root**: the outermost node is the root, and its branch length is always set to `missing`. A root with three or more children stays a polytomy
- **Unlabeled nodes**: each node without a name, leaf or internal, gets `NODE_<n>`, numbered in pre-order from 1 for each tree [[src](https://github.com/PierreBarrat/TreeTools.jl/blob/2fb33ac3a5891a9cf0d25c76d2bc0270e571a6b8/src/reading.jl#L148-L156)]. A name that starts with `[&` becomes `NODE_<n>[&...]`
- **Numeric internal labels**: an internal label that parses as a number, or as numbers joined by `/` (for example `87` or `0.95/87`), counts as a support value. It is renamed to `<label>__<8 random characters>`, so the output names change from run to run [[src](https://github.com/PierreBarrat/TreeTools.jl/blob/2fb33ac3a5891a9cf0d25c76d2bc0270e571a6b8/src/methods.jl#L40-L81)]. Leaf names are never renamed
- **Duplicate labels**: a label that appears twice is an error "Node X appears twice in tree. Use `force_new_labels`." The docstring says a random suffix is added, but the code raises the error
- **Not supported**: quoted labels (quotes stay in the label, and quoted `,`, `(`, `)`, `:` break the parse), whitespace between tokens, and comments (`[...]` stays in the label or makes the length unparsable)
- **Singletons**: a node with one child is accepted. `check_tree` prints a warning, and reading continues

## `MCCs.json`

`write_mccs(filename, MCCs::MCC_set)` writes the MCCs of all pairs with `JSON3.pretty` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_IO.jl#L54-L78)]. A file name without `.json` gives a warning, and the file is still JSON.

```json
{
  "MCC_dict": {
    "1": {
      "trees": ["a", "b"],
      "mccs": [["A"], ["B", "C"]]
    },
    "2": {
      "trees": ["a", "c"],
      "mccs": [["A", "B", "C"]]
    }
  }
}
```

- **Keys**: `"1"` to `"K(K-1)/2"`, numbered in pair order `(1,2), (1,3), ..., (K-1,K)`. The writer builds a `Dict{Int,Any}`, so the order of the keys in the text follows the dictionary, not the numbers
- **`trees`**: the two tree labels in input order
- **`mccs`**: the MCCs of the pair, sorted as described in [`mcc-inference.md`](mcc-inference.md)
- **Reading back**: no function reads this file. Downstream pipelines read the first entry of `MCC_dict` and rely on the size order of the MCC list; TreeTime `arg` reads only the line-based format below (see [`treeknit-ecosystem.md`](../reports/treeknit-ecosystem.md))

## MCC list as text

`write_mccs(filename, MCCs::AbstractArray, mode = "w")` writes one MCC per line with labels joined by `,`, and no newline after the last line [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_IO.jl#L29-L46)]. `read_mccs(file)` reads this format back as `Vector{Vector{String}}` [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_IO.jl#L48-L52)]. Labels that contain `,` cannot be stored. The command line does not use this format.

## Resolved trees

`<name>_resolved<ext>` and `ARG/<name>_liberal_resolved<ext>` are written with TreeTools `write_newick` [[src](https://github.com/PierreBarrat/TreeTools.jl/blob/2fb33ac3a5891a9cf0d25c76d2bc0270e571a6b8/src/writing.jl#L51-L98)]:

- **Order**: children in their stored order, which is the order after ladderizing and sorting
- **Labels**: every node label, including internal ones and the root (`NODE_<n>`, `RESOLVED_<i>`, `Singleton_...` are kept)
- **Lengths**: `:<length>` with Julia's shortest round-trip printing of `Float64` (for example `0.0` and `1.0e-5`). A `missing` length is omitted. The root always ends with `:0`, for example `(...)NODE_1:0;`
- **Line end**: one tree followed by a newline

## `parameters.json`

The `OptArgs` of the run, written with `JSON3.pretty` before the inference [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L131-L134)]. It holds every field of `OptArgs` (see [`pipeline.md`](pipeline.md)), including `Trange` with all temperatures and `cooling_schedule` as a string. Because `OptArgs(K; ...)` ignores `γ`, `nMCMC`, `likelihood_sort`, `seq_lengths`, and `parallel`, the file shows the defaults for these fields, not the command-line values. The upstream example shows the full content [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/examples/treeknit_results/parameters.json#L1-L20)].

## `log.txt`

One line per message: `<path>:<line> [<level>] [HH:MM] - <message>` (see [`cli.md`](cli.md)). Messages with line breaks continue over several lines.

## `ARG/arg.nwk`

Extended Newick in one line [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/SimpleReassortmentGraph/IO.jl#L21-L144)]. Each node is written as `<label>[&segments={<s>}]:<length>`:

- **Segments**: `{0,1}` for a node and branch in both segments, `{0}` or `{1}` for one segment (0-based). For a hybrid node, the annotation gives the segment of the edge by which the traversal reached it
- **Hybrid nodes**: the label is `ARGNode_<random>#H<i>` in the `label#<type><i>` form of extended Newick, with the type `H` (hybridization), with `<i>` counted from 1 in the order of the traversal. The node appears twice, once with its subtree and once as a leaf-like reference with the other segment's length
- **Lengths**: omitted when `missing`, for example at the root. Values include the `eps()` offset (see [`arg.md`](arg.md))
- **Labels**: leaves keep their names. Internal nodes and hybrid nodes have random labels, so two runs give different labels. Labels are never quoted, so a strain name with one of the characters `,():;[]#` makes the file unreadable for extended Newick parsers such as IcyTree
- **Extra root**: `GlobalRoot[&segments={0,1}]:0.` when the two segment roots differ and neither is shared

## `ARG/nodes.dat`

One line per ARG node, with no header and no newline after the last line [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/cli.jl#L182-L202)]:

```
<ARG label>,<label in tree 1>,<label in tree 2>
```

A node that is absent from a tree has a single space in that column. Hybrid nodes have spaces in both columns. The line order follows a dictionary, not the tree. The tree labels refer to the trees inside `arg_from_trees`, after its own resolution and the insertion of `Singleton_...` nodes. The trees in `ARG/*_liberal_resolved*` are written before that step, so they lack the singleton nodes that `nodes.dat` names.

## `auspice_<label>.json`

Written by `write_auspice_json` (see [`visualization.md`](visualization.md)) [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/mcc_IO.jl#L104-L152)]:

```json
{
  "version": "v2",
  "meta": {
    "updated": "",
    "colorings": [{ "key": "mcc_a_b", "title": "mcc_a_b", "type": "ordinal" }],
    "filters": [],
    "panels": ["tree"]
  },
  "tree": {
    "name": "NODE_1",
    "node_attrs": { "div": 0.0, "mcc_a_b": { "value": "2" } },
    "branch_attrs": {},
    "children": []
  }
}
```

Leaves have no `children` key. Internal nodes have a `children` array. The empty `updated` value does not match the date pattern of the augur export schema, and the MCC values are strings, so auspice uses a categorical color scale and shows `"null"` as a category (see [`visualization.md`](visualization.md)).

## Code that reads other formats

`parse_nexus(infile)` would read mutations from a TreeTime nexus file [[src](https://github.com/PierreBarrat/TreeKnit.jl/blob/dbbc89ac691fed0949a622eedbae103787b89320/src/reading.jl#L1-L35)]. The module does not include `src/reading.jl`, and the function calls `parse_muts`, which is defined nowhere, so this code is not available.
