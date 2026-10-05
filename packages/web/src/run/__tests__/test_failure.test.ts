import type { Settings } from "@neherlab/treeknit-wasm";
import { describe, expect, test } from "vitest";

import { CANCELLED_MESSAGE, RESULTS_LOST_MESSAGE } from "../../analysis/client";
import {
  bugReportUrl,
  failureNotice,
  FIX_AND_RUN_AGAIN,
  INTERNAL_ERROR,
  INVALID_REQUEST,
  START_FAILED,
  USE_A_CURRENT_BROWSER,
} from "../failure";

const SETTINGS: Settings = { gamma: 2, seqLengths: [1700, 1400], resolve: "matched", seed: 42 };

const LEAF = "A/Hong Kong/4801/2014";

describe("run failures", () => {
  test("states each failure kind with what to do", () => {
    expect({
      start: failureNotice("start", "no WebAssembly"),
      internal: failureNotice("internal", "unreachable executed"),
      invalid: failureNotice("invalid", "label is empty"),
      cancelled: failureNotice("cancelled", CANCELLED_MESSAGE),
    }).toStrictEqual({
      start: {
        tone: "danger",
        title: START_FAILED,
        details: ["no WebAssembly", USE_A_CURRENT_BROWSER],
        canRunAgain: false,
        canReport: false,
      },
      internal: {
        tone: "danger",
        title: INTERNAL_ERROR,
        details: ["unreachable executed"],
        canRunAgain: true,
        canReport: true,
      },
      invalid: {
        tone: "danger",
        title: INVALID_REQUEST,
        details: ["label is empty", FIX_AND_RUN_AGAIN],
        canRunAgain: false,
        canReport: false,
      },
      cancelled: { tone: "info", title: CANCELLED_MESSAGE, details: [], canRunAgain: false, canReport: false },
    });
  });

  test("says that the results were lost instead of that the analysis stopped", () => {
    expect(failureNotice("internal", RESULTS_LOST_MESSAGE)).toStrictEqual({
      tone: "danger",
      title: RESULTS_LOST_MESSAGE,
      details: [],
      canRunAgain: true,
      canReport: true,
    });
  });

  test("prefills a new issue with the kind, version, browser, and settings", () => {
    const url = new URL(
      bugReportUrl("https://github.com/neherlab/treeknit-rs/", {
        kind: "internal",
        version: "0.5.0-dev",
        userAgent: "Mozilla/5.0 Firefox/140.0",
        settings: SETTINGS,
      }),
    );

    const body = url.searchParams.get("body") ?? "";

    expect({
      page: `${url.origin}${url.pathname}`,
      lines: [
        "Failure: internal",
        "Version: 0.5.0-dev",
        "Browser: Mozilla/5.0 Firefox/140.0",
        '  "resolve": "matched",',
      ].map((line) => body.includes(line)),
      settings: body.includes(JSON.stringify(SETTINGS, null, 2)),
    }).toStrictEqual({
      page: "https://github.com/neherlab/treeknit-rs/issues/new",
      lines: [true, true, true, true],
      settings: true,
    });
  });

  test("never puts the error message or a leaf name into the report", () => {
    const message = `tree "ha": duplicate leaf name ${LEAF} at line 1`;
    const notice = failureNotice("internal", message);

    const url = bugReportUrl("https://github.com/neherlab/treeknit-rs", {
      kind: "internal",
      version: "0.5.0",
      userAgent: "Mozilla/5.0",
      settings: SETTINGS,
    });

    const decoded = decodeURIComponent(url.replaceAll("+", " "));

    expect({
      shown: notice.details,
      leafInUrl: decoded.includes(LEAF) || decoded.includes("Hong Kong"),
      messageInUrl: decoded.includes("duplicate leaf name"),
      newickInUrl: decoded.includes("("),
    }).toStrictEqual({ shown: [message], leafInUrl: false, messageInUrl: false, newickInUrl: false });
  });
});
