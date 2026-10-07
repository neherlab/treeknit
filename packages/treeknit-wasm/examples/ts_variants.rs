//! Write the TypeScript value lists of the Rust enums whose values the web app needs at runtime,
//! such as the choices of a drawing control, and the labels of the legend entries, which the
//! tooltips and the inspector share, to stdout. `just gen` writes them to
//! `pkg/treeknit_variants.ts`, next to the declarations of the module.

use serde::Serialize;
use std::any::type_name;
use strum::VariantArray;
use treeknit_io::display::{LegendKind, Scale, TreeVersion};
use treeknit_io::figure::LabelMode;

fn main() -> Result<(), serde_json::Error> {
  let lists = [
    value_list::<TreeVersion>("TREE_VERSION_VALUES")?,
    value_list::<Scale>("SCALE_VALUES")?,
    value_list::<LabelMode>("LABEL_MODE_VALUES")?,
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
      "export const {} = [{}] as const satisfies readonly {}[];",
      list.constant,
      list.values.join(", "),
      list.type_name
    );
  }
  println!();
  println!("export const LEGEND_LABELS = {{");
  for line in &labels {
    println!("{line}");
  }
  println!("}} as const satisfies Record<LegendKind, string>;");
  Ok(())
}

/// The values of `T` in declaration order, written by the serializer of the boundary, so that
/// they equal the strings that cross it.
fn value_list<T: VariantArray + Serialize>(constant: &'static str) -> Result<ValueList, serde_json::Error> {
  let values = T::VARIANTS
    .iter()
    .map(serde_json::to_string)
    .collect::<Result<_, _>>()?;
  let path = type_name::<T>();
  Ok(ValueList {
    constant,
    type_name: path.rsplit("::").next().unwrap_or(path),
    values,
  })
}

/// One generated constant: its name, the TypeScript type of its values, and the values as JSON.
struct ValueList {
  constant: &'static str,
  type_name: &'static str,
  values: Vec<String>,
}
