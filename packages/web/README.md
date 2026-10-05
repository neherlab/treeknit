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

- `src/main.tsx`: the start-up: fonts, styles, the theme provider (`next-themes`, following the system theme until the user switches, with the settings `THEME_PROVIDER_PROPS` of `src/shell/theme.ts`), and the router
- `src/router.tsx`: the routes `/` (workspace) and `/help`, on hash history so that the app works on any static host without rewrites; search params are plain `key=value` text, and default values stay out of the URL
- `src/workspace/search.ts`: the workspace search params (`view`, `pair`, `version`, `x`, `labels`, `mcc`, `leaf`, `node`) as a zod schema whose invalid or missing values fall back to the defaults; `resolveWorkspaceSearch` replaces values that do not fit the workspace (an unavailable view shows the overview, a pair out of range becomes 0, an `mcc`, `leaf`, or `node` absent from the pair is dropped) from a `WorkspaceAvailability`; `selectPair` clears `mcc` and `node`, because they mean nothing in another pair. A node is written as its side and name, `left:NODE_3`
- `src/workspace/useWorkspaceSearch.ts`: the resolved search params of the workspace and their update; the URL keeps the values as written, so a shared link still applies once the result exists
- `src/shell/`: the application frame: the header (wordmark, "Workspace" and "Help", theme toggle) and the workspace grid of a 336 px rail, the center with the view tabs, and a 320 px inspector. Below 1024 px the rail opens as a side sheet from "Trees and settings" and the run bar stays at the bottom of the screen; below 1320 px the inspector opens as a side sheet from "Details" (`layout.ts`). `Rail`, `RunBar`, `CenterViews` (with one panel component per view), `EmptyCenter`, and `Inspector` are the slots that the workspace features fill
- `src/help/HelpPage.tsx`: the help page
- `src/index.css`: the Tailwind CSS theme tokens: interface colors, type scale, control radius, and the MCC colors, which the app fills from the Rust palette; the dark values apply under the `dark` class on `html`
- `src/download.ts`: `downloadFile`, the one way the app saves a file: a Blob of the content under an object URL, revoked a minute after the click
- `src/analysis/example.ts`: the examples: a small pair of trees, the real H3N2 tree pairs of `data/`, and the simulated cases of `fixtures/sim/`, each tree as its file name and Newick text, one lazily loaded chunk per tree file
- `build/content-security-policy.ts`: the Content Security Policy of the page, and the theme script in the head of `index.html`. `'wasm-unsafe-eval'` lets the page compile WebAssembly. The theme script is the script that `next-themes` renders for `THEME_PROVIDER_PROPS`; it sets the theme class before the first paint, because React does not run a script element that it creates on the client. The built page allows that one inline script by its SHA-256 hash

## UI controls

`src/ui/` holds the controls of the app, one file per component, each built on React Aria Components so that focus, keyboard, overlay, and selection behavior come from the library:

- `Button`: variants `primary`, `secondary` (default), and `quiet`; sizes `md` (32 px), `sm` (28 px), and `xs` (20 px); an optional leading `icon`
- `IconButton`: an icon-only button; its required `label` is both its accessible name and its tooltip
- `TooltipTrigger`: wraps one focusable trigger and shows `tooltip` after 600 ms. With `repeatsName`, the tooltip text equals the accessible name of the trigger and stays out of its description, so that screen readers say it once; `IconButton`, icon-only toggle buttons, and the step buttons of `NumberField` set it
- `InfoButton`: the info icon with the label and tooltip "About <topic>"; it opens a popover dialog with a short explanation
- `TextField` (single line, or `multiline`; `mono` for Newick text, with spell checking, autocorrect, and autocapitalize off), `NumberField` (with step buttons that carry the localized labels of React Aria), `Select` (typed option ids), `RadioGroup` with `Radio`, `Switch`, and `ToggleButtonGroup` with `ToggleButton` (single selection by default; `iconOnly` buttons get a tooltip)
- `InlineNotice`: tones `info`, `warning`, and `danger`, each with its own icon, an optional title, and an optional action such as "Undo"; a danger notice is an alert, which screen readers announce when it appears. `NoticeRegion` is a live region for info and warning notices: it stays mounted while notices come and go inside it, because screen readers usually do not announce a live region that mounts together with its content
- `ProgressBar`: determinate, or a static hatched bar when indeterminate; `labelHidden` hides the label and keeps the value text
- `Tabs` with `TabList`, `Tab`, and `TabPanel`: an underlined tab row on a rule
- `Menu` (with its `trigger`), `MenuItem` (optional `icon`, `tone="danger"`), `MenuSection`, and `MenuSeparator`
- `Dialog`: a modal with a title, a close button, and an optional footer; `placement` `center` (default), or `left` and `right` for a side sheet. Open it from a `DialogTrigger` or with `isOpen` and `onOpenChange`
- `Disclosure`: a section title that expands its content, with an optional info button
- `GridList` with `GridListItem`: rows separated by rules; with `onReorder`, each row has a drag handle, and rows move by pointer or keyboard. `reorder` (`src/ui/reorder.ts`) applies the move to a list
- `Table` with `TableHeader`, `Column`, `TableBody`, `Row`, and `Cell`: a small table in Plex Sans Condensed with a sticky header, sort indicators, and `align="end"` for numbers
- `Link`: a router link (TanStack Router `createLink` over the React Aria link), so links take `to` and `search`
- `EmptyState`: a quiet, left-aligned message with an optional icon, details, and actions
- `CodeBlock`: Plex Mono text in a scrolling region named by its caption, a copy button that shows "Copied" or "Copy failed" for 2 s (the latest copy wins, and the cause of a failure goes to the console), line numbers for more than one line, and an optional `errorRange` in the danger colors (1-based line and column, columns in Unicode code points, end exclusive), which the region scrolls into view without scrolling the page. A line end at the end of the text adds no empty line, unless the range is on that line. `codeLines` follows `newick::line_column` of `treeknit-io`, and both test suites check the same cases

