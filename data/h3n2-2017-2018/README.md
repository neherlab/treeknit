# Human influenza A/H3N2, 2017-2018

HA (segment 4) and NA (segment 6) trees of 53 human A/H3N2 strains sampled worldwide: 13 in 2017 and 40 in 2018. The two trees have the same leaves, so the pair is a complete TreeKnit input:

```sh
treeknit data/h3n2-2017-2018/ha.nwk data/h3n2-2017-2018/na.nwk
```

## Files

- `ha.nwk`: HA tree, branch lengths most likely in substitutions per site (TreeKnit.jl does not state the unit)
- `na.nwk`: NA tree, same leaves

The trees are not raw phylogenies: an early version of TreeKnit has already processed them. Internal node labels come from three sources:

- `NODE_<n>`: the original tree, in the naming of the Nextstrain tool augur
- `RESOLVED_<n>`: a node added by polytomy resolution
- `MCC_<n>` and `shared_<n>_<m>`: nodes labeled by the MCC (maximally compatible clade) inference

## Provenance

- **Trees**: a test case of TreeKnit.jl, published in the TreeKnit.jl repository as `test/splitgraph/ondata/N18/tha.nwk` and `tna.nwk` at commit [`dbbc89a`](https://github.com/PierreBarrat/TreeKnit.jl/tree/dbbc89ac691fed0949a622eedbae103787b89320/test/splitgraph/ondata/N18). TreeKnit.jl documents neither the source sequences nor the tree construction, and its test suite does not read these files
- **Sequences, inferred**: very likely GISAID EpiFlu, through a Nextstrain seasonal-flu build. The strain names follow the Nextstrain spelling without spaces (`A/StPetersburg/RII-532/2018`), and five of six sampled strains have no GenBank record
- **Copy**: byte-identical to the TreeKnit.jl files

| File     | SHA-256                                                            |
| -------- | ------------------------------------------------------------------ |
| `ha.nwk` | `2cb8d7ed7172ca2171a0fc92d9e9ef2fc1858944137c8cd519bbd37d14ea6d47` |
| `na.nwk` | `fa08394acf77cb8e4ac2ce06fd477261f09ff27aaadd14b60c7a41e3c5b9becd` |

## Terms of use

- **Trees**: distributed in TreeKnit.jl under the MIT license, copyright 2021 Pierre Barrat-Charlaix
- **Sequences**: unconfirmed. If the trees derive from GISAID data, the GISAID terms of use apply: they allow publishing results of analyses with an acknowledgment of the originating and submitting laboratories, and forbid distributing the data, including derivatives, to anyone who is not a registered GISAID user. TreeKnit.jl lists no GISAID isolate identifiers, so the laboratories cannot be acknowledged

## References

- Barrat-Charlaix P, Vaughan TG, Neher RA. TreeKnit: Inferring ancestral reassortment graphs of influenza viruses. _PLOS Computational Biology_ 18: e1010394, 2022. <https://doi.org/10.1371/journal.pcbi.1010394>
- GISAID terms of use: <https://gisaid.org/terms-of-use/>
