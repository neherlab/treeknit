# treeknit-web

The TreeKnit web app: a React page that runs analyses in the browser on the WebAssembly build of the core (`packages/treeknit-wasm`). The trees never leave the browser.

```sh
just build-web dev    # unminified build with source maps, into dist/
just build-web prod   # build as shipped
just run-web dev      # Vite dev server with hot reload
just run-web prod     # build as shipped, then serve it
```

The commands are the same in the main checkout and in a worktree; `run-web` prints the URL on the port of the checkout. The developer guide (`docs/dev/developer_guide.md`, section "Web app") describes the modes, the ports, and the use of the build container.

## Structure

- `src/main.tsx`: the start-up: fonts, styles, and the root element
- `src/index.css`: the Tailwind CSS theme tokens: interface colors, type scale, control radius, and the MCC colors, which the app fills from the Rust palette; the dark values apply under the `dark` class on `html`
- `src/download.ts`: `downloadFile`, the one way the app saves a file: a Blob of the content under an object URL, revoked a minute after the click
- `src/analysis/example.ts`: the examples: a small pair of trees, the real H3N2 tree pairs of `data/`, and the simulated cases of `fixtures/sim/`, each tree as its file name and Newick text, one lazily loaded chunk per tree file
- `build/content-security-policy.ts`: the Content Security Policy of the page; `'wasm-unsafe-eval'` lets the page compile WebAssembly

## UI controls

`src/ui/` holds the controls of the app, one file per component, each built on React Aria Components so that focus, keyboard, overlay, and selection behavior come from the library:

- `Button`: variants `primary`, `secondary` (default), and `quiet`; sizes `md` (32 px), `sm` (28 px), and `xs` (20 px); an optional leading `icon`
- `IconButton`: an icon-only button; its required `label` is both its accessible name and its tooltip
- `TooltipTrigger`: wraps one focusable trigger and shows `tooltip` after 600 ms
- `InfoButton`: the info icon with the label and tooltip "About <topic>"; it opens a popover dialog with a short explanation. Every explanation in the app sits behind one, never as a paragraph in the workspace
- `TextField` (single line, or `multiline`; `mono` for Newick text), `NumberField` (with step buttons), `Select` (typed option ids), `RadioGroup` with `Radio`, `Switch`, and `ToggleButtonGroup` with `ToggleButton` (single selection by default; `iconOnly` buttons get a tooltip)
- `InlineNotice`: tones `info`, `warning`, and `danger`, each with its own icon, an optional title, and an optional action such as "Undo"; a danger notice is an alert
- `ProgressBar`: determinate, or a static hatched bar when indeterminate
- `CodeBlock`: Plex Mono text with a caption, a copy button that shows "Copied" for 2 s, line numbers for more than one line, and an optional marked range (1-based line and column, columns in Unicode code points, end exclusive) that it scrolls into view

Conventions for new controls:

- **Styles**: `src/ui/styles.ts` owns the shared classes: the focus ring, the button variants, the field input, label, description, and error, and the popover, list box, and option styles. Variants use `class-variance-authority`, and classes merge with `cn`. Controls use only the theme tokens of `src/index.css`, so both themes follow from the tokens
- **Focus**: a 2 px ring in the focus color with a 2 px offset, on keyboard focus only (`data-focus-visible`); text inputs also turn their border to the focus color on any focus
- **States**: hover, pressed, disabled, and invalid come from the React Aria data attributes (`data-hovered`, `data-pressed`, `data-disabled`, `data-invalid`); disabled text uses `ink-muted`
- **Color**: the primary button, a selected toggle, and an on switch are ink with ground text, so the interface stays quiet next to the colored drawings. Signal marks reassortment only and never appears in a control. Errors use danger with the `circle-alert` icon
- **Shape**: 4 px radius (`rounded-control`) on controls; radio indicators stay round
- **Fields**: a field takes `label`, optional `labelHidden`, `description`, `errorMessage`, and `info` (the text of its info button), through the shared parts in `src/ui/Field.tsx`
- **Icons**: Lucide through `unplugin-icons` (`import InfoIcon from "~icons/lucide/info"`), passed to controls as components (`icon={InfoIcon}`) and always `aria-hidden`
- **Motion**: color changes on hover and the thumb of a switch move in 150 ms and stop under `prefers-reduced-motion`; nothing moves on its own

## Data from Rust

The request and result types are Rust types in `packages/treeknit-wasm/src/analysis.rs`. tsify writes their TypeScript declarations into `packages/treeknit-wasm/pkg/treeknit_wasm.d.ts`, which the app imports from `@neherlab/treeknit-wasm`. Validation, default settings, MCCs per pair, and the output files with their command-line names all come from Rust; the app renders them and does not parse TreeKnit files. After changing the interface, run `just gen` and commit the declarations; `just generated-check` fails when they are stale.

## Tests

`just test-ts` runs the vitest tests in Node over in-memory values: the examples, the download helper, the Content Security Policy, the marked range of the code block, and its copy feedback timer. The analysis itself is tested in Rust (`just test-rs`, `just test-wasm`).
