//! Writing links: the query text of key-value pairs, the link of the web app, and the canonical
//! link of a request.

use crate::analysis::{self, AnalysisRequest, MIN_TREES, Settings};
use crate::examples;
use crate::launch::{FETCH_TIMEOUT_SECONDS, MAX_DOWNLOAD_BYTES, split_label, url_place};
use crate::output;
use crate::schema::{KeyValue, SETTING_KEYS, SettingKey, SettingName};
use crate::wire::wire_name;
use base64::Engine;
use base64::engine::general_purpose;
use flate2::Compression;
use flate2::write::GzEncoder;
use percent_encoding::{AsciiSet, CONTROLS, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use std::io::Write;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// Start of an inline session: a gzip-compressed session file in base64, as a `data:` location.
const INLINE_SESSION_PREFIX: &str = "data:application/gzip;base64,";

/// Longest link to write: Chrome's address bar shows at most 32 kB of a URL.
pub const MAX_LINK_CHARS: usize = 32_000;

/// Link length above which chat apps (Slack allows 4,000 characters per message), link
/// shorteners (Bitly allows 2,048), and QR codes may break a link.
const LONG_LINK_CHARS: usize = 2_000;

/// The characters of a value of [`link_query`] that a reader of form data would misread.
const VALUE_SET: &AsciiSet = &CONTROLS
  .add(b'%')
  .add(b'&')
  .add(b'#')
  .add(b'+')
  .add(b' ')
  .add(b'"')
  .add(b'<')
  .add(b'>');

/// The characters of a key of [`link_query`] that a reader would misread: those of values and `=`.
const KEY_SET: &AsciiSet = &VALUE_SET.add(b'=');

/// The key-value pairs of the canonical link of `request`, whose trees have the `addresses`, or
/// `None` when the request has fewer than two trees, which no link holds, or a tree has no
/// address, or a label or setting cannot be written: input keys, then
/// the settings that differ from `base` in the order of [`SETTING_KEYS`], then `run`. The input
/// is `example=<id>` when the trees are the trees of that example in its order with its labels,
/// and one `tree` per tree otherwise, with `<label>=` only where the label differs from the one
/// that [`parse_launch`](crate::launch::parse_launch) gives the location. A flag is a pair with an empty value.
pub fn launch_pairs(
  request: &AnalysisRequest,
  addresses: &[Option<TreeAddress>],
  base: &Settings,
  run: bool,
) -> Option<Vec<(String, String)>> {
  if addresses.len() != request.trees.len() || request.trees.len() < MIN_TREES {
    return None;
  }
  let mut pairs = match example_input(request, addresses) {
    Some(id) => vec![("example".to_owned(), id.to_owned())],
    None => tree_pairs(request, addresses)?,
  };
  pairs.extend(setting_pairs(&request.settings, base)?);
  if run {
    pairs.push(("run".to_owned(), String::new()));
  }
  Some(pairs)
}

/// The inline session of `request`: its session file, gzip-compressed and base64-encoded, as a
/// `data:` location for the fragment of a link. A session compresses better than its trees one
/// by one, because the trees share their leaf names.
pub fn inline_session(request: &AnalysisRequest) -> String {
  let text = output::session_file(request).text;
  let mut encoder = GzEncoder::new(Vec::new(), Compression::best());
  #[expect(clippy::expect_used, reason = "writing to a Vec cannot fail")]
  encoder
    .write_all(text.as_bytes())
    .expect("compressing into memory succeeds");
  #[expect(clippy::expect_used, reason = "writing to a Vec cannot fail")]
  let bytes = encoder.finish().expect("compressing into memory succeeds");
  format!("{INLINE_SESSION_PREFIX}{}", general_purpose::STANDARD.encode(bytes))
}

/// The link of the web app with the query `query` and the fragment `fragment`, each written by
/// [`link_query`] and left out when empty.
pub fn web_link(query: &[(String, String)], fragment: &[(String, String)]) -> String {
  let mut link = web_app_url();
  if !query.is_empty() {
    link.push('?');
    link.push_str(&link_query(query));
  }
  if !fragment.is_empty() {
    link.push('#');
    link.push_str(&link_query(fragment));
  }
  link
}

/// The query or fragment text of `pairs` (without `?` or `#`): pairs joined by `&`, a flag (an
/// empty value) without `=`. Only what a reader would misread is percent-encoded: `%`, `&`, `#`,
/// `+`, `=` in keys, spaces (`%20`), `"`, `<`, `>`, control characters, and non-ASCII characters;
/// `:`, `/`, `,`, `;`, `@`, `(`, `)`, and the other characters stay readable. [`query_pairs`]
/// reads it back.
pub fn link_query(pairs: &[(String, String)]) -> String {
  pairs
    .iter()
    .map(|(key, value)| {
      let key = utf8_percent_encode(key, KEY_SET);
      if value.is_empty() {
        key.to_string()
      } else {
        format!("{key}={}", utf8_percent_encode(value, VALUE_SET))
      }
    })
    .collect::<Vec<_>>()
    .join("&")
}

/// The key-value pairs of a link `url`: those of its query, then those of its fragment when the
/// fragment holds a `=` (a fragment without one is an anchor, such as `#help-cite`).
pub fn link_pairs(url: &str) -> Vec<(String, String)> {
  let (rest, fragment) = url.split_once('#').unwrap_or((url, ""));
  let query = rest.split_once('?').map_or("", |(_, q)| q);
  let mut pairs = query_pairs(query);
  if fragment.contains('=') {
    pairs.extend(query_pairs(fragment));
  }
  pairs
}

/// The key-value pairs of a query or fragment text, read as browsers read form data
/// (`URLSearchParams`): pairs split at `&`, a key without `=` has an empty value, `+` is a space,
/// and percent-encoded bytes are decoded as UTF-8.
pub fn query_pairs(query: &str) -> Vec<(String, String)> {
  form_urlencoded::parse(query.as_bytes()).into_owned().collect()
}

/// The limits of downloads and links, for the web app.
pub fn link_limits() -> LinkLimits {
  LinkLimits {
    fetch_timeout_seconds: FETCH_TIMEOUT_SECONDS,
    max_download_bytes: MAX_DOWNLOAD_BYTES,
    max_link_chars: MAX_LINK_CHARS,
    long_link_chars: LONG_LINK_CHARS,
  }
}

/// Address of the web app: the GitHub Pages site of the repository of the workspace, such as
/// `https://neherlab.github.io/treeknit/` for `https://github.com/neherlab/treeknit`.
fn web_app_url() -> String {
  let repository = env!("CARGO_PKG_REPOSITORY").trim_end_matches('/');
  match repository
    .strip_prefix("https://github.com/")
    .and_then(|path| path.split_once('/'))
  {
    Some((owner, repo)) => format!("https://{}.github.io/{repo}/", owner.to_ascii_lowercase()),
    None => format!("{repository}/"),
  }
}

/// The id of the example whose trees `request` has in its order with its labels.
fn example_input<'a>(request: &AnalysisRequest, addresses: &'a [Option<TreeAddress>]) -> Option<&'a str> {
  let Some(Some(TreeAddress::Example { id, .. })) = addresses.first() else {
    return None;
  };
  let example = examples::example(id)?;
  let files_match = example.trees.len() == addresses.len()
    && example
      .trees
      .iter()
      .zip(addresses)
      .all(|(t, a)| matches!(a, Some(TreeAddress::Example { id: other, file }) if other == id && file == t.file));
  (files_match && example.labels() == analysis::labels(&request.trees)).then_some(id.as_str())
}

