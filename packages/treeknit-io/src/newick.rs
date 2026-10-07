//! Newick reading and writing, in the classic dialect of `util_newick`: standard Newick, with
//! comments read as text.
//!
//! Supported: quoted labels (`'a b'`, with `''` for a quote), comments in square brackets
//! (skipped; brackets inside a comment nest), branch lengths, whitespace, and trees of any depth.
//! `#` is part of a label. Unnamed internal nodes, internal nodes labeled with a support value
//! (`95`, `0.95`, `80.5/95`) or another number, and duplicate internal names are renamed `NODE_i`.

use std::collections::BTreeSet;
use std::fmt;
use treeknit_core::{NodeId, Tree};
use util_newick::{
  Location, NewickDialect, NewickEdgeData, NewickError, NewickErrorKind, NewickGraph, NewickNodeData,
  NewickReadOptions, NewickWriteOptions, NumberFormat, newick_from_str, newick_to_string, newick_trees,
};

/// Parse a single Newick tree, logging its warnings.
pub fn parse(s: &str, label: &str) -> Result<Tree, ParseError> {
  let parsed = parse_first(s, label)?;
  parsed.log_warnings();
  Ok(parsed.tree)
}

/// Parse the first tree of a Newick file's content. The warnings are returned, not logged:
/// callers that read input files log them with `Parsed::log_warnings`.
///
/// The first tree ends at the first `;` outside quoted labels and comments. When text other than
/// comments follows it, the first tree is read alone, with the warning `SeveralTrees`, which the
/// error of a first tree with an unnamed or a duplicate leaf carries too.
pub fn parse_first(content: &str, label: &str) -> Result<Parsed, ParseError> {
  let options = NewickReadOptions::default();
  let (tree, mut warnings) = match newick_from_str(content, &options) {
    Ok(tree) => (tree, vec![]),
    Err(error) if error.kind == NewickErrorKind::MultipleTrees => {
      match newick_trees(content.as_bytes(), options).next() {
        Some(Ok(tree)) => (tree, vec![ParseWarning::SeveralTrees]),
        Some(Err(first_error)) => return Err(ParseError::from(first_error)),
        None => return Err(ParseError::from(error)),
      }
    },
    Err(error) => return Err(ParseError::from(error)),
  };
  warnings.splice(
    0..0,
    tree
      .warnings
      .iter()
      .map(|warning| ParseWarning::Reader(warning.to_string())),
  );
  let mut tree = tree_from_graph(&tree.graph, label);
  match fix_names(&mut tree) {
    Ok(()) => Ok(Parsed { tree, warnings }),
    Err(message) => Err(ParseError {
      message,
      location: None,
      warnings,
    }),
  }
}

/// Newick string of `t`, with internal labels and branch lengths. A branch length that is not a
/// finite number is an error, because Newick cannot hold it.
pub fn write(t: &Tree) -> Result<String, WriteError> {
  let mut graph = NewickGraph::new(node_data(t, t.root));
  let mut ids = vec![graph.root(); t.nodes.len()];
  for n in t.preorder() {
    for &c in t.children(n) {
      let edge = t
        .node(c)
        .branch_length
        .map_or_else(NewickEdgeData::new, |length| NewickEdgeData::new().with_length(length));
      ids[c] = graph
        .add_child(ids[n], edge, node_data(t, c))
        .map_err(WriteError::new)?;
    }
  }
  newick_to_string(&graph, &write_options(NewickDialect::CLASSIC)).map_err(WriteError::new)
}

#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("Newick parse error: {message}{}", at_byte(*.location))]
pub struct ParseError {
  pub message: String,
  /// Where parsing stopped; `None` for errors of the whole tree, such as duplicate leaf names.
  pub location: Option<Location>,
  /// The warnings that hold although the tree is rejected: `SeveralTrees` next to an error of the
  /// whole tree.
  pub warnings: Vec<ParseWarning>,
}

impl From<NewickError> for ParseError {
  fn from(error: NewickError) -> Self {
    ParseError {
      location: Some(error.location()),
      message: error.message,
      warnings: Vec::new(),
    }
  }
}

/// The ` at byte <offset>` suffix of a [`ParseError`] that has a location.
fn at_byte(location: Option<Location>) -> String {
  location.map(|l| format!(" at byte {}", l.offset)).unwrap_or_default()
}

