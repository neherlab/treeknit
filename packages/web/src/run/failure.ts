import type { Settings } from "@neherlab/treeknit-wasm";
import { match } from "ts-pattern";

import { CANCELLED_MESSAGE, type FailureKind, RESULTS_LOST_MESSAGE } from "../analysis/client";

export const START_FAILED = "TreeKnit could not start in this browser.";

export const USE_A_CURRENT_BROWSER = "Use a current version of Chrome, Edge, Firefox, or Safari.";

export const INTERNAL_ERROR = "The analysis stopped because of an internal error.";

export const INVALID_REQUEST = "TreeKnit could not run these trees and settings.";

export const FIX_AND_RUN_AGAIN = "Fix the errors above and run again.";

const REPORT_TITLE = "Internal error in the web app";

export interface FailureNotice {
  tone: "info" | "danger";
  title: string;
  details: readonly string[];
  canRunAgain: boolean;
  canReport: boolean;
}

export interface BugReport {
  kind: FailureKind;
  version: string;
  userAgent: string;
  settings: Settings;
}

export function failureNotice(kind: FailureKind, message: string): FailureNotice {
  return match(kind)
    .with("start", () => danger(START_FAILED, [message, USE_A_CURRENT_BROWSER], false))
    .with("internal", () =>
      message === RESULTS_LOST_MESSAGE
        ? danger(RESULTS_LOST_MESSAGE, [], true)
        : danger(INTERNAL_ERROR, [message], true),
    )
    .with("invalid", () => danger(INVALID_REQUEST, [message, FIX_AND_RUN_AGAIN], false))
    .with("cancelled", (): FailureNotice => ({
      tone: "info",
      title: CANCELLED_MESSAGE,
      details: [],
      canRunAgain: false,
      canReport: false,
    }))
    .exhaustive();
}

export function bugReportUrl(newIssue: string, report: BugReport): string {
  const body = [
    `Failure: ${report.kind}`,
    `Version: ${report.version}`,
    `Browser: ${report.userAgent}`,
    "",
    "Settings:",
    "",
    "```json",
    JSON.stringify(report.settings, null, 2),
    "```",
    "",
    "What did you do before the error?",
    "",
  ].join("\n");

  const query = new URLSearchParams({ title: REPORT_TITLE, body });

  return `${newIssue}?${query.toString()}`;
}

function danger(title: string, details: readonly string[], internal: boolean): FailureNotice {
  return { tone: "danger", title, details, canRunAgain: internal, canReport: internal };
}
