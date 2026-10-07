# ARG files can give a leaf and an internal node the same label

The ARG builder names internal nodes `ARGNode_<i>` ([arg.rs#L379](../../packages/treeknit-core/src/arg.rs#L379)) and leaves by their taxon names ([arg.rs#L396-L401](../../packages/treeknit-core/src/arg.rs#L396-L401)), without checking that the two sets of names are disjoint. When no segment root is shared, the extended Newick writer also adds a top root with the fixed label `GlobalRoot` ([arg.rs#L24-L28](../../packages/treeknit-io/src/arg.rs#L25-L33)). A leaf named `ARGNode_1` or `GlobalRoot` therefore gets the same label as another node of the ARG.

## Reproduction

```sh
printf "(GlobalRoot,((C,ARGNode_1),D));\n" > ha.nwk
printf "((D,(GlobalRoot,C)),ARGNode_1);\n" > na.nwk
treeknit ha.nwk na.nwk -o out
```

`out/ARG/arg.nwk` holds the leaf `ARGNode_1` and also uses `ARGNode_1` as the label of an internal node:

```text
((ARGNode_1[&segments={0,1}])ARGNode_7#H1[&segments={1}],(...)ARGNode_1[&segments={1}])ARGNode_11[&segments={1}];
```

`out/ARG/nodes.dat` has two lines that start with `ARGNode_1`. The collision with `GlobalRoot` follows from the writer code and needs input where no segment root is shared.

## Impact

- A reader of `arg.nwk` or `nodes.dat` that identifies nodes by label cannot tell the leaf from the internal node
- The ARG view of the web app is not affected, because `treeknit_io::display::arg_view` gives internal nodes unique labels (for example `ARGNode_1_2`) and keeps the leaf names. Its labels therefore differ from `arg.nwk` for these inputs

## Fix direction

- Choose internal ARG labels and the synthetic root label that no leaf has, for example by numbering them above every index that a node of the tree already has, as `Tree::fresh_index` does for the nodes that resolution and imputation add. Then the display can use the labels of the ARG as they are
- TreeKnit.jl names internal ARG nodes `ARGNode_<random string>` (`make_random_label` in `src/SimpleReassortmentGraph/misc.jl`), so a collision there needs a leaf with the same random name; it writes the same fixed `GlobalRoot` label (`src/SimpleReassortmentGraph/IO.jl`) and shares that collision

## Validation

- A test of `arg_from_trees` with a leaf named `ARGNode_1` finds pairwise distinct labels
- A test of `extended_newick` in the case with no shared root, with a leaf named `GlobalRoot`, finds one node with that label