/// A problem in a Newick text that parsing works around.
#[derive(Clone, Debug, PartialEq, Eq, strum::Display)]
pub enum ParseWarning {
  /// The file holds more than one tree; only the first is read.
  #[strum(to_string = "more than one tree in file, using the first")]
  SeveralTrees,
  /// A warning of the Newick reader, with its line and column.
  #[strum(to_string = "{0}")]
  Reader(String),
}

impl ParseWarning {
  /// Log the warning for the tree labeled `label`, as `<label>: <warning>`.
  pub fn log(&self, label: &str) {
    log::warn!("{label}: {self}");
  }
}

/// A parsed tree with the warnings of its text, in text order.
#[derive(Clone, Debug)]
pub struct Parsed {
  pub tree: Tree,
  pub warnings: Vec<ParseWarning>,
}

impl Parsed {
  /// Log the warnings, as the command line does when it reads a tree.
  pub fn log_warnings(&self) {
    for w in &self.warnings {
      w.log(&self.tree.label);
    }
  }
}

/// A tree that the Newick writer rejects.
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
#[error("Newick write error: {message}")]
pub struct WriteError {
  pub message: String,
}

impl WriteError {
  pub(crate) fn new(error: impl fmt::Display) -> Self {
    WriteError {
      message: format!("{error:#}"),
    }
  }
}

/// Write options of the Newick files of TreeKnit in `dialect`: lengths in their shortest exact
/// text, with `.0` on whole numbers (`1.0`), as TreeKnit.jl writes them.
pub(crate) fn write_options(dialect: NewickDialect) -> NewickWriteOptions {
  NewickWriteOptions {
    numbers: NumberFormat {
      point_zero: true,
      ..NumberFormat::default()
    },
    ..NewickWriteOptions::new(dialect)
  }
}

/// The tree `label` of the classic Newick `graph`, with its nodes, names, and branch lengths. A
/// node without a name, such as an internal node labeled with a support value, has the name "".
fn tree_from_graph(graph: &NewickGraph, label: &str) -> Tree {
  let mut t = Tree::new(label);
  t.nodes[t.root].name = graph.node(graph.root()).name().unwrap_or_default().to_owned();
  let mut ids = vec![t.root; graph.node_count()];
  for node in graph.preorder() {
    for &edge in graph.child_edges(node) {
      let entry = graph.edge(edge);
      let name = graph.node(entry.child()).name().unwrap_or_default();
      let child = t.add_node(name, entry.data().branch_length());
      t.attach(ids[node], child);
      ids[entry.child()] = child;
    }
  }
  t
}

/// Check that the leaves have distinct names and rename internal nodes without a usable name; the
/// error is the message of the first broken rule.
fn fix_names(t: &mut Tree) -> Result<(), String> {
  let mut leaves = BTreeSet::new();
  for n in t.leaves() {
    let name = &t.nodes[n].name;
    if name.is_empty() {
      return Err("unnamed leaf".to_owned());
    }
    if !leaves.insert(name.clone()) {
      return Err(format!("duplicate leaf name {name:?}"));
    }
  }
  let mut seen = leaves;
  let mut k = 0;
  for n in t.preorder() {
    if t.is_leaf(n) {
      continue;
    }
    let name = &t.nodes[n].name;
    if name.is_empty() || name.parse::<f64>().is_ok() || seen.contains(name) {
      loop {
        k += 1;
        let cand = format!("NODE_{k}");
        if !seen.contains(&cand) {
          t.nodes[n].name = cand;
          break;
        }
      }
    }
    seen.insert(t.nodes[n].name.clone());
  }
  Ok(())
}

