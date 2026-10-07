//! ARG output: extended Newick (Cardona et al. 2008) with BEAST annotations, and the table of the
//! tree nodes of each ARG node.

use crate::newick::{WriteError, write_options};
use treeknit_core::arg::{Anc, Arg};
use util_newick::{
  LabelSide, NewickArray, NewickComment, NewickDialect, NewickEdgeData, NewickGraph, NewickHybrid, NewickNodeData,
  NewickValue, NodeComment, newick_to_string,
};

/// Label of the synthetic root above two unshared roots.
pub(crate) const GLOBAL_ROOT: &str = "GlobalRoot";

/// Extended Newick string. Every branch is annotated with the segments it carries
/// (`[&segments={0,1}]`, 0-based); reassortment nodes appear once per parent, as `label#Hi`, with
/// the annotation and length of that parent's branch. A reassortment node is written in full where
/// it is first reached, and `i` counts them in the order their first copies end. Labels are quoted
/// where Newick needs it, so `B,1` is written `'B,1'`.
pub fn extended_newick(arg: &Arg) -> Result<String, WriteError> {
  let [r1, r2] = arg.roots;
  let (mut graph, tops) = if r1 == r2 {
    (NewickGraph::new(root_data(arg, r1)), vec![r1])
  } else {
    match (arg.nodes[r1].is_shared(), arg.nodes[r2].is_shared()) {
      (false, false) => {
        let mut graph = NewickGraph::new(
          NewickNodeData::new()
            .with_name(GLOBAL_ROOT)
            .with_comment(segments_comment(&[0, 1])),
        );
        graph.root_edge_mut().set_branch_length(Some(0.0));
        (graph, vec![r1, r2])
      },
      (true, _) => (NewickGraph::new(root_data(arg, r2)), vec![r2]),
      (false, true) => (NewickGraph::new(root_data(arg, r1)), vec![r1]),
    }
  };
  if let [top] = tops.as_slice() {
    graph.root_edge_mut().set_branch_length(branch(arg, *top, None).length);
  }
  let numbers = hybrid_numbers(arg, &tops);
  add_nodes(&mut graph, arg, &tops, &numbers).map_err(WriteError::new)?;
  newick_to_string(&graph, &write_options(NewickDialect::ENEWICK_BEAST)).map_err(WriteError::new)
}

/// One line per ARG node: `arg_node,tree1_node,tree2_node`, with a space for "none".
pub fn node_table(arg: &Arg) -> String {
  arg
    .nodes
    .iter()
    .zip(&arg.tree_nodes)
    .map(|(n, [a, b])| {
      format!(
        "{},{},{}",
        n.label,
        a.as_deref().unwrap_or(" "),
        b.as_deref().unwrap_or(" ")
      )
    })
    .collect::<Vec<_>>()
    .join("\n")
}

/// The number `i` of the tag `#Hi` of each reassortment node: 1, 2, ... in the order in which the
/// first copies of the nodes end in a depth-first walk from `tops`, children in their order.
fn hybrid_numbers(arg: &Arg, tops: &[usize]) -> Vec<Option<u32>> {
  let mut numbers = vec![None; arg.nodes.len()];
  let mut count = 0;
  for &top in tops {
    let mut stack = vec![Visit { node: top, next: 0 }];
    while let Some(visit) = stack.last_mut() {
      let n = visit.node;
      if let Some(&c) = arg.nodes[n].children.get(visit.next) {
        visit.next += 1;
        if numbers[c].is_none() {
          stack.push(Visit { node: c, next: 0 });
        }
        continue;
      }
      stack.pop();
      if arg.nodes[n].hybrid && numbers[n].is_none() {
        count += 1;
        numbers[n] = Some(count);
      }
    }
  }
  numbers
}

