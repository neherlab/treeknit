//! Tables of the link grammar, as JSON: key-value pairs with the query text that `link_query`
//! writes for them, query texts with the pairs that `query_pairs` reads from them, and links with
//! the pairs that `link_pairs` reads from their query and fragment.
//!
//! The web app reads and writes the query of its address itself, because its router does so
//! synchronously on every navigation; its tests check that copy against this table. `just gen`
//! writes it to `packages/web/src/workspace/__tests__/__fixtures__/link_cases.json`, and
//! `just generated-check` fails when the committed file differs from a fresh run.
//!
//! The pairs hold every character that the grammar escapes (`#`, `+`, `=` in keys, `&`, `%`,
//! space, quotes, `<`, `>`, control characters, non-ASCII text), characters that it keeps
//! readable, empty values, and repeated keys.

use serde::Serialize;
use std::process::ExitCode;
use treeknit_io::launch::{link_pairs, link_query, query_pairs};

/// Key-value pairs to write, one list per query.
const WRITTEN: &[&[(&str, &str)]] = &[
  &[
    ("tree", "https://x.org/a/ha.nwk"),
    ("tree", "na=https://x.org/b/na.nwk"),
  ],
  &[("run", ""), ("view", "mccs")],
  &[("label", "a#b+c&d%e f")],
  &[("a=b", "c=d")],
  &[("quote", "\"<x>\""), ("tick", "'")],
  &[("readable", ":/,;@()!*~$")],
  &[("control", "a\u{1}b\u{1f}c\u{7f}d")],
  &[("text", "é名🧬"), ("名", "ö")],
  &[("empty", ""), ("empty", ""), ("empty", "x")],
  &[("", "no key")],
];

/// Query texts to read: as links write them, and as people and other sites write them.
const READ: &[&str] = &[
  "a=x+y&&b=%2B&run&c=1=2",
  "tree=ha%3Dx&tree=na",
  "a=%E2%82%AC&b=%FF&c=%ZZ&d=%",
  "=v&k=",
  "",
  "&&",
];

/// Links whose query and fragment `link_pairs` reads.
const LINKS: &[&str] = &[
  "https://h/p/?run&view=mccs#session=data:,x",
  "https://h/p/help?example=a#help-cite",
  "https://h/p/",
  "https://h/p/?a=1#b=2&c",
  "https://h/p/#?a=1",
  "https://h/p/?a=%23#b=%26",
];

fn main() -> ExitCode {
  let written = WRITTEN
    .iter()
    .map(|pairs| {
      let pairs = owned(pairs);
      let query = link_query(&pairs);
      WrittenCase {
        read_back: query_pairs(&query),
        pairs,
        query,
      }
    })
    .collect();
  let read = READ
    .iter()
    .map(|&query| ReadCase {
      query: query.to_owned(),
      pairs: query_pairs(query),
    })
    .collect();
  let links = LINKS
    .iter()
    .map(|&url| LinkCase {
      url: url.to_owned(),
      pairs: link_pairs(url),
    })
    .collect();
  match serde_json::to_string_pretty(&Table { written, read, links }) {
    Ok(json) => {
      println!("{json}");
      ExitCode::SUCCESS
    },
    Err(e) => {
      eprintln!("link_cases: {e}");
      ExitCode::FAILURE
    },
  }
}

fn owned(pairs: &[(&str, &str)]) -> Vec<(String, String)> {
  pairs.iter().map(|&(k, v)| (k.to_owned(), v.to_owned())).collect()
}

#[derive(Serialize)]
struct Table {
  written: Vec<WrittenCase>,
  read: Vec<ReadCase>,
  links: Vec<LinkCase>,
}

/// The query that `link_query` writes for `pairs`, and the pairs that `query_pairs` reads back.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct WrittenCase {
  pairs: Vec<(String, String)>,
  query: String,
  read_back: Vec<(String, String)>,
}

/// The pairs that `query_pairs` reads from `query`.
#[derive(Serialize)]
struct ReadCase {
  query: String,
  pairs: Vec<(String, String)>,
}

/// The pairs that `link_pairs` reads from `url`.
#[derive(Serialize)]
struct LinkCase {
  url: String,
  pairs: Vec<(String, String)>,
}
