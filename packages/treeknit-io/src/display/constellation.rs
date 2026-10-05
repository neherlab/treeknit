//! The MCC of every leaf in every pair.

use super::pair::pair_slots;
use super::{ConstellationCell, ConstellationTable};
use crate::run::RunResult;
use treeknit_core::mcc_map::leaf_mcc_map;

/// The MCC of every taxon of `run` in every pair, with the color slots of the pair views.
pub fn constellation(run: &RunResult) -> ConstellationTable {
  let n = run.taxa.len();
  // Taxa in row order: the leaves of the first final tree, then each later tree's leaves that no
  // earlier tree has.
  let mut seen = vec![false; n];
  let mut rows = Vec::with_capacity(n);
  for tree in &run.trees {
    for leaf in tree.leaves() {
      if let Some(x) = tree.node(leaf).taxon.filter(|&x| !seen[x]) {
        seen[x] = true;
        rows.push(x);
      }
    }
  }
  // An MCC holds only leaves of the two trees of its pair, so a leaf has an MCC in every pair
  // that has it.
  let leaf_mccs: Vec<Vec<Option<usize>>> = run.pairs.iter().map(|p| leaf_mcc_map(&p.mccs, n)).collect();
  let slots: Vec<&[usize]> = (0..run.pairs.len()).map(|i| pair_slots(run, i)).collect();
  let cells = rows
    .iter()
    .map(|&x| {
      run
        .pairs
        .iter()
        .zip(&leaf_mccs)
        .zip(&slots)
        .map(|((p, leaf_mcc), slots)| {
          let mcc = leaf_mcc[x]?;
          Some(ConstellationCell {
            mcc,
            size: p.mccs[mcc].len(),
            slot: slots[mcc],
          })
        })
        .collect()
    })
    .collect();
  ConstellationTable {
    leaves: rows.iter().map(|&x| run.taxa.names[x].clone()).collect(),
    pairs: run
      .pairs
      .iter()
      .map(|p| [run.trees[p.i].label.clone(), run.trees[p.j].label.clone()])
      .collect(),
    cells,
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::analysis::{self, Settings, TreeText};
  use crate::display::{Scale, TreeVersion, pair_view};
  use crate::run;
  use pretty_assertions::assert_eq;

  fn run_trees(trees: &[(&str, &str)]) -> RunResult {
    let texts: Vec<TreeText> = trees
      .iter()
      .map(|(label, newick)| TreeText {
        label: (*label).to_owned(),
        newick: (*newick).to_owned(),
      })
      .collect();
    let s = Settings::default();
    let opts = analysis::options(&s, texts.len(), false).unwrap();
    run::run(analysis::parse_trees(&texts).unwrap(), &opts, s.seed, &|_| {})
  }

  #[test]
  fn constellation_of_the_two_tree_example() {
    let r = run_trees(&[("ha", "((A,B),(C,(D,X)));"), ("na", "((A,(B,X)),(C,D));")]);
    let table = constellation(&r);
    assert_eq!(r.trees[0].leaf_names(), table.leaves);
    assert_eq!(vec![["ha".to_owned(), "na".to_owned()]], table.pairs);
    let row = |name: &str| &table.cells[table.leaves.iter().position(|l| l == name).unwrap()];
    // Slots as in the pair view: the larger MCC [A,B,C,D] slot 0, X slot 1.
    let cell = |mcc, size, slot| Some(ConstellationCell { mcc, size, slot });
    assert_eq!(&vec![cell(0, 1, 1)], row("X"));
    assert_eq!(&vec![cell(1, 4, 0)], row("A"));
    let view = pair_view(&r, 0, TreeVersion::Resolved, Scale::Div).unwrap();
    assert_eq!(view.mccs[0].slot, row("X")[0].unwrap().slot);
  }

  #[test]
  fn constellation_has_a_row_for_each_leaf_only_in_later_trees() {
    // R is in seg1 and seg2, Q in seg2 only. Their rows follow the leaves of seg0, in the order
    // of the first tree that has them.
    let r = run_trees(&[
      ("seg0", "((A,B),(C,(D,(E,X))));"),
      ("seg1", "((A,(B,X)),(C,D,(E,R)));"),
      ("seg2", "((A,(B,Q)),((C,D),(E,(X,R))));"),
    ]);
    let t = constellation(&r);
    let seg0: Vec<String> = r.trees[0].leaf_names();
    assert_eq!(seg0[..], t.leaves[..seg0.len()]);
    assert_eq!(vec!["R".to_owned(), "Q".to_owned()], t.leaves[seg0.len()..].to_vec());
    assert_eq!(3, t.pairs.len());
    assert!(t.cells.iter().all(|row| row.len() == 3));
    // Every pair holds R, attached in the pairs with seg0. Q is in neither tree of pair (0,1).
    let (row_r, row_q) = (&t.cells[seg0.len()], &t.cells[seg0.len() + 1]);
    assert!(row_r.iter().all(Option::is_some));
    assert_eq!(None, row_q[0]);
    assert!(row_q[1].is_some() && row_q[2].is_some());
  }
}