/// One `tree` pair per tree, or `None` when a tree has no location or a label cannot be written.
fn tree_pairs(request: &AnalysisRequest, addresses: &[Option<TreeAddress>]) -> Option<Vec<(String, String)>> {
  let locations: Vec<(String, String)> = request
    .trees
    .iter()
    .zip(addresses)
    .map(|(tree, address)| match address {
      Some(TreeAddress::Url { url }) => Some((url.clone(), url_place(url).file_name)),
      Some(TreeAddress::Data) => Some((format!("data:,{}", tree.newick.replace('%', "%25")), String::new())),
      Some(TreeAddress::Example { .. }) | None => None,
    })
    .collect::<Option<_>>()?;
  let labels = analysis::labels(&request.trees);
  let explicit = explicit_labels(&labels, &locations);
  labels
    .iter()
    .zip(&locations)
    .zip(explicit)
    .map(|((label, (location, _)), explicit)| {
      let value = if explicit {
        if split_label(&format!("{label}=x")).label != Some(label.as_str()) {
          return None;
        }
        format!("{label}={location}")
      } else {
        location.clone()
      };
      Some(("tree".to_owned(), value))
    })
    .collect()
}

/// Which trees need an explicit label: the fewest trees such that [`parse_launch`](crate::launch::parse_launch) gives the
/// others their labels. A tree whose automatic label differs needs one, which can change the
/// automatic labels of the others, so the set grows until no automatic label differs.
fn explicit_labels(labels: &[String], locations: &[(String, String)]) -> Vec<bool> {
  let mut explicit = vec![false; labels.len()];
  loop {
    let given: Vec<String> = labels
      .iter()
      .zip(&explicit)
      .filter(|(_, e)| **e)
      .map(|(l, _)| l.clone())
      .collect();
    let names: Vec<String> = locations
      .iter()
      .zip(&explicit)
      .filter(|(_, e)| !**e)
      .map(|((_, name), _)| name.clone())
      .collect();
    let mut automatic = analysis::tree_labels(&names, &given).into_iter();
    let mut changed = false;
    for (label, e) in labels.iter().zip(explicit.iter_mut()) {
      if !*e && automatic.next().as_ref() != Some(label) {
        *e = true;
        changed = true;
      }
    }
    if !changed {
      return explicit;
    }
  }
}