Conventions for new controls:

- **Styles**: `src/ui/styles.ts` owns the shared classes: the focus ring, the button variants, the field input, label, description, and error, and the popover, list box, and option styles. Variants use `class-variance-authority`, and classes merge with `cn`. Controls use only the theme tokens of `src/index.css`, so both themes follow from the tokens
- **Focus**: a 2 px ring in the focus color with a 2 px offset, on keyboard focus only (`data-focus-visible`, or `:focus-visible` through `nativeFocusRing` for an element without React Aria); text inputs also turn their border to the focus color on any focus
- **States**: hover, pressed, disabled, and invalid come from the React Aria data attributes (`data-hovered`, `data-pressed`, `data-disabled`, `data-invalid`); disabled text uses `ink-muted`
- **Color**: the primary button, a selected toggle, and an on switch are ink with ground text, so the interface stays quiet next to the colored drawings. Signal marks reassortment only and never appears in a control. Errors use danger with the `circle-alert` icon
- **Shape**: 4 px radius (`rounded-control`) on controls, and 2 px (`rounded-inner`) on parts inside a control, such as the thumb of a switch; radio indicators stay round
- **Fields**: a field takes `label`, optional `labelHidden`, `description`, `errorMessage`, and `info`, through the shared parts in `src/ui/Field.tsx`. An `errorMessage` alone marks the field invalid. A field with `labelHidden` takes no `info`, because the info button would have no visible topic
- **Description and info**: `description` is a few words under the field that the user needs while filling it, such as a unit or an input format. Every explanation, such as what a setting does, why it does not apply, or how to read a table, goes into `info`, the popover of the info button, and never into a paragraph in the workspace
- **Icons**: Lucide through `unplugin-icons` (`import InfoIcon from "~icons/lucide/info"`), passed to controls as components (`icon={InfoIcon}`) and always `aria-hidden`
- **Motion**: color changes on hover and the thumb of a switch move in 150 ms and stop under `prefers-reduced-motion`; nothing moves on its own

## Tree canvas

`src/canvas/` holds the drawing engine that every tree drawing uses: one deck.gl canvas whose horizontal axis is fitted to the width and whose leaf axis zooms, as in Auspice. It draws the shapes that Rust lays out (branch elbows, link and ribbon curves, marks) and builds no shape of its own; it only maps their normalized units to the canvas.

