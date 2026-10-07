# ARG node table writes labels without quotes

`ARG/nodes.dat` joins the label of each ARG node and the labels of its tree nodes with `,`, without quotes (`fn node_table` [packages/treeknit-io/src/arg.rs#L47-L62](../../packages/treeknit-io/src/arg.rs#L47-L62)). A label that contains `,` makes its line ambiguous. `ARG/arg.nwk` quotes such labels, as Newick needs.

## Reproduction

```sh
printf "((A,'B,1'),(C,(D,X)));\n" > q1.nwk
printf "((A,('B,1',X)),(C,D));\n" > q2.nwk
treeknit q1.nwk q2.nwk -o out
```

`out/ARG/arg.nwk` contains `'B,1'`. `out/ARG/nodes.dat` has the line `B,1,B,1,B,1`, which has six fields instead of three.

## Impact

- Strain names of influenza often contain `/` and sometimes `|` or spaces; a name with `,` makes `nodes.dat` ambiguous for every reader
- TreeKnit.jl has the same defect

## Fix direction

- Write `nodes.dat` with quoted fields, or document that labels with `,` are not supported there. `MCCs.dat` has the same limit, as in TreeKnit.jl

## Validation

- A test of `node_table` with the leaf `B,1` gives three fields per line
