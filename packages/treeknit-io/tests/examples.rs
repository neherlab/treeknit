//! The example catalog against the files of the repository.

#[cfg(test)]
mod tests {
  use pretty_assertions::assert_eq;
  use std::collections::BTreeSet;
  use std::fs;
  use std::path::{Path, PathBuf};
  use treeknit_io::examples::{EXAMPLES, ExampleText};

  fn repository() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
  }

  #[test]
  fn every_listed_tree_file_exists() {
    let missing: Vec<&str> = EXAMPLES
      .iter()
      .flat_map(|e| e.trees)
      .filter_map(|t| match t.text {
        ExampleText::File { path, .. } => (!repository().join(path).is_file()).then_some(path),
        ExampleText::Inline(_) => None,
      })
      .collect();
    assert_eq!(Vec::<&str>::new(), missing);
  }

  #[test]
  fn every_tree_file_of_the_example_directories_is_listed() {
    // A new directory under data/ or fixtures/sim/ needs an id in the catalog.
    let listed: BTreeSet<String> = EXAMPLES
      .iter()
      .flat_map(|e| e.trees)
      .filter_map(|t| match t.text {
        ExampleText::File { path, .. } => Some(path.to_owned()),
        ExampleText::Inline(_) => None,
      })
      .collect();
    let mut found = BTreeSet::new();
    for parent in ["data", "fixtures/sim"] {
      for dir in fs::read_dir(repository().join(parent)).unwrap() {
        let dir = dir.unwrap();
        if !dir.file_type().unwrap().is_dir() {
          continue;
        }
        for file in fs::read_dir(dir.path()).unwrap() {
          let path = file.unwrap().path();
          if path.extension().is_some_and(|e| e == "nwk") {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            found.insert(format!("{parent}/{}/{name}", dir.file_name().to_string_lossy()));
          }
        }
      }
    }
    assert_eq!(found, listed);
  }
}
