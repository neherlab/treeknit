//! The built-in examples: stable ids that links (`example=<id>`) and the command line
//! (`--example <id>`) name, with the trees of each example.
//!
//! Published links depend on the ids, so an example keeps its id when its directory or its
//! display name changes.

use crate::analysis;
use serde::Serialize;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// The menu group of an example.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub enum ExampleGroup {
  Small,
  Real,
  Simulated,
}

impl ExampleGroup {
  /// Title of the group in the Examples menu.
  pub fn name(self) -> &'static str {
    match self {
      ExampleGroup::Small => "Small",
      ExampleGroup::Real => "Real data (influenza A/H3N2)",
      ExampleGroup::Simulated => "Simulated (ARGTools)",
    }
  }
}

/// A built-in example.
#[derive(Clone, Copy, Debug)]
pub struct Example {
  /// Stable id, as in `example=<id>`.
  pub id: &'static str,
  /// Display name.
  pub name: &'static str,
  pub group: ExampleGroup,
  pub trees: &'static [ExampleTree],
}

/// A tree of an example: a file of the repository or an inline Newick text.
#[derive(Clone, Copy, Debug)]
pub struct ExampleTree {
  /// File name of the tree, which gives its label: `ha.nwk` gives `ha`.
  pub file: &'static str,
  pub text: ExampleText,
}

/// Where the Newick text of an example tree is.
#[derive(Clone, Copy, Debug)]
pub enum ExampleText {
  /// A file at a path relative to the repository root. With the cargo feature
  /// `embedded-examples`, `text` holds its content.
  File {
    path: &'static str,
    #[cfg(feature = "embedded-examples")]
    text: &'static str,
  },
  /// A Newick text written here.
  Inline(&'static str),
}

impl ExampleText {
  /// The Newick text: inline, or embedded with the feature `embedded-examples`.
  #[cfg(feature = "embedded-examples")]
  pub fn newick(&self) -> &'static str {
    match *self {
      ExampleText::File { text, .. } | ExampleText::Inline(text) => text,
    }
  }
}

/// The example tree in the file `<parent><dir>/<file>` of the repository, embedded with the
/// feature `embedded-examples`.
macro_rules! tree_file {
  ($parent:literal, $dir:literal, $file:literal) => {
    ExampleTree {
      file: $file,
      text: ExampleText::File {
        path: concat!($parent, $dir, "/", $file),
        #[cfg(feature = "embedded-examples")]
        text: include_str!(concat!("../../../", $parent, $dir, "/", $file)),
      },
    }
  };
}

/// The real influenza A/H3N2 example of directory `data/<id>` with the segments `files`.
macro_rules! real {
  ($id:literal, [$($file:literal),+]) => {
    Example {
      id: $id,
      name: $id,
      group: ExampleGroup::Real,
      trees: &[$(tree_file!("data/", $id, $file)),+],
    }
  };
}

/// The simulated example of directory `fixtures/sim/<id>` with the trees `files`.
macro_rules! simulated {
  ($id:literal, [$($file:literal),+]) => {
    Example {
      id: $id,
      name: $id,
      group: ExampleGroup::Simulated,
      trees: &[$(tree_file!("fixtures/sim/", $id, $file)),+],
    }
  };
}

/// Every example, in the order of the Examples menu: by group, then by id.
pub const EXAMPLES: &[Example] = &[
  Example {
    id: "5-leaves",
    name: "5 leaves",
    group: ExampleGroup::Small,
    trees: &[
      ExampleTree {
        file: "ha.nwk",
        text: ExampleText::Inline("((A,B),(C,(D,X)));"),
      },
      ExampleTree {
        file: "na.nwk",
        text: ExampleText::Inline("((A,(B,X)),(C,D));"),
      },
    ],
  },
  real!("h3n2-2012-2018", ["ha.nwk", "na.nwk"]),
  real!("h3n2-2017", ["ha.nwk", "na.nwk"]),
  real!("h3n2-2017-2018", ["ha.nwk", "na.nwk"]),
  real!("h3n2-2k-4-segments", ["ha.nwk", "na.nwk", "pb1.nwk", "pb2.nwk"]),
  real!("h3n2-new-york-1999-2004", ["ha.nwk", "na.nwk"]),
  simulated!("sim_k2_n100_r0.01", ["tree1.nwk", "tree2.nwk"]),
  simulated!("sim_k2_n100_r0.01_poly", ["tree1.nwk", "tree2.nwk"]),
  simulated!("sim_k2_n100_r0.02", ["tree1.nwk", "tree2.nwk"]),
  simulated!("sim_k2_n100_r0.02_poly", ["tree1.nwk", "tree2.nwk"]),
  simulated!("sim_k2_n100_r0.03", ["tree1.nwk", "tree2.nwk"]),
  simulated!("sim_k2_n100_r0.03_poly", ["tree1.nwk", "tree2.nwk"]),
  simulated!("sim_k2_n50_r0.02", ["tree1.nwk", "tree2.nwk"]),
  simulated!("sim_k2_n50_r0.02_poly", ["tree1.nwk", "tree2.nwk"]),
  simulated!("sim_k2_n50_r0.05", ["tree1.nwk", "tree2.nwk"]),
  simulated!("sim_k2_n50_r0.05_poly", ["tree1.nwk", "tree2.nwk"]),
  simulated!("sim_k2_n50_r0.1", ["tree1.nwk", "tree2.nwk"]),
  simulated!("sim_k2_n50_r0.1_poly", ["tree1.nwk", "tree2.nwk"]),
  simulated!("sim_k3_n50_r0.05", ["tree1.nwk", "tree2.nwk", "tree3.nwk"]),
  simulated!("sim_k3_n50_r0.05_poly", ["tree1.nwk", "tree2.nwk", "tree3.nwk"]),
  simulated!("sim_k3_n50_r0.1", ["tree1.nwk", "tree2.nwk", "tree3.nwk"]),
  simulated!("sim_k3_n50_r0.1_poly", ["tree1.nwk", "tree2.nwk", "tree3.nwk"]),
];