/// The setting pairs of `settings` that differ from `base`, or `None` for a difference that a link
/// cannot write (sequence lengths in `base` but not in `settings`).
fn setting_pairs(settings: &Settings, base: &Settings) -> Option<Vec<(String, String)>> {
  let mut pairs = Vec::new();
  for s in &SETTING_KEYS {
    let value = match s.setting {
      SettingName::Gamma => changed(&settings.gamma, &base.gamma).map(f64::to_string),
      SettingName::SeqLengths => match (&settings.seq_lengths, &base.seq_lengths) {
        (a, b) if a == b => None,
        (Some(v), _) if !v.is_empty() => Some(v.iter().map(f64::to_string).collect::<Vec<_>>().join(",")),
        _ => return None,
      },
      SettingName::NMcmcIt => changed(&settings.n_mcmc_it, &base.n_mcmc_it).map(u64::to_string),
      SettingName::Resolve => changed(&settings.resolve, &base.resolve).map(wire_name),
      SettingName::Rounds => changed(&settings.rounds, &base.rounds).map(u64::to_string),
      SettingName::Seed => changed(&settings.seed, &base.seed).map(u64::to_string),
      SettingName::PreResolve => flag_pair(s, settings.pre_resolve, base.pre_resolve),
      SettingName::FinalRound => flag_pair(s, settings.final_round, base.final_round),
      SettingName::Likelihood => flag_pair(s, settings.likelihood, base.likelihood),
      SettingName::Naive => flag_pair(s, settings.naive, base.naive),
    };
    if let Some(value) = value {
      pairs.push(value_pair(s, value));
    }
  }
  Some(pairs)
}

/// The pair of a setting: a flag key with an empty value as its own key (`no-final-round`), so
/// `value` is the key; other settings their key and `value`.
fn value_pair(s: &SettingKey, value: String) -> (String, String) {
  match s.value {
    KeyValue::Flag { .. } => (value, String::new()),
    KeyValue::Number | KeyValue::Numbers | KeyValue::Integer | KeyValue::Mode => (s.key.to_owned(), value),
  }
}

/// The key of a flag that differs from `base`: the key when it sets `value`, its opposite
/// otherwise.
fn flag_pair(s: &SettingKey, value: bool, base: bool) -> Option<String> {
  let KeyValue::Flag { key_sets, opposite } = s.value else {
    return None;
  };
  (value != base).then(|| {
    if value == key_sets {
      s.key.to_owned()
    } else {
      opposite.to_owned()
    }
  })
}

/// `value` when it differs from `base`.
fn changed<'a, T: PartialEq>(value: &'a T, base: &T) -> Option<&'a T> {
  (value != base).then_some(value)
}

