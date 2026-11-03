import { createStateFromQueryOrJSONs } from "auspice/src/actions/recomputeReduxState";
import { CLEAN_START } from "auspice/src/actions/types";
import { afterEach, beforeEach, describe, expect, test, vi } from "vitest";

import { AUSPICE_I18N } from "../i18n";
import { createAuspiceStore } from "../store";
import { auspiceSummary } from "../summary";

const GENOME_ANNOTATIONS = {
  nuc: { start: 1, end: 18, strand: "+" },
  S: { start: 1, end: 9, strand: "+" },
  E: { start: 10, end: 18, strand: "+" },
};

describe("auspice branch length measures", () => {
  beforeEach(() => {
    vi.stubGlobal("window", { innerWidth: 1280, innerHeight: 800, document: { body: { clientHeight: 800 } } });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  test.each<{ name: string; dataset: Dataset; expected: Measures }>([
    {
      name: "an undated tree offers divergence only",
      dataset: dataset({ dates: "none" }),
      expected: { availableDistanceMeasures: ["div"], distanceMeasure: "div" },
    },
    {
      name: "a dated tree offers time first, and shows it",
      dataset: dataset({ dates: "all" }),
      expected: { availableDistanceMeasures: ["num_date", "div"], distanceMeasure: "num_date" },
    },
    {
      name: "a tree with an undated node offers no time, as that node has no place on a time axis",
      dataset: dataset({ dates: "partly" }),
      expected: { availableDistanceMeasures: ["div"], distanceMeasure: "div" },
    },
    {
      name: "an annotated tree offers the mutation counts of the genome and of each CDS",
      dataset: dataset({ dates: "none", mutations: { nuc: ["A5G"], S: ["K2R"], E: ["L1F"] } }),
      expected: {
        availableDistanceMeasures: ["div", "mutationCount:nuc", "mutationCount:S", "mutationCount:E"],
        distanceMeasure: "div",
      },
    },
    {
      name: "an annotated tree offers no mutation count for a CDS whose mutations are all invalid (X)",
      dataset: dataset({ dates: "none", mutations: { nuc: ["A5G"], S: ["K2R"], E: ["L1X"] } }),
      expected: {
        availableDistanceMeasures: ["div", "mutationCount:nuc", "mutationCount:S"],
        distanceMeasure: "div",
      },
    },
  ])("$name", ({ dataset: json, expected }) => {
    const { controls } = loadedState({ json });

    expect(measures(controls)).toStrictEqual(expected);
  });

  test("an undated tree loads without an error about the default time measure", () => {
    const errors = consoleErrors();

    loadedState({ json: dataset({ dates: "none" }) });

    expect(errors).toStrictEqual([]);
  });

  test("an undated second tree shows both trees by divergence, without an error", () => {
    const errors = consoleErrors();

    const { controls } = loadedState({
      json: dataset({ dates: "all" }),
      secondTreeDataset: dataset({ dates: "none" }),
    });

    expect({ measures: measures(controls), errors }).toStrictEqual({
      measures: { availableDistanceMeasures: ["div"], distanceMeasure: "div" },
      errors: [],
    });
  });
});

describe("auspiceSummary", () => {
  beforeEach(() => {
    vi.stubGlobal("window", { innerWidth: 1280, innerHeight: 800, document: { body: { clientHeight: 800 } } });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  test.each<{ name: string; dates: Dates; expected: string }>([
    { name: "counts the shown leaves of an undated tree", dates: "none", expected: "Showing 3 of 3 leaves." },
    {
      name: "counts the shown leaves of a dated tree and names the months of their dates",
      dates: "all",
      expected: "Showing 3 of 3 leaves sampled between Jan 2020 and Mar 2020.",
    },
  ])("$name", ({ dates, expected }) => {
    const state = loadedState({ json: dataset({ dates }) });

    expect(auspiceSummary(state, AUSPICE_I18N.getFixedT("en"))).toBe(expected);
  });
});

function loadedState({ json, secondTreeDataset }: { json: Dataset; secondTreeDataset?: Dataset }) {
  const store = createAuspiceStore();

  const state = createStateFromQueryOrJSONs({
    json,
    secondTreeDataset,
    mainTreeName: "a",
    secondTreeName: "b",
    query: {},
    dispatch: store.dispatch,
  });

  store.dispatch({ ...state, type: CLEAN_START });

  return store.getState();
}

function consoleErrors(): unknown[][] {
  const errors: unknown[][] = [];

  vi.stubGlobal("console", {
    ...console,
    error: (...args: unknown[]) => {
      errors.push(args);
    },
  });

  return errors;
}

function measures({ availableDistanceMeasures, distanceMeasure }: Measures): Measures {
  return { availableDistanceMeasures, distanceMeasure };
}

interface Measures {
  availableDistanceMeasures: readonly string[];
  distanceMeasure: string;
}

function dataset({
  dates,
  mutations,
}: {
  dates: Dates;
  mutations?: Readonly<Record<string, readonly string[]>>;
}): Dataset {
  const node = (name: string, div: number, numDate: number, children?: readonly DatasetNode[]): DatasetNode => {
    const nodeAttrs: DatasetNodeAttrs = { div, clade: { value: "c1" } };

    if (dates === "all" || (dates === "partly" && name !== "C")) {
      nodeAttrs.num_date = { value: numDate };
    }

    return {
      name,
      node_attrs: nodeAttrs,
      branch_attrs: name === "B" && mutations !== undefined ? { mutations } : {},
      children,
    };
  };

  const meta: DatasetMeta = {
    title: "measures",
    panels: ["tree"],
    colorings: [{ key: "clade", title: "Clade", type: "categorical" }],
    display_defaults: { color_by: "clade" },
  };

  if (mutations !== undefined) {
    meta.genome_annotations = GENOME_ANNOTATIONS;
  }

  return {
    version: "v2",
    meta,
    tree: node("ROOT", 0, 2019.9, [
      node("A", 1, 2020.05),
      node("N", 1, 2020, [node("B", 2, 2020.1), node("C", 2, 2020.2)]),
    ]),
  };
}

type Dates = "none" | "all" | "partly";

interface Dataset {
  version: string;
  meta: DatasetMeta;
  tree: DatasetNode;
}

interface DatasetMeta {
  title: string;
  panels: readonly string[];
  colorings: readonly { key: string; title: string; type: string }[];
  display_defaults: { color_by: string };
  genome_annotations?: typeof GENOME_ANNOTATIONS;
}

interface DatasetNode {
  name: string;
  node_attrs: DatasetNodeAttrs;
  branch_attrs: Readonly<Record<string, unknown>>;
  children: readonly DatasetNode[] | undefined;
}

interface DatasetNodeAttrs {
  div: number;
  clade: { value: string };
  num_date?: { value: number };
}