/// Add the nodes below `tops` to `graph`, whose root is the parent of `tops` (or the single top
/// itself), with one edge per ARG branch.
fn add_nodes(graph: &mut NewickGraph, arg: &Arg, tops: &[usize], numbers: &[Option<u32>]) -> Result<(), WriteError> {
  let mut ids: Vec<Option<usize>> = vec![None; arg.nodes.len()];
  let mut stack = Vec::new();
  if let [top] = tops {
    ids[*top] = Some(graph.root());
    stack.push(Frame {
      node: *top,
      id: graph.root(),
      next: 0,
    });
  } else {
    for &top in tops {
      let (edge, node) = child_data(arg, top, None, numbers);
      let id = graph.add_child(graph.root(), edge, node).map_err(WriteError::new)?;
      ids[top] = Some(id);
      stack.push(Frame { node: top, id, next: 0 });
    }
    stack.reverse();
  }
  while let Some(frame) = stack.last_mut() {
    let (n, parent) = (frame.node, frame.id);
    let Some(&c) = arg.nodes[n].children.get(frame.next) else {
      stack.pop();
      continue;
    };
    frame.next += 1;
    let (edge, node) = child_data(arg, c, Some(n), numbers);
    if let Some(existing) = ids[c] {
      graph.add_edge(parent, existing, edge).map_err(WriteError::new)?;
    } else {
      let id = graph.add_child(parent, edge, node).map_err(WriteError::new)?;
      ids[c] = Some(id);
      stack.push(Frame { node: c, id, next: 0 });
    }
  }
  Ok(())
}

/// The data of the root `n` of the graph: its label and the segments of its branch.
fn root_data(arg: &Arg, n: usize) -> NewickNodeData {
  NewickNodeData::new()
    .with_name(&arg.nodes[n].label)
    .with_comment(segments_comment(&branch(arg, n, None).segments))
}

/// The data of the branch from `anc` to `n`, and of node `n`: its label, and its tag `#Hi` if it
/// is a reassortment node. The segments of the branch are a comment of node `n`, or of this copy
/// of it for a reassortment node, which has one copy per parent.
fn child_data(arg: &Arg, n: usize, anc: Option<usize>, numbers: &[Option<u32>]) -> (NewickEdgeData, NewickNodeData) {
  let label = &arg.nodes[n].label;
  let branch = branch(arg, n, anc);
  let edge = branch
    .length
    .map_or_else(NewickEdgeData::new, |length| NewickEdgeData::new().with_length(length));
  let node = if label.is_empty() {
    NewickNodeData::new()
  } else {
    NewickNodeData::new().with_name(label)
  };
  let comment = segments_comment(&branch.segments);
  match numbers[n] {
    Some(i) => (
      edge.with_occurrence_comment(comment),
      node.with_hybrid(NewickHybrid::new(Some("H".to_owned()), i)),
    ),
    None => (edge, node.with_comment(comment)),
  }
}

/// The segments and length of the branch from `anc` to `n`. Without `anc`, the segments of `n`
/// that do not end at a segment root, or else all its segments.
fn branch(arg: &Arg, n: usize, anc: Option<usize>) -> Branch {
  let node = &arg.nodes[n];
  let segments: Vec<usize> = match anc {
    Some(a) => arg.edge_segments(a, n),
    None => (0..2).filter(|&c| node.has(c) && !node.is_root(c)).collect(),
  };
  let segments = if segments.is_empty() {
    (0..2).filter(|&c| node.has(c)).collect()
  } else {
    segments
  };
  let length = segments
    .first()
    .filter(|&&c| matches!(node.anc[c], Anc::Node(_)))
    .and_then(|_| segments.iter().find_map(|&c| node.tau[c]));
  Branch { segments, length }
}

/// The annotation `[&segments={...}]` after a label.
fn segments_comment(segments: &[usize]) -> NodeComment {
  let values = segments.iter().map(|&c| segment_value(c)).collect();
  NodeComment::new(
    LabelSide::AfterLabel,
    NewickComment::Beast(vec![(
      "segments".to_owned(),
      NewickValue::Array(NewickArray::new(values)),
    )]),
  )
}

#[expect(
  clippy::as_conversions,
  reason = "segment indices are 0 and 1, which f64 holds exactly"
)]
fn segment_value(c: usize) -> NewickValue {
  NewickValue::Number(c as f64)
}

struct Visit {
  node: usize,
  next: usize,
}

struct Frame {
  node: usize,
  id: usize,
  next: usize,
}

struct Branch {
  segments: Vec<usize>,
  length: Option<f64>,
}
