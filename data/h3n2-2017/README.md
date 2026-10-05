# Human influenza A/H3N2, 2017

HA (segment 4) and NA (segment 6) trees of 100 human A/H3N2 strains: 99 isolates sampled worldwide in 2017, and the root `A/Victoria/361/2011`. The two trees have the same leaves, so the pair is a complete TreeKnit input:

```sh
treeknit data/h3n2-2017/ha.nwk data/h3n2-2017/na.nwk
```

## Files

- `ha.nwk`: HA tree, rooted on `A/Victoria/361/2011`, branch lengths most likely in substitutions per site (TreeKnit.jl does not state the unit)
- `na.nwk`: NA tree, same leaves and root

Internal nodes are labeled `NODE_<n>`, the naming of the Nextstrain tool augur.

## Provenance

- **Trees**: the example of the TreeKnit.jl documentation, published in the TreeKnit.jl repository as `examples/tree_h3n2_ha.nwk` and `examples/tree_h3n2_na.nwk` at commit [`dbbc89a`](https://github.com/PierreBarrat/TreeKnit.jl/tree/dbbc89ac691fed0949a622eedbae103787b89320/examples). TreeKnit.jl documents neither the source sequences nor the tree construction
- **Sequences, inferred**: very likely GISAID EpiFlu, through a Nextstrain seasonal-flu build. The strain names follow the Nextstrain spelling without spaces (`A/HongKong/2291/2017`), the root `A/Victoria/361/2011` is the reference of Nextstrain H3N2 builds, and none of nine sampled 2017 strains has a GenBank record
- **Copy**: byte-identical to the TreeKnit.jl files

| File     | SHA-256                                                            |
| -------- | ------------------------------------------------------------------ |
| `ha.nwk` | `18908963604b09bf4458365caf064e252b4aa5726bd2c3269cbb77d6e996dfa0` |
| `na.nwk` | `a68c0d78aef33b88b9a57e753d1ba707108e1dce7b2cba4777fa78e163336ef2` |

## Terms of use

- **Trees**: distributed in TreeKnit.jl under the MIT license, copyright 2021 Pierre Barrat-Charlaix
- **Sequences**: unconfirmed. If the trees derive from GISAID data, the GISAID terms of use apply: they allow publishing results of analyses with an acknowledgment of the originating and submitting laboratories, and forbid distributing the data, including derivatives, to anyone who is not a registered GISAID user. TreeKnit.jl lists no GISAID isolate identifiers, so the laboratories cannot be acknowledged

## References

- Barrat-Charlaix P, Vaughan TG, Neher RA. TreeKnit: Inferring ancestral reassortment graphs of influenza viruses. _PLOS Computational Biology_ 18: e1010394, 2022. <https://doi.org/10.1371/journal.pcbi.1010394>
- GISAID terms of use: <https://gisaid.org/terms-of-use/>
