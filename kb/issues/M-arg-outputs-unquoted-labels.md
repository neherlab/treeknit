# ARG files write labels without quotes

The Newick reader accepts quoted labels such as `'B,1'`, and the Newick writer quotes labels that contain `(),:;[]'` or whitespace. The ARG writer does not: `Writer::node` appends the label as it is ([arg.rs#L59](../../packages/treeknit-io/src/arg.rs#L59)). `node_table` joins the labels with `,` without quotes ([arg.rs#L89-L105](../../packages/treeknit-io/src/arg.rs#L89-L105)).

## Reproduction

```sh
printf "((A,'B,1'),(C,(D,X)));\n" > q1.nwk
printf "((A,('B,1',X)),(C,D));\n" > q2.nwk
treeknit q1.nwk q2.nwk -o out
```

`out/q1_resolved.nwk` contains `'B,1'`. `out/ARG/arg.nwk` contains `(B,1[&segments={0,1}],...)`, which an extended Newick reader parses as two leaves `B` and `1`. `out/ARG/nodes.dat` has the line `B,1,B,1,B,1`, which has six fields instead of three.

## Impact

- Strain names of influenza often contain `/` and sometimes `|` or spaces. A name with one of `,():;[]#'` or whitespace makes `arg.nwk` unreadable for IcyTree and other readers, and makes `nodes.dat` ambiguous
- TreeKnit.jl has the same defect. The port keeps it although its own Newick writer quotes labels

## Fix direction

- Quote ARG labels with the rule of `newick::quote`, and also quote `#`, which marks hybrid nodes in extended Newick
- Write `nodes.dat` with quoted fields, or document that labels with `,` are not supported there. `MCCs.dat` has the same limit, as in TreeKnit.jl

## Validation

- A test of `extended_newick` with a leaf `B,1` parses the output again and finds the leaf
- A test of `node_table` with the same leaf gives three fields per line
