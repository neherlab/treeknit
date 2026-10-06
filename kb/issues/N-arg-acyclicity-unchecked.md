# ARG construction does not check for cycles

An ARG is a valid genealogy only if it has no directed cycle, so that its nodes can be ordered in time. The MCCs that inference returns form an acyclic agreement forest by an argument from their construction ([`agreement-forest-validation.md`](../proposals/agreement-forest-validation.md)), but topology matching, imputation, and the liberal resolution inside ARG construction change MCCs or trees afterwards, and no code checks the result.

## Locations

- `pub fn arg_from_trees()` [packages/treeknit-core/src/arg.rs#L141-L170](../../packages/treeknit-core/src/arg.rs#L141-L170): builds the ARG and checks shared nodes, not cycles
- `fn topological_order()` [packages/treeknit-io/src/display/arg_view.rs#L141-L160](../../packages/treeknit-io/src/display/arg_view.rs#L141-L160): Kahn's algorithm for the ARG drawing; a node on a cycle never enters the order, and the caller [packages/treeknit-io/src/display/arg_view.rs#L22-L29](../../packages/treeknit-io/src/display/arg_view.rs#L22-L29) does not check its length

> [!IMPORTANT]
> **Investigation required.** No cyclic ARG has been observed. A property test over random tree pairs with every resolution mode, with and without leaves missing from one tree, would show whether the later steps can produce one.

## Fix direction

- After construction, order the ARG nodes with Kahn's algorithm in the core and return an `ArgError` when fewer nodes are ordered than the ARG has
- Add the property test of the investigation above

## Validation

- A unit test builds an ARG with a cycle by hand and gets the error
- The property test passes on its random pairs
