//! Write the TypeScript constants of the Rust values that the web app needs before a worker
//! answers, to stdout: the value list and the default value of each Rust enum whose values a
//! control of the web app offers, the labels of the legend entries, which the tooltips and the
//! inspector share, and the tree counts of a run. `just gen` writes them to
//! `pkg/treeknit_variants.ts`, next to the declarations of the module.

use serde::Serialize;
use std::any::type_name;
use strum::VariantArray;
use treeknit_io::analysis::{ARG_TREE_COUNT, MIN_TREES};
use treeknit_io::display::{LegendKind, Scale, TreeVersion};
use treeknit_io::figure::LabelMode;

fn main() -> Result<(), serde_json::Error> {
  let lists = [
    value_list::<TreeVersion>("TREE_VERSION")?,
    value_list::<Scale>("SCALE")?,
    value_list::<LabelMode>("LABEL_MODE")?,
  ];
  let labels = LegendKind::VARIANTS
    .iter()
    .map(|kind| {
      Ok(format!(
        "  {}: {},",
        serde_json::to_string(kind)?,
        serde_json::to_string(kind.label())?
      ))
    })
    .collect::<Result<Vec<_>, serde_json::Error>>()?;
  let mut types: Vec<&str> = lists.iter().map(|list| list.type_name).chain(["LegendKind"]).collect();
  types.sort_unstable();
  println!("import type {{ {} }} from \"./treeknit_wasm\";", types.join(", "));
  for list in &lists {
    println!();
    println!(
      "export const {}_VALUES = [{}] as const satisfies readonly {}[];",
      list.prefix,
      list.values.join(", "),
      list.type_name
    );
    println!();
    println!(
      "export const {}_DEFAULT = {} as const satisfies {};",
      list.prefix, list.default, list.type_name
    );
  }
  println!();
  println!("export const LEGEND_LABELS = {{");
  for line in &labels {
    println!("{line}");
  }
  println!("}} as const satisfies Record<LegendKind, string>;");
  println!();
  println!("export const MIN_TREES = {MIN_TREES};");
  println!();
  println!("export const ARG_TREE_COUNT = {ARG_TREE_COUNT};");
  Ok(())
}

/// The values of `T` in declaration order and its default value, written by the serializer of the
/// boundary, so that they equal the strings that cross it.
fn value_list<T: VariantArray + Default + Serialize>(prefix: &'static str) -> Result<ValueList, serde_json::Error> {
  let values = T::VARIANTS
    .iter()
    .map(serde_json::to_string)
    .collect::<Result<_, _>>()?;
  let path = type_name::<T>();
  Ok(ValueList {
    prefix,
    type_name: path.rsplit("::").next().unwrap_or(path),
    values,
    default: serde_json::to_string(&T::default())?,
  })
}

/// The generated constants of one enum: the prefix of their names, the TypeScript type of the
/// values, and the values and the default value as JSON.
struct ValueList {
  prefix: &'static str,
  type_name: &'static str,
  values: Vec<String>,
  default: String,
}
