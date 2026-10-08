# The Auspice view of the web app fails on deep trees

The Auspice view of the web app cannot show a pair whose trees are deeper than about 1,100 levels. The datasets nest one object per tree level, and two steps between Rust and Auspice handle the nesting by recursion: the structured clone of comlink, which carries the datasets from the worker to the page, and the Rust code that builds and converts them. The rest of a run, the Auspice JSON files included, writes trees of any depth without recursion. The design options for deep trees are in [`deep-trees.md`](../proposals/deep-trees.md).

## Locations

- **Structured clone**: `Session.auspiceView` returns the datasets from the worker through comlink, which sends them with `postMessage`. Node 24.21 (V8) clones an object of the shape of an Auspice node (`name`, `node_attrs`, `children`) at a nesting depth of 1,135 tree levels and throws "Maximum call stack size exceeded" at 1,136; `JSON.parse` of the same nesting succeeds at 200,000. Measured on 2026-10-08 with `structuredClone` in the project container; the limit of a browser worker was not measured
- **Rust datasets**: `pub fn auspice_view()` [packages/treeknit-io/src/display/auspice.rs#L52](../../packages/treeknit-io/src/display/auspice.rs#L52) builds the nesting bottom-up, but `struct AuspiceNode` holds its children, so its derived `Serialize`, `Clone`, and `PartialEq`, its drop, and the finiteness check of `fn to_js` [packages/treeknit-wasm/src/lib.rs#L594](../../packages/treeknit-wasm/src/lib.rs#L594) recurse once per level. In the WebAssembly tests in Node, `auspice_view` with its JSON conversion runs a caterpillar of depth 5,000 and fails at 10,000 with "Maximum call stack size exceeded"
- **Auspice**: Auspice 3.0.0 processes the tree in the page; whether it recurses per level was not checked

The depth of real trees is far below these limits: the deepest tree in `data/` has a nesting depth of 136 (`data/h5n1-587x3-timetree/tree_pb1.nwk`, 587 leaves).

## Impact

- The view shows the failure of the query instead of the trees. The client keeps the session worker after this error; whether the worker still answers afterwards was not tested, because the JavaScript exception leaves the WebAssembly frames without running Rust destructors
- The "JSON" download of the view (`auspiceFiles`) converts the same datasets

## Fix direction

- Carry the datasets from the worker as JSON text and parse them in the page, because `JSON.parse` does not recurse per level; or send them flat, with child indices, and nest them in the page with an explicit stack
- Write the dataset text in Rust with an explicit stack, as `auspice_json` [packages/treeknit-io/src/auspice.rs#L18](../../packages/treeknit-io/src/auspice.rs#L18) does for the files, instead of deriving `Serialize` for a nested type
- Check how Auspice handles a deep tree before choosing, because a fix before Auspice helps only if Auspice itself draws such a tree

## Validation

- The Auspice view of a pair of caterpillar trees of depth 10,000 shows both trees
- A WebAssembly test converts the datasets of `auspice_view` for a caterpillar of depth 12,000
