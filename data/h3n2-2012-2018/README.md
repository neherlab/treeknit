# Human influenza A/H3N2, 2012-2018

HA (segment 4) and NA (segment 6) trees of 428 human A/H3N2 strains: 427 isolates sampled worldwide between 2012 and 2018, and the root `A/Victoria/361/2011`. The two trees have the same leaves, so the pair is a complete TreeKnit input:

```sh
treeknit data/h3n2-2012-2018/ha.nwk data/h3n2-2012-2018/na.nwk
```

| Year    | 2011 | 2012 | 2013 | 2014 | 2015 | 2016 | 2017 | 2018 |
| ------- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Strains |    1 |    2 |   21 |   49 |  101 |   79 |   98 |   77 |

## Files

- `ha.nwk`: HA tree, rooted on `A/Victoria/361/2011`, branch lengths most likely in substitutions per site (TreeKnit.jl does not state the unit)
- `na.nwk`: NA tree, same leaves and root

The trees are not raw phylogenies: an early version of TreeKnit has already processed them. Internal node labels come from three sources:

- `NODE_<n>`: the original tree, in the naming of the Nextstrain tool augur
- `RESOLVED_<n>`: a node added by polytomy resolution
- `MCC_<n>` and `shared_<n>_<m>`: nodes labeled by the MCC (maximally compatible clade) inference

## Provenance

- **Trees**: test data of TreeKnit.jl, published in the TreeKnit.jl repository as `test/splitgraph/ondata/alltree/tha.nwk` and `tna.nwk` at commit [`dbbc89a`](https://github.com/PierreBarrat/TreeKnit.jl/tree/dbbc89ac691fed0949a622eedbae103787b89320/test/splitgraph/ondata/alltree). The same directory holds `out_ha.nwk` and `out_na.nwk`, TreeKnit outputs with fewer leaves, which are not copied here. TreeKnit.jl documents neither the source sequences nor the tree construction, and its test suite does not read these files
- **Sequences, inferred**: very likely GISAID EpiFlu, through a Nextstrain seasonal-flu build. The strain names follow the Nextstrain spelling without spaces, the root `A/Victoria/361/2011` is the reference of Nextstrain H3N2 builds, and eight of ten sampled strains have no GenBank record
- **Copy**: byte-identical to the TreeKnit.jl files

| File     | SHA-256                                                            |
| -------- | ------------------------------------------------------------------ |
| `ha.nwk` | `bccbce588be451a3ecff73f5466ae343a44900778b29a9ee7665f52eff8868d4` |
| `na.nwk` | `43f79f525055b48ae12a63333b2ed4a240b922ba716dee8ed6441b245a652546` |

## Terms of use

- **Trees**: distributed in TreeKnit.jl under the MIT license, copyright 2021 Pierre Barrat-Charlaix
- **Sequences**: unconfirmed. If the trees derive from GISAID data, the GISAID terms of use apply: they allow publishing results of analyses with an acknowledgment of the originating and submitting laboratories, and forbid distributing the data, including derivatives, to anyone who is not a registered GISAID user. TreeKnit.jl lists no GISAID isolate identifiers, so the laboratories cannot be acknowledged

## References

- Barrat-Charlaix P, Vaughan TG, Neher RA. TreeKnit: Inferring ancestral reassortment graphs of influenza viruses. _PLOS Computational Biology_ 18: e1010394, 2022. <https://doi.org/10.1371/journal.pcbi.1010394>
- GISAID terms of use: <https://gisaid.org/terms-of-use/>