/// A key-value pair of a link, as the web app passes it; a flag has an empty value.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct LinkEntry {
  key: String,
  value: String,
}

impl LinkEntry {
  /// The pairs of `entries`.
  pub fn pairs(entries: &[LinkEntry]) -> Vec<(String, String)> {
    entries.iter().map(|e| (e.key.clone(), e.value.clone())).collect()
  }

  /// The entries of `pairs`.
  pub fn of_pairs(pairs: Vec<(String, String)>) -> Vec<LinkEntry> {
    pairs.into_iter().map(|(key, value)| LinkEntry { key, value }).collect()
  }
}

/// What the web app writes a link of: a request, where a link gets each of its trees again, and
/// whether the link runs the analysis (see [`launch_pairs`]).
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct LinkSource {
  pub request: AnalysisRequest,
  /// One address per tree; `None` for a tree that a link cannot get again.
  pub addresses: Vec<Option<TreeAddress>>,
  pub run: bool,
}

/// The limits of downloads and links; see the constants of the same names.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct LinkLimits {
  fetch_timeout_seconds: u32,
  max_download_bytes: usize,
  max_link_chars: usize,
  long_link_chars: usize,
}

/// Where a link can get a tree of the workspace again.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum TreeAddress {
  /// The tree file `file` of the example `id`.
  Example { id: String, file: String },
  /// An `https:` URL as a link gave it.
  Url { url: String },
  /// A `data:` text of a link: the tree's own Newick text.
  Data,
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::analysis::{ResolveMode, TreeText, ValidationError};
  use crate::launch::{LaunchInput, LinkLocation, apply, parse_launch, parse_location};
  use pretty_assertions::assert_eq;
  use rand::{Rng, SeedableRng};
  use rand_xoshiro::Xoshiro256PlusPlus;
  use rstest::rstest;
  use std::iter;

  #[rustfmt::skip]
  #[rstest]
  #[case::leaf(      &[("leaf", "A/New York/392/2004")],         "leaf=A/New%20York/392/2004")]
  #[case::auspice(   &[("auspice", "legend=open&l=radial")],     "auspice=legend=open%26l=radial")]
  #[case::flag(      &[("example", "5-leaves"), ("run", "")],   "example=5-leaves&run")]
  #[case::readable(  &[("tree", "ha=https://x/a.nwk?b=c")],      "tree=ha=https://x/a.nwk?b=c")]
  #[case::plus_hash( &[("s", "a+b#c%d")],                       "s=a%2Bb%23c%25d")]
  #[case::unicode(   &[("leaf", "Ä")],                           "leaf=%C3%84")]
  #[case::key_equals(&[("a=b", "c")],                            "a%3Db=c")]
  #[trace]
  fn link_query_encodes_only_what_a_reader_misreads(#[case] given: &[(&str, &str)], #[case] expected: &str) {
    let given: Vec<(String, String)> = given.iter().map(|(k, v)| ((*k).to_owned(), (*v).to_owned())).collect();
    assert_eq!(expected, link_query(&given));
    assert_eq!(given, query_pairs(expected));
  }

  #[test]
  fn query_pairs_read_like_browsers() {
    let expected: Vec<(String, String)> = [("a", "x y"), ("b", "+"), ("run", ""), ("c", "1=2")]
      .iter()
      .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
      .collect();
    assert_eq!(expected, query_pairs("a=x+y&&b=%2B&run&c=1=2"));
  }

  #[test]
  fn link_pairs_read_the_query_and_a_fragment_with_keys() {
    let read = |u: &str| link_pairs(u).into_iter().map(|(k, _)| k).collect::<Vec<_>>();
    assert_eq!(
      vec!["run", "view", "session"],
      read("https://h/p/?run&view=mccs#session=data:,x")
    );
    assert_eq!(vec!["example"], read("https://h/p/help?example=a#help-cite"));
    assert_eq!(Vec::<String>::new(), read("https://h/p/"));
  }

  #[test]
  fn web_app_url_is_the_pages_site_of_the_repository() {
    assert_eq!("https://neherlab.github.io/treeknit/", web_app_url());
  }

  fn request(labels: &[&str], settings: Settings) -> AnalysisRequest {
    AnalysisRequest {
      trees: labels.iter().map(|l| TreeText::new(*l, "((A,B),C);")).collect(),
      settings,
    }
  }

  fn example_addresses(id: &str) -> Vec<Option<TreeAddress>> {
    examples::example(id)
      .unwrap()
      .trees
      .iter()
      .map(|t| {
        Some(TreeAddress::Example {
          id: id.to_owned(),
          file: t.file.to_owned(),
        })
      })
      .collect()
  }

  fn written(request: &AnalysisRequest, addresses: &[Option<TreeAddress>], run: bool) -> Option<String> {
    launch_pairs(request, addresses, &Settings::default(), run).map(|p| link_query(&p))
  }

  #[test]
  fn an_example_is_written_by_its_id() {
    let r = request(
      &["ha", "na"],
      Settings {
        gamma: 3.0,
        final_round: false,
        ..Settings::default()
      },
    );
    assert_eq!(
      Some("example=h3n2-2017&gamma=3&no-final-round&run".to_owned()),
      written(&r, &example_addresses("h3n2-2017"), true)
    );
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::renamed(  &["HA", "na"])]
  #[case::reordered(&["na", "ha"])]
  #[trace]
  fn example_trees_that_differ_from_the_example_have_no_link(#[case] labels: &[&str]) {
    let mut addresses = example_addresses("h3n2-2017");
    if labels[0] == "na" {
      addresses.reverse();
    }
    assert_eq!(None, written(&request(labels, Settings::default()), &addresses, false));
  }

  #[test]
  fn url_trees_get_a_label_only_where_it_differs_from_the_file_name() {
    let addresses = [
      Some(TreeAddress::Url {
        url: "https://x/seg4.nwk".to_owned(),
      }),
      Some(TreeAddress::Url {
        url: "https://x/na.nwk".to_owned(),
      }),
      Some(TreeAddress::Data),
    ];
    let r = request(
      &["HA", "na", "tree"],
      Settings {
        seq_lengths: Some(vec![1701.0, 1410.0, 0.5]),
        ..Settings::default()
      },
    );
    assert_eq!(
      Some(
        "tree=HA=https://x/seg4.nwk&tree=https://x/na.nwk&tree=data:,((A,B),C);&seq-lengths=1701,1410,0.5".to_owned()
      ),
      written(&r, &addresses, false)
    );
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::no_tree( &[],     &[])]
  #[case::one_tree(&["ha"], &[Some(TreeAddress::Data)])]
  #[trace]
  fn fewer_than_two_trees_have_no_link_even_with_settings(#[case] labels: &[&str], #[case] addresses: &[Option<TreeAddress>]) {
    // A link with settings and no input would fail to read: "gamma needs an input".
    let r = request(labels, Settings { gamma: 3.0, ..Settings::default() });
    assert_eq!(None, written(&r, addresses, false));
  }

  #[test]
  fn a_tree_without_an_address_or_a_label_that_a_link_cannot_hold_has_no_link() {
    let url = Some(TreeAddress::Url {
      url: "https://x/a.nwk".to_owned(),
    });
    assert_eq!(
      None,
      written(&request(&["a", "b"], Settings::default()), &[url.clone(), None], false)
    );
    assert_eq!(
      None,
      written(&request(&["a", "b=c"], Settings::default()), &[url.clone(), url], false)
    );
  }

  #[test]
  fn flags_are_written_in_the_polarity_that_differs_from_the_base() {
    let base = Settings {
      pre_resolve: true,
      naive: true,
      ..Settings::default()
    };
    let r = request(
      &["ha", "na"],
      Settings {
        pre_resolve: false,
        naive: true,
        likelihood: false,
        ..Settings::default()
      },
    );
    let pairs = launch_pairs(&r, &example_addresses("h3n2-2017"), &base, false).unwrap();
    assert_eq!("example=h3n2-2017&no-pre-resolve&no-likelihood", link_query(&pairs));
  }

  #[test]
  fn inline_session_reads_back_to_the_request() {
    let r = request(
      &["ha", "na"],
      Settings {
        seed: 7,
        ..Settings::default()
      },
    );
    let Ok(LinkLocation::Data { text }) = parse_location(&inline_session(&r)) else {
      panic!("the inline session is a data: location");
    };
    assert_eq!(Ok(r), analysis::read_session(&text));
  }

  #[test]
  fn inline_session_is_shorter_than_its_session_file() {
    let newick = "((A/New York/392/2004:0.1,A/New York/393/2004:0.2):0.05,A/Hong Kong/1/2003:0.3);";
    let r = AnalysisRequest {
      trees: ["ha", "na"]
        .iter()
        .map(|l| TreeText::new(*l, newick.repeat(20)))
        .collect(),
      settings: Settings::default(),
    };
    assert!(inline_session(&r).len() * 4 < output::session_file(&r).text.len());
  }

  /// A random label that the label check accepts, sometimes the label that a file name gives.
  fn random_label(rng: &mut Xoshiro256PlusPlus) -> String {
    const CHARS: &[char] = &['a', 'b', 'H', 'A', '_', '.', ' ', 'é', '=', '2', '%', '&', '+', '#'];
    let len = rng.gen_range(1..5);
    iter::repeat_with(|| CHARS[rng.gen_range(0..CHARS.len())])
      .take(len)
      .collect()
  }

  fn random_settings(rng: &mut Xoshiro256PlusPlus, k: usize) -> Settings {
    let d = Settings::default();
    let maybe = |rng: &mut Xoshiro256PlusPlus| rng.gen_bool(0.5);
    let modes = [
      ResolveMode::None,
      ResolveMode::Strict,
      ResolveMode::Liberal,
      ResolveMode::Matched,
    ];
    Settings {
      gamma: if maybe(rng) { rng.gen_range(-1.0..10.0) } else { d.gamma },
      seq_lengths: maybe(rng).then(|| iter::repeat_with(|| rng.gen_range(0.0..3000.0)).take(k).collect()),
      n_mcmc_it: if maybe(rng) {
        rng.gen_range(0..1000)
      } else {
        d.n_mcmc_it
      },
      resolve: modes[rng.gen_range(0..4)],
      pre_resolve: maybe(rng),
      rounds: if maybe(rng) { rng.gen_range(0..5) } else { d.rounds },
      final_round: maybe(rng),
      likelihood: maybe(rng),
      naive: maybe(rng),
      seed: if maybe(rng) {
        rng.gen_range(0..=analysis::MAX_SEED)
      } else {
        d.seed
      },
    }
  }

  /// A random request with addresses: an example, or URL and data trees.
  fn random_request(rng: &mut Xoshiro256PlusPlus) -> (AnalysisRequest, Vec<Option<TreeAddress>>) {
    if rng.gen_bool(0.2) {
      let e = &examples::EXAMPLES[rng.gen_range(0..examples::EXAMPLES.len())];
      let r = request(
        &e.labels().iter().map(String::as_str).collect::<Vec<_>>(),
        random_settings(rng, e.trees.len()),
      );
      return (r, example_addresses(e.id));
    }
    let k = rng.gen_range(2..5);
    let names = ["ha.nwk", "ha", "na.nwk", "a b.tree", "%C3%A9.nwk", "", "seg:4.nwk"];
    let addresses: Vec<Option<TreeAddress>> = iter::repeat_with(|| {
      if rng.gen_bool(0.2) {
        Some(TreeAddress::Data)
      } else {
        let name = names[rng.gen_range(0..names.len())];
        Some(TreeAddress::Url {
          url: format!("https://example.org/d/{name}"),
        })
      }
    })
    .take(k)
    .collect();
    let defaults: Vec<String> = analysis::tree_labels(
      &addresses
        .iter()
        .map(|a| match a {
          Some(TreeAddress::Url { url }) => url_place(url).file_name,
          _ => String::new(),
        })
        .collect::<Vec<_>>(),
      &[],
    );
    let mut labels: Vec<String> = Vec::new();
    for default in defaults {
      let mut label = if rng.gen_bool(0.5) { default } else { random_label(rng) };
      while labels
        .iter()
        .any(|l| analysis::label_key(l) == analysis::label_key(&label))
      {
        label.push('x');
      }
      labels.push(label);
    }
    let r = AnalysisRequest {
      trees: labels
        .into_iter()
        .map(|label| TreeText::new(label, "((A %,B),(C,D));"))
        .collect(),
      settings: random_settings(rng, k),
    };
    (r, addresses)
  }

  /// The request and addresses that the launch of `pairs` describes, with the trees of
  /// `original` for the texts that a link does not hold.
  fn read_back(pairs: &[(String, String)], original: &AnalysisRequest) -> (AnalysisRequest, Vec<Option<TreeAddress>>) {
    let parsed = parse_launch(pairs, &[]);
    assert_eq!(Vec::<ValidationError>::new(), parsed.errors);
    let launch = parsed.launch.unwrap();
    let settings = apply(&Settings::default(), &launch.settings);
    let (trees, addresses) = match launch.input {
      LaunchInput::Example { id } => {
        let e = examples::example(&id).unwrap();
        let trees = e
          .labels()
          .into_iter()
          .zip(&original.trees)
          .map(|(label, t)| TreeText::new(label, t.newick.clone()))
          .collect();
        (trees, example_addresses(&id))
      },
      LaunchInput::Trees { trees } => trees
        .into_iter()
        .zip(&original.trees)
        .map(|(t, o)| match t.location {
          LinkLocation::Url { url, .. } => (TreeText::new(t.label, o.newick.clone()), Some(TreeAddress::Url { url })),
          LinkLocation::Data { text } => (TreeText::new(t.label, text), Some(TreeAddress::Data)),
        })
        .unzip(),
      #[cfg_attr(
        dylint_lib = "mordant",
        expect(wildcard_over_own_enum, reason = "the test fails on every other variant")
      )]
      other => panic!("a written link names an example or trees, got {other:?}"),
    };
    (AnalysisRequest { trees, settings }, addresses)
  }

  #[test]
  fn written_links_read_back_to_the_request() {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(20261006);
    let mut written_count = 0;
    // The link pairs of each request that does not read back to itself.
    let mut failures: Vec<Vec<(String, String)>> = Vec::new();
    for _ in 0..2000 {
      let (r, addresses) = random_request(&mut rng);
      let Some(pairs) = launch_pairs(&r, &addresses, &Settings::default(), false) else {
        continue;
      };
      written_count += 1;
      let through_text = query_pairs(&link_query(&pairs));
      let (back, back_addresses) = read_back(&through_text, &r);
      let rewritten = launch_pairs(&back, &back_addresses, &Settings::default(), false);
      if through_text != pairs || back != r || back_addresses != addresses || rewritten.as_ref() != Some(&pairs) {
        failures.push(pairs);
      }
    }
    // More than half of the random requests have a link, so the round trip is tested broadly.
    assert_eq!(
      (Vec::<Vec<(String, String)>>::new(), true),
      (failures, written_count > 1000)
    );
  }

  #[test]
  fn link_query_round_trips_arbitrary_text() {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(7);
    let chars = [
      'a',
      '=',
      '&',
      '#',
      '+',
      '%',
      ' ',
      '/',
      ':',
      'é',
      '\u{1F600}',
      '\n',
      '?',
      ',',
      '"',
      '2',
    ];
    let text = |rng: &mut Xoshiro256PlusPlus| -> String {
      let len = rng.gen_range(1..8);
      iter::repeat_with(|| chars[rng.gen_range(0..chars.len())])
        .take(len)
        .collect()
    };
    let changed: Vec<Vec<(String, String)>> = iter::repeat_with(|| {
      let n = rng.gen_range(1..4);
      iter::repeat_with(|| (text(&mut rng), text(&mut rng))).take(n).collect()
    })
    .take(2000)
    .filter(|pairs: &Vec<(String, String)>| *pairs != query_pairs(&link_query(pairs)))
    .collect();
    assert_eq!(Vec::<Vec<(String, String)>>::new(), changed);
  }

  #[test]
  fn inline_sessions_of_random_requests_read_back() {
    let mut rng = Xoshiro256PlusPlus::seed_from_u64(11);
    let changed: Vec<AnalysisRequest> = iter::repeat_with(|| random_request(&mut rng).0)
      .take(200)
      .filter(|r| {
        let Ok(LinkLocation::Data { text }) = parse_location(&inline_session(r)) else {
          panic!("the inline session is a data: location");
        };
        analysis::read_session(&text).as_ref() != Ok(r)
      })
      .collect();
    assert_eq!(Vec::<AnalysisRequest>::new(), changed);
  }
}
