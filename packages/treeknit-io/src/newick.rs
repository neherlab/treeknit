//! Newick reading and writing.
//!
//! Supported: quoted labels (`'a b'`, with `''` for a quote), comments in square brackets
//! (skipped), branch lengths, whitespace. Unnamed or numeric (support value) internal
//! nodes and duplicate internal names are renamed `NODE_i`.

#![expect(
  clippy::disallowed_types,
  clippy::unwrap_used,
  reason = "findings from before the strict lint set; kb/issues/N-lint-baseline.md tracks their removal"
)]

use std::collections::HashSet;
use std::fmt::Write;
use treeknit_core::{NodeId, Tree};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
  pub message: String,
  /// Byte offset in the text where parsing stopped; `None` for errors of the whole tree, such
  /// as duplicate leaf names.
  pub offset: Option<usize>,
}

impl ParseError {
  fn new(message: impl Into<String>) -> Self {
    ParseError {
      message: message.into(),
      offset: None,
    }
  }
}

impl std::fmt::Display for ParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Newick parse error: {}", self.message)?;
    if let Some(offset) = self.offset {
      write!(f, " at byte {offset}")?;
    }
    Ok(())
  }
}
impl std::error::Error for ParseError {}

/// 1-based line and column of byte `offset` in `text`, with the column counted in Unicode
/// characters, as editors show positions. An offset past the end gives the end of the text.
pub fn line_column(text: &str, offset: usize) -> (usize, usize) {
  let before = &text.as_bytes()[..offset.min(text.len())];
  let line_start = before.iter().rposition(|&b| b == b'\n').map_or(0, |i| i + 1);
  let line = before.split(|&b| b == b'\n').count();
  // Count the bytes that start a character, so an offset inside a character counts it.
  let column = before[line_start..].iter().filter(|&&b| b & 0xC0 != 0x80).count() + 1;
  (line, column)
}

/// A problem in a Newick text that parsing works around.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ParseWarning {
  /// The file holds more than one tree; only the first is read.
  SeveralTrees,
  /// A branch length that is not a number, read as a missing length.
  InvalidLength(String),
}

impl ParseWarning {
  /// Log the warning for the tree labeled `label`, with the lines the command line has always
  /// written.
  pub fn log(&self, label: &str) {
    match self {
      ParseWarning::SeveralTrees => log::warn!("{label}: {self}"),
      ParseWarning::InvalidLength(_) => log::warn!("{self}"),
    }
  }
}

impl std::fmt::Display for ParseWarning {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      ParseWarning::SeveralTrees => f.write_str("more than one tree in file, using the first"),
      ParseWarning::InvalidLength(text) => write!(f, "ignoring invalid branch length '{text}'"),
    }
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

struct Parser<'a> {
  s: &'a [u8],
  i: usize,
  warnings: Vec<ParseWarning>,
}

