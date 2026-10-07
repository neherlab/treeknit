//! The drawing rules and tables of their columns and row thresholds at chosen sizes, as JSON.
//!
//! The web app applies the column and threshold formulas of `DrawingRules` itself, because it
//! needs them on every zoom frame and resize; its tests check that copy against this table.
//! `just gen` writes it to `packages/web/src/drawing/__tests__/__fixtures__/drawing_cases.json`,
//! and `just generated-check` fails when the committed file differs from a fresh run.
//!
//! The inputs sit at the boundaries of the rules: drawings narrower than both margins, the row
//! heights where labels (10 px per row) and links (6 px per row) switch on, label widths below
//! and above the 15% floor of the link zone, and label widths beyond the caps of both label
//! columns.

use serde::Serialize;
use std::process::ExitCode;
use treeknit_io::display::{ColumnPx, DRAWING_RULES, DrawingRules, TanglegramColumns};

/// Drawing widths in px: narrower than both margins, at both margins, and wide.
const WIDTHS_PX: [f64; 4] = [20.0, 32.0, 800.0, 1232.0];

/// Longest label widths in px: none, short, at the floor of the link zone, wide, and capped.
const LONGEST_LABELS_PX: [f64; 5] = [0.0, 8.0, 18.0, 88.0, 500.0];

/// Row heights in px around the thresholds of the links and the labels.
const ROWS_PX: [f64; 4] = [5.5, 6.0, 9.5, 10.0];

fn main() -> ExitCode {
  let columns = WIDTHS_PX
    .into_iter()
    .flat_map(|width| LONGEST_LABELS_PX.map(|label| columns(width, label)))
    .collect();
  let rows = ROWS_PX.into_iter().map(rows).collect();
  match serde_json::to_string_pretty(&Table {
    rules: DRAWING_RULES,
    columns,
    rows,
  }) {
    Ok(json) => {
      println!("{json}");
      ExitCode::SUCCESS
    },
    Err(e) => {
      eprintln!("drawing_cases: {e}");
      ExitCode::FAILURE
    },
  }
}

fn columns(width_px: f64, longest_label_px: f64) -> ColumnsCase {
  let rules = DRAWING_RULES;
  let tanglegram_label = rules.label_column_px(longest_label_px, rules.tanglegram_label_max_px(width_px));
  let arg_label = rules.label_column_px(longest_label_px, rules.arg_label_max_px(width_px));
  ColumnsCase {
    width_px,
    longest_label_px,
    tanglegram: rules.tanglegram_columns(width_px, tanglegram_label),
    arg_column: rules.arg_column(width_px, arg_label),
  }
}

fn rows(row_px: f64) -> RowsCase {
  RowsCase {
    row_px,
    labels_shown: DRAWING_RULES.auto_labels_shown(row_px),
    ribbons_shown: DRAWING_RULES.ribbons_shown(row_px),
  }
}

#[derive(Serialize)]
struct Table {
  rules: DrawingRules,
  columns: Vec<ColumnsCase>,
  rows: Vec<RowsCase>,
}

/// The columns of a drawing `width_px` wide whose longest label is `longest_label_px` wide.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ColumnsCase {
  width_px: f64,
  longest_label_px: f64,
  tanglegram: TanglegramColumns,
  arg_column: ColumnPx,
}

/// The row thresholds at `row_px` px per row; `labels_shown` is for the label mode `auto`.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RowsCase {
  row_px: f64,
  labels_shown: bool,
  ribbons_shown: bool,
}
