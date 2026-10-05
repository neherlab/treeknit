# ARG construction fails when imputed trees disagree inside an MCC

For two trees with different leaf sets, `arg_inputs` builds the ARG from the imputed trees, restricted to the shared leaves and the leaves with an unambiguous attachment ([pipeline.rs#L630-L646](../../packages/treeknit-core/src/pipeline.rs#L639-L655)). Each tree gets the missing leaves of the other tree at the place that the other tree gives them. When both trees lack leaves near the same place, the two imputed trees can group the leaves differently inside one MCC. `arg_from_trees` then finds no consistent shared nodes and returns an error. The command line logs it and writes no ARG.

## Reproduction

```sh
printf '((A:1,B:1):1,(C:1,(D:1,X:1):1):1);\n' > ha.nwk   # lacks P
printf '((A,B),(C,P));\n' > part.nwk                   # lacks D and X
treeknit ha.nwk part.nwk -o out --impute
# ERROR ARG construction failed: shared node IMPUTED_2 has different parents; no ARG written
```

All six leaves form one MCC, with no ambiguous attachment. The imputed `part` tree has the clade `(C,(D,X))`, and the imputed `ha` tree has `(C,P)`. Both claim to share all branches, but their topologies differ.

## Impact

- No ARG for inputs where each tree lacks leaves near the same place
- `MCCs.json` reports one MCC for trees that, after imputation, have different topologies inside it, so the MCC claim and the imputed trees contradict each other

## Fix direction

> [!IMPORTANT]
> **Decision required.** Options: build the ARG only from the shared leaves (the MCCs restricted to them), so imputation never enters the ARG; impute both trees from one common placement, so that the two imputed trees agree inside each MCC; or mark such attachments ambiguous and leave their leaves out of the ARG, as for other ambiguous attachments.

## Validation

- A command-line test with the trees above writes `ARG/arg.nwk`, and the segment trees of the ARG, restricted to the shared leaves, have the splits of the input trees