impl Parser<'_> {
  fn err<T>(&self, msg: &str) -> Result<T, ParseError> {
    Err(ParseError {
      message: msg.to_owned(),
      offset: Some(self.i),
    })
  }

  /// Skip whitespace and `[...]` comments.
  fn skip(&mut self) -> Result<(), ParseError> {
    loop {
      while self.i < self.s.len() && self.s[self.i].is_ascii_whitespace() {
        self.i += 1;
      }
      if self.peek() == Some(b'[') {
        match self.s[self.i..].iter().position(|&c| c == b']') {
          Some(k) => self.i += k + 1,
          None => return self.err("unterminated comment"),
        }
      } else {
        return Ok(());
      }
    }
  }

  fn peek(&self) -> Option<u8> {
    self.s.get(self.i).copied()
  }

  fn label(&mut self) -> Result<String, ParseError> {
    self.skip()?;
    if self.peek() == Some(b'\'') {
      let mut out = Vec::new();
      self.i += 1;
      loop {
        match self.peek() {
          None => return self.err("unterminated quoted label"),
          Some(b'\'') if self.s.get(self.i + 1) == Some(&b'\'') => {
            out.push(b'\'');
            self.i += 2;
          },
          Some(b'\'') => {
            self.i += 1;
            break;
          },
          Some(c) => {
            out.push(c);
            self.i += 1;
          },
        }
      }
      return String::from_utf8(out).map_err(|e| ParseError::new(e.to_string()));
    }
    let start = self.i;
    while let Some(c) = self.peek() {
      if b"(),:;[".contains(&c) || c.is_ascii_whitespace() {
        break;
      }
      self.i += 1;
    }
    Ok(String::from_utf8_lossy(&self.s[start..self.i]).into_owned())
  }

  fn length(&mut self) -> Result<Option<f64>, ParseError> {
    self.skip()?;
    if self.peek() != Some(b':') {
      return Ok(None);
    }
    self.i += 1;
    self.skip()?;
    let start = self.i;
    while let Some(c) = self.peek() {
      if b"(),:;[".contains(&c) || c.is_ascii_whitespace() {
        break;
      }
      self.i += 1;
    }
    let txt = String::from_utf8_lossy(&self.s[start..self.i]);
    if let Ok(x) = txt.parse::<f64>() {
      Ok(Some(x))
    } else {
      self.warnings.push(ParseWarning::InvalidLength(txt.into_owned()));
      Ok(None)
    }
  }

  /// Parse one tree up to its terminating `;`, and leave the position at that `;`.
  fn tree(&mut self, label: &str) -> Result<Tree, ParseError> {
    let mut t = Tree::new(label);
    let root = t.root;
    self.subtree(&mut t, root)?;
    self.skip()?;
    match self.peek() {
      Some(b';') => {},
      None => return Err(ParseError::new("no ';' found")),
      Some(_) => return self.err("expected ';'"),
    }
    t.nodes[root].branch_length = None;
    fix_names(&mut t)?;
    Ok(t)
  }

  /// Move to the next `;` outside quoted labels and comments, the end of a tree, and return
  /// whether there is one. The text is split into the tokens of the parser without building a
  /// tree, so a syntax error does not stop the search; an unterminated quoted label or comment
  /// ends it.
  fn next_tree_end(&mut self) -> bool {
    loop {
      if self.skip().is_err() {
        return false;
      }
      match self.peek() {
        None => return false,
        Some(b';') => return true,
        Some(b'(' | b')' | b',' | b':') => self.i += 1,
        Some(_) => {
          if self.label().is_err() {
            return false;
          }
        },
      }
    }
  }

  fn subtree(&mut self, t: &mut Tree, n: NodeId) -> Result<(), ParseError> {
    self.skip()?;
    if self.peek() == Some(b'(') {
      self.i += 1;
      loop {
        let c = t.add_node("", None);
        self.subtree(t, c)?;
        t.attach(n, c);
        self.skip()?;
        match self.peek() {
          Some(b',') => self.i += 1,
          Some(b')') => {
            self.i += 1;
            break;
          },
          _ => return self.err("expected ',' or ')'"),
        }
      }
    }
    t.nodes[n].name = self.label()?;
    t.nodes[n].branch_length = self.length()?;
    Ok(())
  }
}

/// Parse a single Newick tree, logging its warnings.
pub fn parse(s: &str, label: &str) -> Result<Tree, ParseError> {
  let parsed = parse_first(s, label)?;
  parsed.log_warnings();
  Ok(parsed.tree)
}

/// Parse the first tree of a Newick file's content. The warnings are returned, not logged:
/// callers that read input files log them with `Parsed::log_warnings`.
///
/// The first tree ends at the first `;` outside quoted labels and comments. The text after it
/// is not parsed; a further `;` there gives the warning `SeveralTrees` (see [`holds_several_trees`]).
pub fn parse_first(content: &str, label: &str) -> Result<Parsed, ParseError> {
  let mut p = Parser {
    s: content.as_bytes(),
    i: 0,
    warnings: Vec::new(),
  };
  let tree = p.tree(label)?;
  let mut warnings = p.warnings;
  if holds_several_trees(content) {
    warnings.insert(0, ParseWarning::SeveralTrees);
  }
  Ok(Parsed { tree, warnings })
}

/// Whether `content` holds more than one tree: two `;` outside quoted labels and comments. This
/// holds whether or not the first tree parses, so callers can report it next to a parse error.
pub fn holds_several_trees(content: &str) -> bool {
  let mut p = Parser {
    s: content.as_bytes(),
    i: 0,
    warnings: Vec::new(),
  };
  if !p.next_tree_end() {
    return false;
  }
  p.i += 1;
  p.next_tree_end()
}

