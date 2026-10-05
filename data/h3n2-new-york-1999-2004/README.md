# Human influenza A/H3N2, New York 1999-2004

HA (segment 4) and NA (segment 6) trees of 154 human A/H3N2 strains: 153 isolates sampled in New York State between 1999 and 2004, and the outgroup `A/Sydney/5/1997`. The two trees have the same leaves, so the pair is a complete TreeKnit input:

```sh
treeknit data/h3n2-new-york-1999-2004/ha.nwk data/h3n2-new-york-1999-2004/na.nwk
```

## Files

- `ha.nwk`: HA tree, rooted on `A/Sydney/5/1997`, branch lengths in substitutions per site
- `na.nwk`: NA tree, same leaves and root

Internal node labels have the form `<bootstrap support>__<random suffix>`, for example `97__F8oQ4`.

## Provenance

- **Sequences**: whole genomes from the NIAID Influenza Genome Sequencing Project, analyzed in Holmes et al. 2005 and deposited in GenBank. The data set is a common test case for reassortment inference
- **Trees**: built by the authors of TreeKnit for Barrat-Charlaix et al. 2022 (IQ-TREE maximum likelihood with ultrafast bootstrap, Supplementary Figures S1 and S2), and published in the TreeKnit.jl repository as `test/NYdata/tree_ha.nwk` and `test/NYdata/tree_na.nwk` at commit [`dbbc89a`](https://github.com/PierreBarrat/TreeKnit.jl/tree/dbbc89ac691fed0949a622eedbae103787b89320/test/NYdata)
- **Copy**: byte-identical to the TreeKnit.jl files

| File     | SHA-256                                                            |
| -------- | ------------------------------------------------------------------ |
| `ha.nwk` | `c95a5df42985892df5655c047c777df16fb0fe0088e1ab7175fbdf712e1f7e88` |
| `na.nwk` | `4463ff3de271134bb819ae21809b606f7d3f387644e681e9236136cd6d2a7b2d` |

## Terms of use

- **Sequences**: GenBank. NCBI places no restrictions on the use or distribution of GenBank data ([GenBank data usage](https://www.ncbi.nlm.nih.gov/genbank/about/))
- **Trees**: distributed in TreeKnit.jl under the MIT license, copyright 2021 Pierre Barrat-Charlaix

## References

- Holmes EC, Ghedin E, Miller N, et al. Whole-genome analysis of human influenza A virus reveals multiple persistent lineages and reassortment among recent H3N2 viruses. _PLoS Biology_ 3(9): e300, 2005. <https://doi.org/10.1371/journal.pbio.0030300>
- Barrat-Charlaix P, Vaughan TG, Neher RA. TreeKnit: Inferring ancestral reassortment graphs of influenza viruses. _PLOS Computational Biology_ 18: e1010394, 2022. <https://doi.org/10.1371/journal.pcbi.1010394>