- **Coordinates**: world x is in canvas pixels and world y in leaf rows, row `i` at `y = i`, with half a row of margin at both ends. On the narrow layout (`leafAxis` `x`) the two axes swap
- **View state** (`viewState.ts`): pure functions over a `CanvasFrame` (canvas size, row count, leaf axis). The view zooms the leaf axis only, with deck.gl's `zoomX`, `zoomY`, and `zoomAxis`; the cross axis keeps zoom 0 and its center, so a drawing never moves sideways. The leaf zoom stays between the fitted view and 64 px per row, and a zoom that changes the canvas by less than one pixel from either limit snaps to that limit, so rounding in repeated zoom steps never leaves a zoom button enabled without effect. The view stays inside the drawing. `fitRowsViewState` frames a row range (the clade zoom)
- **`useTreeView(rows, leafAxis)`**: the view of one drawing, with a pure reducer (`treeViewReducer`) behind stable `actions` (resize, controller update, zoom in, zoom out, fit, fit rows, pan to a row, pan by a row offset). A new row count or leaf axis fits the drawing again; a resize keeps the visible rows
- **`TreeCanvas`**: the `DeckGL` element with an `OrthographicView`. Wheel and pinch zoom around the pointer, drag pans, and the arrow keys and `+`/`-` pan and zoom while the drawing has focus. A double-click asks `onCladeZoom` for the row range of the picked branch and frames it. The container is focusable, has `role="img"`, the drawing's label, and a description that points to the tables. Without WebGL 2 it shows a notice instead
- **`Minimap`**: for more than 200 leaves, a second canvas in the lower right, 160 px across the cross axis and 200 px along the leaf axis (`minimapSize`), shows the whole drawing and the visible rows; pressing or dragging on it moves the main view. deck.gl sets the canvas to that size and the frame is an outline, so the drawn area keeps the full size; the projection and the pointer mapping use the size deck.gl measures. It takes its own layer instances, because a deck.gl layer belongs to one canvas
- **`ZoomControls`**: the "Zoom in", "Zoom out", and "Fit to view" buttons for a drawing toolbar
- **Projection** (`projection.ts`): a `Column` maps normalized x in 0 to 1 to canvas pixels, mirrored for a tree drawn with its root at the right; the `Point` and `Bezier` shapes are the generated Rust types. A cubic Bézier curve becomes a path by uniform parameter subdivision with exact endpoints; `wangSegmentCount` takes the number of segments from Wang's bound, $n = \lceil \sqrt{\tfrac{3}{4} M / \varepsilon} \rceil$, where $M$ is the largest second difference of the control points in pixels at 64 px per row and $\varepsilon$ = 0.5 px is the largest distance between path and curve, limited to 1 to 256 segments
- **Layer builders** (`layers/`): `labelLayer` and `pathLayer` turn data and accessors into deck.gl layers. Geometry depends only on the data, the options, and the canvas size; selection and hover change colors through `colorTriggers`, so a hover does not rebuild the paths of 20,000 leaves. Dashed paths use the deck.gl `PathStyleExtension` with dashes in pixels
- **Colors** (`drawingColors.ts`): `useDrawingColors` reads the drawing tokens of `src/index.css` (ground, ink, ink-muted, signal, focus, segments, MCC slots) as RGBA bytes and reads them again when the theme class or the inline MCC colors on `html` change
- **Labels** (`labels.ts`): 12 px IBM Plex Sans Condensed. The `auto` mode shows labels from 10 px per row, `on` and `off` always and never; a label longer than 40 characters is shortened in the middle. The text layer waits for the font (`useLabelFontReady`) and builds its glyphs from the data (`characterSet: "auto"`), because strain names hold letters outside ASCII
- **Motion** (`motion.ts`): `useFadeIn` gives the 300 ms opacity ramp of ribbons and links when a new result arrives, and full opacity under `prefers-reduced-motion`
- **Loading**: deck.gl is large, so a drawing view is a separate chunk: load the module of a drawing tab with `React.lazy` inside `CanvasBoundary`, which shows a progress bar while the chunk loads and an error notice if the drawing fails. Modules outside a lazy drawing import only the deck.gl-free parts of `src/canvas/` (view state, colors, labels, motion, zoom controls)

## Data from Rust

The request and result types are Rust types in `packages/treeknit-wasm/src/analysis.rs`. tsify writes their TypeScript declarations into `packages/treeknit-wasm/pkg/treeknit_wasm.d.ts`, which the app imports from `@neherlab/treeknit-wasm`. Validation, default settings, MCCs per pair, and the output files with their command-line names all come from Rust; the app renders them and does not parse TreeKnit files. After changing the interface, run `just gen` and commit the declarations; `just generated-check` fails when they are stale.

## Tests

`just test-ts` runs the vitest tests in Node over in-memory values: the examples, the download helper, the Content Security Policy with the theme script, the marked range of the code block and its scroll offset, its copy feedback, the reorder of list items, the workspace search params with their defaults and fallbacks, the view tabs, the breakpoints of the rail and the inspector, the theme toggle labels, and the canvas engine (view state against the deck.gl viewport projection, the view reducer, the minimap, column projection and Bézier sampling, color conversion, the label rule and shortening, the fade-in, WebGL 2 detection, and the layer builders). The analysis itself is tested in Rust (`just test-rs`, `just test-wasm`).
