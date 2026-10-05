# Example data

Real influenza A/H3N2 segment trees for running TreeKnit. Each directory holds one tree per segment with the same leaves, named after the segment (`ha.nwk`, `na.nwk`, `pb1.nwk`, `pb2.nwk`). The directories of the HA and NA pairs also hold a `README.md` with provenance, checksums, and terms of use.

| Directory                  | Strains | Segments         | Sampled        | Sequence source         |
| -------------------------- | ------: | ---------------- | -------------- | ----------------------- |
| `h3n2-new-york-1999-2004/` |     154 | HA, NA           | New York State | GenBank (open)          |
| `h3n2-2017/`               |     100 | HA, NA           | worldwide      | likely GISAID, see note |
| `h3n2-2017-2018/`          |      53 | HA, NA           | worldwide      | likely GISAID, see note |
| `h3n2-2012-2018/`          |     428 | HA, NA           | worldwide      | likely GISAID, see note |
| `h3n2-2k-4-segments/`      |    1997 | HA, NA, PB1, PB2 | worldwide      | not documented          |

The HA and NA pairs come unchanged from the [TreeKnit.jl](https://github.com/PierreBarrat/TreeKnit.jl) repository, which distributes them under the MIT license. Only the New York pair has a documented open sequence source. The other three pairs very likely derive from GISAID data, whose terms of use restrict the distribution of derived data; the README of each directory states the evidence and the terms.