/// The example with id `id`.
pub fn example(id: &str) -> Option<&'static Example> {
  EXAMPLES.iter().find(|e| e.id == id)
}

impl Example {
  /// The labels of the trees, as a link or the web app gives them to trees loaded from the
  /// example's files: `analysis::tree_labels` of the file names.
  pub fn labels(&self) -> Vec<String> {
    let files: Vec<String> = self.trees.iter().map(|t| t.file.to_owned()).collect();
    analysis::tree_labels(&files, &[])
  }

  /// The labeled trees of the example, from the embedded texts.
  #[cfg(feature = "embedded-examples")]
  pub fn tree_texts(&self) -> Vec<analysis::TreeText> {
    self
      .trees
      .iter()
      .zip(self.labels())
      .map(|(t, label)| analysis::TreeText {
        label,
        newick: t.text.newick().to_owned(),
      })
      .collect()
  }
}

/// An example as the web app lists it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ExampleInfo {
  pub id: String,
  pub name: String,
  pub group: ExampleGroup,
  /// Title of the group in the Examples menu.
  pub group_name: String,
  pub trees: Vec<ExampleTreeInfo>,
}

/// A tree of an example as the web app lists it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct ExampleTreeInfo {
  /// File name, such as `ha.nwk`.
  pub file: String,
  /// Label of the tree, as in a link.
  pub label: String,
  /// Path of the tree file relative to the repository root; `None` for an inline tree.
  pub path: Option<String>,
  /// The Newick text of an inline tree; `None` for a tree file.
  pub newick: Option<String>,
}

/// Every example with its trees, in the order of `EXAMPLES`.
pub fn example_infos() -> Vec<ExampleInfo> {
  EXAMPLES
    .iter()
    .map(|e| ExampleInfo {
      id: e.id.to_owned(),
      name: e.name.to_owned(),
      group: e.group,
      group_name: e.group.name().to_owned(),
      trees: e
        .trees
        .iter()
        .zip(e.labels())
        .map(|(t, label)| {
          let (path, newick) = match t.text {
            ExampleText::File { path, .. } => (Some(path.to_owned()), None),
            ExampleText::Inline(text) => (None, Some(text.to_owned())),
          };
          ExampleTreeInfo {
            file: t.file.to_owned(),
            label,
            path,
            newick,
          }
        })
        .collect(),
    })
    .collect()
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;

  #[test]
  fn example_ids_are_fixed() {
    // Published links name these ids; a renamed example keeps its old id.
    let expected = [
      "5-leaves",
      "h3n2-2012-2018",
      "h3n2-2017",
      "h3n2-2017-2018",
      "h3n2-2k-4-segments",
      "h3n2-new-york-1999-2004",
      "sim_k2_n100_r0.01",
      "sim_k2_n100_r0.01_poly",
      "sim_k2_n100_r0.02",
      "sim_k2_n100_r0.02_poly",
      "sim_k2_n100_r0.03",
      "sim_k2_n100_r0.03_poly",
      "sim_k2_n50_r0.02",
      "sim_k2_n50_r0.02_poly",
      "sim_k2_n50_r0.05",
      "sim_k2_n50_r0.05_poly",
      "sim_k2_n50_r0.1",
      "sim_k2_n50_r0.1_poly",
      "sim_k3_n50_r0.05",
      "sim_k3_n50_r0.05_poly",
      "sim_k3_n50_r0.1",
      "sim_k3_n50_r0.1_poly",
    ];
    assert_eq!(expected.to_vec(), EXAMPLES.iter().map(|e| e.id).collect::<Vec<_>>());
  }

  #[test]
  fn example_labels_are_the_file_stems() {
    let labels = |id: &str| example(id).unwrap().labels();
    assert_eq!(vec!["ha", "na", "pb1", "pb2"], labels("h3n2-2k-4-segments"));
    assert_eq!(vec!["tree1", "tree2", "tree3"], labels("sim_k3_n50_r0.1"));
  }

  #[test]
  fn example_lookup_takes_the_exact_id() {
    assert_eq!(Some("h3n2-2017"), example("h3n2-2017").map(|e| e.id));
    assert!(example("H3N2-2017").is_none());
  }

  #[test]
  fn example_infos_list_inline_texts_and_file_paths() {
    let infos = example_infos();
    let small = &infos[0];
    assert_eq!(("5-leaves", "Small"), (small.id.as_str(), small.group_name.as_str()));
    assert_eq!(Some("((A,B),(C,(D,X)));"), small.trees[0].newick.as_deref());
    let real = infos.iter().find(|e| e.id == "h3n2-2017").unwrap();
    assert_eq!(Some("data/h3n2-2017/na.nwk"), real.trees[1].path.as_deref());
    assert_eq!("na", real.trees[1].label);
  }
}