fn fix_names(t: &mut Tree) -> Result<(), ParseError> {
  let mut leaves = HashSet::new();
  for n in t.leaves() {
    let name = &t.nodes[n].name;
    if name.is_empty() {
      return Err(ParseError::new("unnamed leaf"));
    }
    if !leaves.insert(name.clone()) {
      return Err(ParseError::new(format!("duplicate leaf name {name}")));
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

fn quote(name: &str) -> String {
  if name.bytes().any(|c| b"(),:;[]' \t\n".contains(&c)) {
    format!("'{}'", name.replace('\'', "''"))
  } else {
    name.to_owned()
  }
}

/// Newick string of `t`, with internal labels and branch lengths.
pub fn write(t: &Tree) -> String {
  // Iterative writer to avoid deep recursion on ladder-like trees.
  enum Step {
    Enter(NodeId),
    Exit(NodeId),
    Comma,
  }
  let mut s = String::new();
  let mut stack = vec![Step::Enter(t.root)];
  while let Some(step) = stack.pop() {
    match step {
      Step::Enter(n) => {
        let ch = t.children(n);
        if ch.is_empty() {
          write_node(&mut s, t, n);
        } else {
          s.push('(');
          stack.push(Step::Exit(n));
          for (k, &c) in ch.iter().enumerate().rev() {
            stack.push(Step::Enter(c));
            if k > 0 {
              stack.push(Step::Comma);
            }
          }
        }
      },
      Step::Exit(n) => {
        s.push(')');
        write_node(&mut s, t, n);
      },
      Step::Comma => s.push(','),
    }
  }
  s.push(';');
  s
}

fn write_node(s: &mut String, t: &Tree, n: NodeId) {
  s.push_str(&quote(t.name(n)));
  if n != t.root {
    if let Some(b) = t.node(n).branch_length {
      write!(s, ":{}", fmt_f64(b)).unwrap();
    }
  }
}

/// Shortest representation that round-trips; exponent notation for very small/large values.
pub fn fmt_f64(x: f64) -> String {
  format!("{x:?}")
}

#[cfg(test)]
mod tests {
  use super::*;
  use pretty_assertions::assert_eq;
  use rstest::rstest;

  #[test]
  fn roundtrip() {
    let s = "((A:1.5,'b c':0.25)x:1.0,(C,D)NODE_7,E:1e-20)root;";
    let t = parse(s, "t").unwrap();
    assert_eq!(t.leaf_names(), vec!["A", "b c", "C", "D", "E"]);
    assert_eq!(write(&t), "((A:1.5,'b c':0.25)x:1.0,(C,D)NODE_7,E:1e-20)root;");
  }

  #[test]
  fn comments_whitespace_and_support_values() {
    let t = parse("( (A[&x=1]:1 , B ) 95 : 2 ,C)\n;", "t").unwrap();
    assert_eq!(t.leaf_names(), vec!["A", "B", "C"]);
    assert_eq!(write(&t), "((A:1.0,B)NODE_2:2.0,C)NODE_1;");
  }

  #[test]
  fn invalid_length_is_missing() {
    let t = parse("((A,B):0.R,C);", "t").unwrap();
    assert_eq!(write(&t), "((A,B)NODE_2,C)NODE_1;");
  }

  #[test]
  fn parse_first_returns_the_warnings_in_text_order() {
    let parsed = parse_first("((A,B):0.R,C:x);\n(A,B,C);\n", "t").unwrap();
    let expected = vec![
      ParseWarning::SeveralTrees,
      ParseWarning::InvalidLength("0.R".into()),
      ParseWarning::InvalidLength("x".into()),
    ];
    assert_eq!(expected, parsed.warnings);
    assert_eq!("((A,B)NODE_2,C)NODE_1;", write(&parsed.tree));
  }

  #[test]
  fn parse_first_of_a_clean_text_has_no_warnings() {
    let parsed = parse_first("((A:1,B:2):3,C:4);\n", "t").unwrap();
    assert_eq!(Vec::<ParseWarning>::new(), parsed.warnings);
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::semicolon_in_quoted_label(   "('a;b',C);",          (vec!["a;b", "C"], vec![]))]
  #[case::semicolon_in_comment(        "(A[;],B);",           (vec!["A", "B"],   vec![]))]
  #[case::semicolon_in_trailing_comment("(A,B);[x;y]\n",     (vec!["A", "B"],   vec![]))]
  #[case::semicolon_in_trailing_quote( "(A,B);\n('x;y'",     (vec!["A", "B"],   vec![]))]
  #[case::second_tree_after_comment(   "(A,B);[c](C,D);",     (vec!["A", "B"],   vec![ParseWarning::SeveralTrees]))]
  #[case::second_tree_with_quotes(     "(A,B);\n('x;y',z);", (vec!["A", "B"],   vec![ParseWarning::SeveralTrees]))]
  #[case::unterminated_trailing_comment("(A,B);[x;",          (vec!["A", "B"],   vec![]))]
  #[trace]
  fn parse_first_ends_the_tree_at_a_semicolon_outside_quotes_and_comments(
    #[case] text: &str,
    #[case] (leaves, warnings): (Vec<&str>, Vec<ParseWarning>),
  ) {
    let parsed = parse_first(text, "t").unwrap();
    assert_eq!(leaves, parsed.tree.leaf_names());
    assert_eq!(warnings, parsed.warnings);
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::one_tree(              "(A,B);\n",         false)]
  #[case::two_trees(             "(A,B);(C,D);",      true)]
  #[case::broken_first_tree(     "(A,,;(C,D);",       true)]
  #[case::semicolons_in_quotes(  "('a;b',C);[;]",     false)]
  #[case::no_semicolon(          "(A,B)",             false)]
  #[trace]
  fn holds_several_trees_counts_semicolons_outside_quotes_and_comments(#[case] text: &str, #[case] expected: bool) {
    assert_eq!(expected, holds_several_trees(text));
  }

  #[test]
  fn parse_first_without_a_semicolon_outside_quotes_fails() {
    let e = parse_first("('a;b',C)", "t").unwrap_err();
    assert_eq!("Newick parse error: no ';' found", e.to_string());
  }

  #[test]
  fn warnings_keep_the_log_lines_of_the_command_line() {
    assert_eq!(
      "more than one tree in file, using the first",
      ParseWarning::SeveralTrees.to_string()
    );
    assert_eq!(
      "ignoring invalid branch length '0.R'",
      ParseWarning::InvalidLength("0.R".into()).to_string()
    );
  }

  #[test]
  fn duplicate_leaves_rejected() {
    parse("(A,A);", "t").unwrap_err();
    parse("(A,B", "t").unwrap_err();
  }

  #[test]
  fn syntax_error_has_its_byte_offset() {
    let e = parse("((A,B)C;", "t").unwrap_err();
    assert_eq!(Some(7), e.offset);
    assert_eq!("Newick parse error: expected ',' or ')' at byte 7", e.to_string());
  }

  #[test]
  fn tree_error_has_no_offset() {
    let e = parse("(A,A);", "t").unwrap_err();
    assert_eq!(None, e.offset);
    assert_eq!("Newick parse error: duplicate leaf name A", e.to_string());
  }

  #[test]
  fn line_column_counts_lines_and_characters() {
    assert_eq!((1, 1), line_column("(A,B);", 0));
    assert_eq!((1, 4), line_column("(A,B);", 3));
    assert_eq!((2, 1), line_column("(A,\nB);", 4));
    assert_eq!((2, 3), line_column("(A,\r\nB);", 7));
    // "é" and "ü" are two bytes each but one column each.
    assert_eq!((1, 4), line_column("(é,ü);", 4));
    assert_eq!((1, 6), line_column("(é,ü);", 7));
  }

  #[test]
  fn line_column_clamps_to_the_end() {
    assert_eq!((2, 3), line_column("(A,\nB)", 100));
  }

  #[test]
  fn error_offset_maps_to_line_and_column() {
    let text = "(A,\n(B,C)D\n;";
    let e = parse_first(text, "t").unwrap_err();
    assert_eq!((3, 1), line_column(text, e.offset.unwrap()));
  }
}
