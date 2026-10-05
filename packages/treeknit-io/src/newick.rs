//! Newick reading and writing.
//!
//! Supported: quoted labels (`'a b'`, with `''` for a quote), comments in square brackets
//! (skipped), branch lengths, whitespace. Unnamed or numeric (support value) internal
//! nodes and duplicate internal names are renamed `NODE_i`.

use std::collections::HashSet;
use std::fmt::Write;
use treeknit_core::{NodeId, Tree};

#[derive(Debug)]
pub struct ParseError(pub String);

impl std::fmt::Display for ParseError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "Newick parse error: {}", self.0)
  }
}
impl std::error::Error for ParseError {}

struct Parser<'a> {
  s: &'a [u8],
  i: usize,
}

impl Parser<'_> {
  fn err<T>(&self, msg: &str) -> Result<T, ParseError> {
    Err(ParseError(format!("{msg} at byte {}", self.i)))
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
      return String::from_utf8(out).map_err(|e| ParseError(e.to_string()));
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
      log::warn!("ignoring invalid branch length '{txt}'");
      Ok(None)
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

/// Parse a single Newick tree.
pub fn parse(s: &str, label: &str) -> Result<Tree, ParseError> {
  let mut p = Parser { s: s.as_bytes(), i: 0 };
  let mut t = Tree::new(label);
  let root = t.root;
  p.subtree(&mut t, root)?;
  p.skip()?;
  if p.peek() != Some(b';') {
    return p.err("expected ';'");
  }
  t.nodes[root].branch_length = None;
  fix_names(&mut t)?;
  Ok(t)
}

/// Parse the first tree of a Newick file's content.
pub fn parse_first(content: &str, label: &str) -> Result<Tree, ParseError> {
  let end = content.find(';').ok_or_else(|| ParseError("no ';' found".into()))?;
  if content[end + 1..].contains(';') {
    log::warn!("{label}: more than one tree in file, using the first");
  }
  parse(&content[..=end], label)
}

fn fix_names(t: &mut Tree) -> Result<(), ParseError> {
  let mut leaves = HashSet::new();
  for n in t.leaves() {
    let name = &t.nodes[n].name;
    if name.is_empty() {
      return Err(ParseError(format!("tree {}: unnamed leaf", t.label)));
    }
    if !leaves.insert(name.clone()) {
      return Err(ParseError(format!("tree {}: duplicate leaf name {name}", t.label)));
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
  let mut s = String::new();
  // Iterative writer to avoid deep recursion on ladder-like trees.
  enum Step {
    Enter(NodeId),
    Exit(NodeId),
    Comma,
  }
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
  fn duplicate_leaves_rejected() {
    parse("(A,A);", "t").unwrap_err();
    parse("(A,B", "t").unwrap_err();
  }
}