/// The node data of node `n` of `t`: its name, or no name for "".
fn node_data(t: &Tree, n: NodeId) -> NewickNodeData {
  let name = t.name(n);
  if name.is_empty() {
    NewickNodeData::new()
  } else {
    NewickNodeData::new().with_name(name)
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use rstest::rstest;
  use std::fmt::Write as _;

  /// The parse error of `text`, or its tree written back as Newick, so a result compares whole.
  fn parsed(text: &str) -> Result<String, ParseError> {
    parse_first(text, "t").map(|p| write(&p.tree).unwrap())
  }

  /// A parse error with `message` at `location`, without warnings.
  fn parse_error(message: &str, location: Option<Location>) -> ParseError {
    ParseError {
      message: message.to_owned(),
      location,
      warnings: Vec::new(),
    }
  }

  /// The location at byte `offset`, line `line`, and column `column`.
  fn at(offset: usize, line: usize, column: usize) -> Option<Location> {
    Some(Location { offset, line, column })
  }

  #[test]
  fn roundtrip() {
    let s = "((A:1.5,'b c':0.25)x:1.0,(C,D)NODE_7,E:1e-20)root;";
    let t = parse(s, "t").unwrap();
    assert_eq!(t.leaf_names(), vec!["A", "b c", "C", "D", "E"]);
    assert_eq!(
      Ok("((A:1.5,'b c':0.25)x:1.0,(C,D)NODE_7,E:1.0e-20)root;".to_owned()),
      write(&t)
    );
  }

  #[test]
  fn comments_whitespace_and_support_values() {
    let t = parse("( (A[&x=1]:1 , B ) 95 : 2 ,C)\n;", "t").unwrap();
    assert_eq!(t.leaf_names(), vec!["A", "B", "C"]);
    assert_eq!(Ok("((A:1.0,B)NODE_2:2.0,C)NODE_1;".to_owned()), write(&t));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::support_value(      "((A,B)95,C);",            "((A,B)NODE_2,C)NODE_1;")]
  #[case::several_supports(   "((A,B)80.5/95,C);",       "((A,B)NODE_2,C)NODE_1;")]
  #[case::quoted_number(      "((A,B)'95',C);",          "((A,B)NODE_2,C)NODE_1;")]
  #[case::duplicate_internal( "((A,B)x,(C,D)x)r;",       "((A,B)x,(C,D)NODE_1)r;")]
  #[case::numeric_leaf_names( "((1,2)x,3)r;",            "((1,2)x,3)r;")]
  #[case::hash_in_names(      "(EPI_ISL#402124,B#H1)r;", "('EPI_ISL#402124','B#H1')r;")]
  #[case::nested_comment(     "(A[a[b]c],B)r;",          "(A,B)r;")]
  #[case::root_length_dropped("(A:1,B:2)r:3;",           "(A:1.0,B:2.0)r;")]
  #[trace]
  fn parse_and_write(#[case] text: &str, #[case] expected: &str) {
    assert_eq!(Ok(expected.to_owned()), parsed(text));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::whole(         1.0,       "1.0")]
  #[case::fraction(      0.25,      "0.25")]
  #[case::small(         0.0001,    "0.0001")]
  #[case::smaller(       0.00001,   "1.0e-5")]
  #[case::shortest_exact(0.1 + 0.2, "0.30000000000000004")]
  #[case::zero(          0.0,       "0.0")]
  #[case::large(         123456.5,  "123456.5")]
  #[case::largest_plain( 1e15,      "1000000000000000.0")]
  #[case::huge(          2.5e16,    "2.5e16")]
  #[trace]
  fn write_branch_length(#[case] length: f64, #[case] expected: &str) {
    let mut t = Tree::new("t");
    let a = t.add_node("A", Some(length));
    t.attach(t.root, a);
    let b = t.add_node("B", None);
    t.attach(t.root, b);
    assert_eq!(Ok(format!("(A:{expected},B);")), write(&t));
  }

  #[test]
  fn write_rejects_a_length_that_is_not_finite() {
    let mut t = Tree::new("t");
    let a = t.add_node("A", Some(f64::INFINITY));
    t.attach(t.root, a);
    let expected = WriteError {
      message:
        "When writing Newick: When writing the branch above node 1 ('A'): Newick cannot represent the number inf"
          .to_owned(),
    };
    assert_eq!(Err(expected), write(&t));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::not_a_number(  "((A,B):0.R,C);", parse_error("expected comment, ')', ',', ';'", at(9, 1, 10)))]
  #[case::out_of_range(  "(A:1e999,B);",   parse_error("\"1e999\" is too large for a 64-bit floating-point number", at(3, 1, 4)))]
  #[trace]
  fn invalid_length_is_an_error(#[case] text: &str, #[case] expected: ParseError) {
    assert_eq!(Err(expected), parsed(text));
  }

  #[test]
  fn parse_first_of_a_clean_text_has_no_warnings() {
    let parsed = parse_first("((A:1,B:2):3,C:4);\n", "t").unwrap();
    assert_eq!(Vec::<ParseWarning>::new(), parsed.warnings);
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::semicolon_in_quoted_label(    "('a;b',C);",          (vec!["a;b", "C"], vec![]))]
  #[case::semicolon_in_comment(         "(A[;],B);",           (vec!["A", "B"],   vec![]))]
  #[case::semicolon_in_trailing_comment("(A,B);[x;y]\n",       (vec!["A", "B"],   vec![]))]
  #[case::second_tree(                  "(A,B);\n(C,D);\n",    (vec!["A", "B"],   vec![ParseWarning::SeveralTrees]))]
  #[case::second_tree_after_comment(    "(A,B);[c](C,D);",     (vec!["A", "B"],   vec![ParseWarning::SeveralTrees]))]
  #[case::second_tree_with_quotes(      "(A,B);\n('x;y',z);",  (vec!["A", "B"],   vec![ParseWarning::SeveralTrees]))]
  #[case::broken_second_tree(           "(A,B);\n(C,,;",       (vec!["A", "B"],   vec![ParseWarning::SeveralTrees]))]
  #[case::trailing_text(                "(A,B);\nx",           (vec!["A", "B"],   vec![ParseWarning::SeveralTrees]))]
  #[trace]
  fn parse_first_reads_the_tree_up_to_its_semicolon(
    #[case] text: &str,
    #[case] (leaves, warnings): (Vec<&str>, Vec<ParseWarning>),
  ) {
    let parsed = parse_first(text, "t").unwrap();
    let leaves: Vec<String> = leaves.into_iter().map(ToOwned::to_owned).collect();
    assert_eq!((leaves, warnings), (parsed.tree.leaf_names(), parsed.warnings));
  }

  #[test]
  fn several_trees_warning_stays_next_to_an_error_of_the_whole_tree() {
    let expected = ParseError {
      warnings: vec![ParseWarning::SeveralTrees],
      ..parse_error("duplicate leaf name \"A\"", None)
    };
    assert_eq!(Err(expected), parsed("(A,A);\n(A,B);\n"));
  }

  #[test]
  fn parse_first_without_a_semicolon_outside_quotes_fails() {
    let expected = parse_error("The tree does not end with ';'", at(9, 1, 10));
    assert_eq!(Err(expected.clone()), parsed("('a;b',C)"));
    assert_eq!(
      "Newick parse error: The tree does not end with ';' at byte 9",
      expected.to_string()
    );
  }

  #[test]
  fn warnings_keep_the_log_lines_of_the_command_line() {
    assert_eq!(
      "more than one tree in file, using the first",
      ParseWarning::SeveralTrees.to_string()
    );
  }

  #[test]
  fn duplicate_leaves_and_unclosed_trees_are_rejected() {
    let expected = [
      Err(parse_error("duplicate leaf name \"A\"", None)),
      Err(parse_error("The tree does not end with ';'", at(4, 1, 5))),
    ];
    assert_eq!(expected, [parsed("(A,A);"), parsed("(A,B")]);
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::unexpected_token("((A,B)C D);", parse_error("expected comment, ')', ',', ':', ';'", at(8, 1, 9)))]
  #[case::unclosed(        "((A,B)C;",    parse_error("the '(' is never closed", at(0, 1, 1)))]
  #[trace]
  fn syntax_error_has_its_location(#[case] text: &str, #[case] expected: ParseError) {
    assert_eq!(Err(expected), parsed(text));
  }

  #[test]
  fn tree_error_has_no_location() {
    let expected = parse_error("duplicate leaf name \"A\"", None);
    assert_eq!(Err(expected.clone()), parsed("(A,A);"));
    assert_eq!("Newick parse error: duplicate leaf name \"A\"", expected.to_string());
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::second_line("(A,\n(B,C)D x);",   (2, 8))]
  #[case::crlf(       "(A,\r\n(B,C)D x;", (2, 8))]
  #[case::characters( "((é,ü)x y,C);",     (1, 9))]
  #[trace]
  fn error_location_counts_lines_and_characters(#[case] text: &str, #[case] (line, column): (usize, usize)) {
    let position = parsed(text).map_err(|e| e.location.map(|l| (l.line, l.column)));
    assert_eq!(Err(Some((line, column))), position);
  }

  #[test]
  fn deep_caterpillar_tree_reads_and_writes() {
    let depth = 200_000;
    let mut text = "(".repeat(depth - 1);
    text.push_str("L0");
    for i in 1..depth {
      write!(text, ",L{i})NODE_{}", depth - i).unwrap();
    }
    text.push(';');
    let t = parse(&text, "t").unwrap();
    assert_eq!((depth, Ok(text.clone())), (t.n_leaves(), write(&t)));
  }
}
