# Example data

Real influenza A/H3N2 segment trees for running TreeKnit. Each directory holds one HA tree (`ha.nwk`) and one NA tree (`na.nwk`) with the same leaves, and a `README.md` with provenance, checksums, and terms of use.

| Directory                  | Strains | Sampled        | Sequence source         |
| -------------------------- | ------: | -------------- | ----------------------- |
| `h3n2-new-york-1999-2004/` |     154 | New York State | GenBank (open)          |
| `h3n2-2017/`               |     100 | worldwide      | likely GISAID, see note |
| `h3n2-2017-2018/`          |      53 | worldwide      | likely GISAID, see note |
| `h3n2-2012-2018/`          |     428 | worldwide      | likely GISAID, see note |

All trees come unchanged from the [TreeKnit.jl](https://github.com/PierreBarrat/TreeKnit.jl) repository, which distributes them under the MIT license. Only the New York pair has a documented open sequence source. The other three pairs very likely derive from GISAID data, whose terms of use restrict the distribution of derived data; the README of each directory states the evidence and the terms.
