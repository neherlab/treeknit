//! Links of the web app that name trees, settings, and a run: the grammar of their keys, read
//! by the web app and the command line, and the canonical link of a request, written by both.
//!
//! A link carries key-value pairs in its query and, for inline data, in its fragment:
//!
//! ```text
//! ?example=h3n2-2017&gamma=3&run
//! ?tree=ha=https://example.org/ha.nwk&tree=na=https://example.org/na.nwk&resolve=strict
//! ?run#session=data:application/gzip;base64,H4sIAAAA...
//! ```
//!
//! The input keys (`example`, `tree`, `session`, `from`) name the trees, the setting keys of
//! [`SETTING_KEYS`] change the settings, `run` starts the run, and `v` is the version of the
//! format. The web app owns the keys of its display (`view`, `pair`, ...), which this module
//! leaves alone.

use crate::analysis::{self, AnalysisRequest, Field, ResolveMode, Settings, ValidationError};
use crate::examples;
use crate::output;
use crate::schema::{KeyValue, SETTING_KEYS, SettingKey, SettingName};
use crate::wire::{from_wire_name, wire_name};
use base64::Engine;
use base64::alphabet;
use base64::engine::general_purpose::{GeneralPurpose, GeneralPurposeConfig};
use base64::engine::{DecodePaddingMode, general_purpose};
use flate2::Compression;
use flate2::read::MultiGzDecoder;
use flate2::write::GzEncoder;
use percent_encoding::{AsciiSet, CONTROLS, percent_decode_str, utf8_percent_encode};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::io::{self, Read, Write};
use std::string::FromUtf8Error;
#[cfg(feature = "tsify")]
use tsify::Tsify;

/// The version of the link format that this build reads and writes. A link without `v` is
/// version 1, now and after later versions, so that published links keep their meaning.
pub const FORMAT_VERSION: u64 = 1;

/// Start of an inline session: a gzip-compressed session file in base64, as a `data:` location.
pub const INLINE_SESSION_PREFIX: &str = "data:application/gzip;base64,";

/// Largest text that a compressed tree or session file may expand to. A wrong link must not
/// exhaust the memory of the 32-bit WebAssembly module, and a Newick tree of 100,000 leaves is
/// about 5 MB.
pub const MAX_TEXT_BYTES: usize = MAX_TEXT_MIB * 1024 * 1024;

/// [`MAX_TEXT_BYTES`] in MiB.
const MAX_TEXT_MIB: usize = 256;

/// Longest time that reading a file of a link may take, in seconds, so that a wrong address
/// cannot hang the web app or the command line.
pub const FETCH_TIMEOUT_SECONDS: u32 = 60;

/// Largest file that a link may download, counted while it arrives, so that a wrong address
/// cannot exhaust the memory of a browser tab: a Newick tree of 100,000 leaves is about 5 MB.
pub const MAX_DOWNLOAD_BYTES: usize = 64 * 1024 * 1024;

/// Longest link to write: Chrome's address bar shows at most 32 kB of a URL.
pub const MAX_LINK_CHARS: usize = 32_000;

/// Link length above which chat apps (Slack allows 4,000 characters per message), link
/// shorteners (Bitly allows 2,048), and QR codes may break a link.
pub const LONG_LINK_CHARS: usize = 2_000;

/// Keys that name an input; a link has at most one kind of them.
const INPUT_KEYS: [&str; 4] = ["example", "tree", "session", "from"];

/// Shortest unknown key that gets a suggestion: shorter keys, such as Auspice's `c`, are within
/// edit distance 2 of too many keys.
const SUGGESTION_MIN_CHARS: usize = 4;

/// Largest edit distance between an unknown key and the key it suggests.
const SUGGESTION_MAX_DISTANCE: usize = 2;

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

/// The characters that a URI cannot hold (RFC 3986): every ASCII character outside its
/// unreserved and reserved sets and `%`.
const URI_SET: &AsciiSet = &CONTROLS
  .add(b' ')
  .add(b'"')
  .add(b'<')
  .add(b'>')
  .add(b'\\')
  .add(b'^')
  .add(b'`')
  .add(b'{')
  .add(b'|')
  .add(b'}');

/// Read the launch of the key-value pairs of a link, in the order of the link. `view_keys` are
/// the keys that the web app reads itself, which are neither errors nor ignored. Every problem is
/// an error at the key it concerns (`gamma`, `tree[1]`); a value of the right type outside the
/// bounds of a setting is no error here, so that the settings form shows it at its field.
pub fn parse_launch(pairs: &[(String, String)], view_keys: &[String]) -> LaunchParse {
  let mut reader = LaunchReader::default();
  let mut after_query_location = false;
  for (key, value) in pairs {
    let known = reader.read(key, value);
    if !known && !view_keys.contains(key) {
      reader.ignored.push(IgnoredKey {
        key: key.clone(),
        suggestion: suggestion(key, view_keys),
        after_location_query: after_query_location,
      });
    }
    after_query_location = matches!(key.as_str(), "tree" | "session") && value.contains('?');
  }
  reader.finish()
}

/// The label and the location of a tree token `[<label>=]<location>`: the text before the first
/// `=` is a label when it is not empty and holds no `:`, `/`, or `\`, which a label cannot hold
/// and every location before its first `=` does (`https://`, `data:`, a path).
pub fn split_label(token: &str) -> LabeledToken<'_> {
  match token.split_once('=') {
    Some((label, rest)) if !label.is_empty() && !label.contains([':', '/', '\\']) => LabeledToken {
      label: Some(label),
      rest,
    },
    _ => LabeledToken {
      label: None,
      rest: token,
    },
  }
}

/// A token split by [`split_label`].
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LabeledToken<'a> {
  pub label: Option<&'a str>,
  /// The location or path after the label.
  pub rest: &'a str,
}

/// The location of `value`: an `https:` URL, rewritten to the address that serves the file to
/// other sites (see [`fetch_url`]), or a `data:` text, decoded.
pub fn parse_location(value: &str) -> Result<LinkLocation, LocationError> {
  let scheme = value.split_once(':').map(|(s, _)| s.to_ascii_lowercase());
  match scheme.as_deref() {
    Some("https") => {
      check_https(value)?;
      Ok(LinkLocation::Url {
        url: value.to_owned(),
        fetch: fetch_url(value),
      })
    },
    Some("data") => Ok(LinkLocation::Data {
      text: decode_data_url(value)?,
    }),
    Some("http") => Err(LocationError::Http(value.to_owned())),
    _ => Err(LocationError::Unsupported(value.to_owned())),
  }
}

/// The address that serves the file of `url` to other sites, which browsers require: a GitHub
/// file page (`github.com/<owner>/<repo>/blob/<ref>/<path>` or `.../raw/...`) becomes its file
/// on `raw.githubusercontent.com`, and a Zenodo file (`zenodo.org/records/<id>/files/<name>`)
/// its `/content` address of the Zenodo API. Other URLs stay as they are.
pub fn fetch_url(url: &str) -> String {
  let Some(parts) = UrlParts::of(url) else {
    return url.to_owned();
  };
  let host = parts.host.to_ascii_lowercase();
  let segments: Vec<&str> = parts.path.split('/').filter(|s| !s.is_empty()).collect();
  match (host.as_str(), segments.as_slice()) {
    ("github.com" | "www.github.com", [owner, repo, "blob" | "raw", git_ref, path @ ..]) if !path.is_empty() => {
      format!(
        "https://raw.githubusercontent.com/{owner}/{repo}/{git_ref}/{}",
        path.join("/")
      )
    },
    ("zenodo.org" | "www.zenodo.org", ["records", id, "files", name @ ..]) if !name.is_empty() => {
      format!("https://zenodo.org/api/records/{id}/files/{}/content", name.join("/"))
    },
    _ => url.to_owned(),
  }
}

/// The file name of an `https:` URL: its last path segment, percent-decoded, or an empty text
/// without one. Labels of unlabeled trees come from it.
pub fn url_file_name(url: &str) -> String {
  UrlParts::of(url)
    .and_then(|p| p.path.rsplit('/').find(|s| !s.is_empty()))
    .map(|segment| percent_decode_str(segment).decode_utf8_lossy().into_owned())
    .unwrap_or_default()
}

/// The text of a tree or session file: gzip-compressed bytes (they start with `1f 8b`) are
/// decompressed first, up to [`MAX_TEXT_BYTES`]; the text must be UTF-8, and a web page (a text
/// that starts with `<!doctype html` or `<html`) is rejected, because a wrong address often
/// serves one.
pub fn decode_tree_bytes(bytes: &[u8]) -> Result<String, DecodeError> {
  let bytes = if bytes.starts_with(&[0x1f, 0x8b]) {
    let mut text = Vec::new();
    let limit = u64::try_from(MAX_TEXT_BYTES + 1).unwrap_or(u64::MAX);
    MultiGzDecoder::new(bytes)
      .take(limit)
      .read_to_end(&mut text)
      .map_err(DecodeError::Gzip)?;
    if text.len() > MAX_TEXT_BYTES {
      return Err(DecodeError::TooLarge);
    }
    text
  } else {
    bytes.to_vec()
  };
  let text = String::from_utf8(bytes).map_err(DecodeError::NotUtf8)?;
  let start = text
    .trim_start()
    .chars()
    .take(14)
    .collect::<String>()
    .to_ascii_lowercase();
  if start.starts_with("<!doctype html") || start.starts_with("<html") {
    return Err(DecodeError::WebPage);
  }
  Ok(text)
}

/// Why a value is not the location of a tree or session file, from [`parse_location`].
#[derive(Debug, thiserror::Error)]
pub enum LocationError {
  #[error("{0:?} is an http: address; use https:, because a page served over https cannot read http: addresses")]
  Http(String),
  #[error("{0:?} is neither an https: address nor a data: text")]
  Unsupported(String),
  #[error("{0:?} is not an https:// address")]
  NotHttps(String),
  #[error("{0:?} has no host")]
  NoHost(String),
  #[error("{0:?} must not contain a user name or password")]
  UserInfo(String),
  #[error("a data: location needs a comma before its data, as in data:,((A,B),C);")]
  DataWithoutComma,
  #[error("the base64 data is invalid: {0}")]
  Base64(#[source] base64::DecodeError),
  #[error(transparent)]
  Decode(#[from] DecodeError),
}

/// Why bytes are not the text of a tree or session file, from [`decode_tree_bytes`].
#[derive(Debug, thiserror::Error)]
pub enum DecodeError {
  #[error("not a valid gzip file: {0}")]
  Gzip(#[source] io::Error),
  #[error("the decompressed file is larger than {} MiB", MAX_TEXT_MIB)]
  TooLarge,
  #[error("not a UTF-8 text file: {}", .0.utf8_error())]
  NotUtf8(#[source] FromUtf8Error),
  #[error("a web page, not a tree file")]
  WebPage,
}

/// The settings of `base` with the values of `patch`: `Settings::default()` for trees and
/// examples, and the settings of the session file for `session`.
pub fn apply(base: &Settings, patch: &SettingsPatch) -> Settings {
  Settings {
    gamma: patch.gamma.unwrap_or(base.gamma),
    seq_lengths: patch.seq_lengths.clone().or_else(|| base.seq_lengths.clone()),
    n_mcmc_it: patch.n_mcmc_it.unwrap_or(base.n_mcmc_it),
    resolve: patch.resolve.unwrap_or(base.resolve),
    pre_resolve: patch.pre_resolve.unwrap_or(base.pre_resolve),
    rounds: patch.rounds.unwrap_or(base.rounds),
    final_round: patch.final_round.unwrap_or(base.final_round),
    likelihood: patch.likelihood.unwrap_or(base.likelihood),
    naive: patch.naive.unwrap_or(base.naive),
    seed: patch.seed.unwrap_or(base.seed),
  }
}

/// The key-value pairs of the canonical link of `request`, whose trees have the `addresses`, or
/// `None` when the request has fewer than two trees, which no link holds, or a tree has no
/// address, or a label or setting cannot be written: input keys, then
/// the settings that differ from `base` in the order of [`SETTING_KEYS`], then `run`. The input
/// is `example=<id>` when the trees are the trees of that example in its order with its labels,
/// and one `tree` per tree otherwise, with `<label>=` only where the label differs from the one
/// that [`parse_launch`] gives the location. A flag is a pair with an empty value.
pub fn launch_pairs(
  request: &AnalysisRequest,
  addresses: &[Option<TreeAddress>],
  base: &Settings,
  run: bool,
) -> Option<Vec<(String, String)>> {
  if addresses.len() != request.trees.len() || request.trees.len() < 2 {
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

/// The key-value pairs of a query or fragment text, read as browsers read form data
/// (`URLSearchParams`): pairs split at `&`, a key without `=` has an empty value, `+` is a space,
/// and percent-encoded bytes are decoded as UTF-8.
pub fn query_pairs(query: &str) -> Vec<(String, String)> {
  form_urlencoded::parse(query.as_bytes()).into_owned().collect()
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

/// Address of the web app: the GitHub Pages site of the repository of the workspace, such as
/// `https://neherlab.github.io/treeknit-rs/` for `https://github.com/neherlab/treeknit-rs`.
pub fn web_app_url() -> String {
  let repository = env!("CARGO_PKG_REPOSITORY").trim_end_matches('/');
  match repository
    .strip_prefix("https://github.com/")
    .and_then(|path| path.split_once('/'))
  {
    Some((owner, repo)) => format!("https://{}.github.io/{repo}/", owner.to_ascii_lowercase()),
    None => format!("{repository}/"),
  }
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

/// `url` with every character that a URI cannot hold percent-encoded (spaces, non-ASCII
/// characters, `"`, `<`, `>`, ...), as browsers send it: links keep such characters readable,
/// and an HTTP client needs them encoded.
pub fn request_url(url: &str) -> String {
  utf8_percent_encode(url, URI_SET).to_string()
}

/// Every key of links that is not a key of the display, with its value and a one-line
/// description, for the help of the web app: the inputs, the settings, `run`, and `v`.
pub fn launch_keys() -> Vec<LaunchKeyInfo> {
  let key = |key: &str, value: Option<&str>, description: String| LaunchKeyInfo {
    key: key.to_owned(),
    opposite: None,
    value: value.map(str::to_owned),
    description,
  };
  let mut keys = vec![
    key(
      "example",
      Some("<id>"),
      "A built-in example, such as h3n2-2017: the id of the Examples menu.".to_owned(),
    ),
    key(
      "tree",
      Some("[<label>=]<location>"),
      "A tree at an https: address or as a data: text, with an optional label; give at least two.".to_owned(),
    ),
    key(
      "session",
      Some("<location>"),
      format!(
        "A session file ({}) at an https: address or as a data: text.",
        output::SESSION_FILE
      ),
    ),
    key(
      "from",
      Some("opener|parent"),
      "Receive a session file from the page that opened TreeKnit (opener) or that embeds it (parent).".to_owned(),
    ),
  ];
  keys.extend(SETTING_KEYS.iter().map(|s| LaunchKeyInfo {
    key: s.key.to_owned(),
    opposite: s.opposite.map(str::to_owned),
    value: value_placeholder(s.value).map(str::to_owned),
    description: s.description.to_owned(),
  }));
  keys.push(key(
    "run",
    None,
    "Run the analysis once the inputs have loaded.".to_owned(),
  ));
  keys.push(key(
    "v",
    Some("<number>"),
    format!("Version of the link format; without it, version {FORMAT_VERSION}."),
  ));
  keys
}

/// A key-value pair of a link, as the web app passes it; a flag has an empty value.
#[derive(Clone, Debug, PartialEq, Eq, Deserialize, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
pub struct LinkEntry {
  pub key: String,
  pub value: String,
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
  pub fetch_timeout_seconds: u32,
  pub max_download_bytes: usize,
  pub max_link_chars: usize,
  pub long_link_chars: usize,
}

/// What [`parse_launch`] read from a link.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct LaunchParse {
  /// The launch; `None` when the link has no launch keys or its input cannot load.
  pub launch: Option<Launch>,
  /// Every problem, at the key it concerns.
  pub errors: Vec<ValidationError>,
  /// Keys that are neither launch keys nor keys of the display, in their order.
  pub ignored: Vec<IgnoredKey>,
}

/// The input, the settings, and the run of a link.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct Launch {
  pub input: LaunchInput,
  /// The settings of the link, applied by [`apply`] to the defaults or to the settings of the
  /// session file.
  pub settings: SettingsPatch,
  /// The link asks to run the analysis once the input has loaded.
  pub run: bool,
}

/// Where the trees of a launch come from.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LaunchInput {
  /// A built-in example (`example=<id>`).
  Example { id: String },
  /// Trees one by one (`tree=...`), at least two.
  Trees { trees: Vec<LaunchTree> },
  /// A session file (`session=<location>`).
  Session { location: LinkLocation },
  /// A session file posted by another window (`from=opener` or `from=parent`).
  Message { source: MessageSource },
}

/// A tree of a link with its label: given in the link, or from the file name of its location.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct LaunchTree {
  pub label: String,
  pub location: LinkLocation,
}

/// Where a file of a link is.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum LinkLocation {
  /// An `https:` URL as the link gives it, and the address to read it from ([`fetch_url`]).
  Url { url: String, fetch: String },
  /// The decoded text of a `data:` location.
  Data { text: String },
}

/// The window that posts the session file of `from=`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub enum MessageSource {
  /// The window that opened TreeKnit (`window.opener`).
  Opener,
  /// The window that embeds TreeKnit in a frame (`window.parent`).
  Parent,
}

/// A setting value of a link for each field of `Settings`; `None` keeps the value of the base.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct SettingsPatch {
  pub gamma: Option<f64>,
  pub seq_lengths: Option<Vec<f64>>,
  pub n_mcmc_it: Option<u64>,
  pub resolve: Option<ResolveMode>,
  pub pre_resolve: Option<bool>,
  pub rounds: Option<u64>,
  pub final_round: Option<bool>,
  pub likelihood: Option<bool>,
  pub naive: Option<bool>,
  pub seed: Option<u64>,
}

/// A key of a link that TreeKnit does not read.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct IgnoredKey {
  pub key: String,
  /// The launch or display key it most likely misspells, within edit distance 2; none for keys
  /// shorter than 4 characters.
  pub suggestion: Option<String>,
  /// The key follows a tree or session location with a `?`, so it is most likely part of that
  /// location's query: `&` inside a location must be written `%26`.
  pub after_location_query: bool,
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

/// A key of links with its value and description.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[cfg_attr(feature = "tsify", derive(Tsify))]
#[serde(rename_all = "camelCase")]
pub struct LaunchKeyInfo {
  pub key: String,
  /// For a flag, the key that sets the other value.
  pub opposite: Option<String>,
  /// The form of the value, such as `<number>`; `None` for a key without a value.
  pub value: Option<String>,
  pub description: String,
}

/// The state of [`parse_launch`] while it reads the pairs of a link.
#[derive(Default)]
struct LaunchReader {
  /// The kind of the first input key of the link.
  kind: Option<InputKind>,
  /// The input of a single-value input key, `None` when its value is invalid.
  input: Option<LaunchInput>,
  /// The `tree` values, in their order.
  trees: Vec<String>,
  patch: SettingsPatch,
  /// Keys of value settings given so far, so that a repeated key is an error.
  seen: BTreeSet<&'static str>,
  run: bool,
  /// The first key that needs an input: a setting or `run`.
  needs_input: Option<String>,
  /// The link names a format version that this build cannot read.
  too_new: bool,
  errors: Vec<ValidationError>,
  ignored: Vec<IgnoredKey>,
}

impl LaunchReader {
  /// Read one pair; `false` for a key that is no launch key.
  fn read(&mut self, key: &str, value: &str) -> bool {
    if let Some(kind) = input_kind(key) {
      self.read_input(kind, key, value);
    } else if let Some((setting, sets)) = setting_key(key) {
      self.read_setting(setting, sets, key, value);
      self.needs_input.get_or_insert_with(|| key.to_owned());
    } else if key == "run" {
      let on = self.flag(key, value);
      self.run |= on;
      self.needs_input.get_or_insert_with(|| key.to_owned());
    } else if key == "v" {
      self.read_version(value);
    } else {
      return false;
    }
    true
  }

  fn read_input(&mut self, kind: InputKind, key: &str, value: &str) {
    match self.kind {
      Some(first) if first != kind => {
        self.error(
          key,
          "use one input: an example, trees, a session file, or a message".to_owned(),
        );
        return;
      },
      Some(_) if kind != InputKind::Trees => {
        self.error(key, format!("{key} is given twice"));
        return;
      },
      _ => {},
    }
    self.kind = Some(kind);
    self.input = match kind {
      InputKind::Trees => {
        self.trees.push(value.to_owned());
        None
      },
      InputKind::Example => self.example(value).map(|id| LaunchInput::Example { id }),
      InputKind::Session => match parse_location(value) {
        Ok(location) => Some(LaunchInput::Session { location }),
        Err(e) => {
          self.error(key, e.to_string());
          None
        },
      },
      InputKind::Message => match value {
        "opener" => Some(LaunchInput::Message {
          source: MessageSource::Opener,
        }),
        "parent" => Some(LaunchInput::Message {
          source: MessageSource::Parent,
        }),
        _ => {
          self.error(key, format!("from must be opener or parent, got {value:?}"));
          None
        },
      },
    };
  }

  /// The example `id`, or an error with the closest id.
  fn example(&mut self, id: &str) -> Option<String> {
    if examples::example(id).is_some() {
      return Some(id.to_owned());
    }
    let hint = closest(id, examples::EXAMPLES.iter().map(|e| e.id))
      .map(|s| format!("; did you mean {s:?}?"))
      .unwrap_or_default();
    self.error("example", format!("there is no example {id:?}{hint}"));
    None
  }

  /// Read a setting key; `sets` is the value a flag key sets, `None` for a key with a value. A
  /// flag and its opposite may both occur, the last one wins; a key with a value occurs once.
  fn read_setting(&mut self, setting: &SettingKey, sets: Option<bool>, key: &str, value: &str) {
    if let Some(on) = sets {
      if self.flag(key, value) {
        let field = match setting.setting {
          SettingName::PreResolve => &mut self.patch.pre_resolve,
          SettingName::FinalRound => &mut self.patch.final_round,
          SettingName::Likelihood => &mut self.patch.likelihood,
          SettingName::Naive => &mut self.patch.naive,
          SettingName::Gamma
          | SettingName::SeqLengths
          | SettingName::NMcmcIt
          | SettingName::Resolve
          | SettingName::Rounds
          | SettingName::Seed => return,
        };
        *field = Some(on);
      }
      return;
    }
    if !self.seen.insert(setting.key) {
      self.error(key, format!("{key} is given twice"));
      return;
    }
    let p = &mut self.patch;
    let read = match setting.setting {
      SettingName::Gamma => parse_number(key, value).map(|v| p.gamma = Some(v)),
      SettingName::SeqLengths => parse_numbers(key, value).map(|v| p.seq_lengths = Some(v)),
      SettingName::NMcmcIt => parse_integer(key, value).map(|v| p.n_mcmc_it = Some(v)),
      SettingName::Rounds => parse_integer(key, value).map(|v| p.rounds = Some(v)),
      SettingName::Seed => parse_integer(key, value).map(|v| p.seed = Some(v)),
      SettingName::Resolve => parse_mode(value).map(|v| p.resolve = Some(v)),
      SettingName::PreResolve | SettingName::FinalRound | SettingName::Likelihood | SettingName::Naive => Ok(()),
    };
    if let Err(e) = read {
      self.error(key, e);
    }
  }

  /// Whether the flag `key` has no value, as a flag must; an error otherwise.
  fn flag(&mut self, key: &str, value: &str) -> bool {
    if !value.is_empty() {
      self.error(key, format!("{key} takes no value, got {value:?}"));
    }
    value.is_empty()
  }

  fn read_version(&mut self, value: &str) {
    match value.parse::<u64>() {
      Ok(v) if v <= FORMAT_VERSION => {},
      Ok(_) => {
        self.too_new = true;
        self.error("v", "This link needs a newer version of TreeKnit".to_owned());
      },
      Err(_) => self.error("v", format!("v must be a whole number, got {value:?}")),
    }
  }

  fn error(&mut self, key: &str, message: String) {
    self
      .errors
      .push(ValidationError::at(Field::LinkKey { key: key.to_owned() }, message));
  }

  fn finish(mut self) -> LaunchParse {
    let input = match self.kind {
      Some(InputKind::Trees) => self.tree_input(),
      Some(_) => self.input.take(),
      None => {
        if let Some(key) = self.needs_input.take() {
          self.error(
            &key,
            format!("{key} needs an input: add example=, tree=, session=, or from= to the link"),
          );
        }
        None
      },
    };
    let launch = input.filter(|_| !self.too_new).map(|input| Launch {
      input,
      settings: self.patch,
      run: self.run,
    });
    LaunchParse {
      launch,
      errors: self.errors,
      ignored: self.ignored,
    }
  }

  /// The trees of the `tree` values, labeled, or `None` when a location is invalid or the link
  /// has fewer than two trees.
  fn tree_input(&mut self) -> Option<LaunchInput> {
    let mut parsed = Vec::new();
    let mut valid = true;
    for (i, value) in self.trees.iter().enumerate() {
      let token = split_label(value);
      match parse_location(token.rest) {
        Ok(location) => parsed.push((token.label.map(str::to_owned), location)),
        Err(e) => {
          self
            .errors
            .push(ValidationError::at(Field::LinkTree { index: i }, e.to_string()));
          valid = false;
        },
      }
    }
    if self.trees.len() < 2 {
      self.errors.push(ValidationError::at(
        Field::LinkKey { key: "tree".to_owned() },
        format!("a link needs at least two trees, got {}", self.trees.len()),
      ));
      valid = false;
    }
    if !valid {
      return None;
    }
    let labels = link_labels(&parsed);
    let trees = parsed
      .into_iter()
      .zip(labels)
      .map(|((_, location), label)| LaunchTree { label, location })
      .collect();
    Some(LaunchInput::Trees { trees })
  }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum InputKind {
  Example,
  Trees,
  Session,
  Message,
}

/// The input kind of a key, `None` for a key that names no input.
fn input_kind(key: &str) -> Option<InputKind> {
  match key {
    "example" => Some(InputKind::Example),
    "tree" => Some(InputKind::Trees),
    "session" => Some(InputKind::Session),
    "from" => Some(InputKind::Message),
    _ => None,
  }
}

/// The setting of a key and, for a flag, the value it sets.
fn setting_key(key: &str) -> Option<(&'static SettingKey, Option<bool>)> {
  SETTING_KEYS.iter().find_map(|s| match s.value {
    KeyValue::Flag { key_sets } if s.key == key => Some((s, Some(key_sets))),
    KeyValue::Flag { key_sets } if s.opposite == Some(key) => Some((s, Some(!key_sets))),
    KeyValue::Flag { .. } => None,
    _ if s.key == key => Some((s, None)),
    _ => None,
  })
}

/// Labels of the trees of a link: the given labels, and for the others `analysis::tree_labels`
/// of the file names of their locations, with the given labels as existing labels.
fn link_labels(trees: &[(Option<String>, LinkLocation)]) -> Vec<String> {
  let given: Vec<String> = trees.iter().filter_map(|(label, _)| label.clone()).collect();
  let names: Vec<String> = trees
    .iter()
    .filter(|(label, _)| label.is_none())
    .map(|(_, location)| location_file_name(location))
    .collect();
  let mut automatic = analysis::tree_labels(&names, &given).into_iter();
  trees
    .iter()
    .map(|(label, _)| label.clone().or_else(|| automatic.next()).unwrap_or_default())
    .collect()
}

fn location_file_name(location: &LinkLocation) -> String {
  match location {
    LinkLocation::Url { url, .. } => url_file_name(url),
    LinkLocation::Data { .. } => String::new(),
  }
}

fn parse_number(key: &str, value: &str) -> Result<f64, String> {
  value
    .trim()
    .parse::<f64>()
    .ok()
    .filter(|v| v.is_finite())
    .ok_or_else(|| format!("{key} must be a number, got {value:?}"))
}

fn parse_numbers(key: &str, value: &str) -> Result<Vec<f64>, String> {
  value
    .split(',')
    .map(|v| v.trim().parse::<f64>().ok().filter(|v| v.is_finite()))
    .collect::<Option<Vec<_>>>()
    .ok_or_else(|| format!("{key} must be numbers separated by commas, such as 1701,1410, got {value:?}"))
}

/// A whole number of a link, at most `analysis::MAX_SEED`, the largest integer that a JavaScript
/// number holds exactly, so the web app receives the value of the link.
fn parse_integer(key: &str, value: &str) -> Result<u64, String> {
  match value.trim().parse::<u64>() {
    Ok(v) if v <= analysis::MAX_SEED => Ok(v),
    Ok(_) => Err(format!("{key} must be at most {}, got {value}", analysis::MAX_SEED)),
    Err(_) => Err(format!("{key} must be a whole number, got {value:?}")),
  }
}

fn parse_mode(value: &str) -> Result<ResolveMode, String> {
  from_wire_name(value).ok_or_else(|| format!("resolve must be matched, strict, liberal, or none, got {value:?}"))
}

fn value_placeholder(value: KeyValue) -> Option<&'static str> {
  match value {
    KeyValue::Number => Some("<number>"),
    KeyValue::Numbers => Some("<number>,<number>,..."),
    KeyValue::Integer => Some("<whole number>"),
    KeyValue::Mode => Some("matched|strict|liberal|none"),
    KeyValue::Flag { .. } => None,
  }
}

/// The key of the launch keys and `view_keys` that `key` most likely misspells.
fn suggestion(key: &str, view_keys: &[String]) -> Option<String> {
  if key.chars().count() < SUGGESTION_MIN_CHARS {
    return None;
  }
  let launch = INPUT_KEYS
    .into_iter()
    .chain(SETTING_KEYS.iter().flat_map(|s| [Some(s.key), s.opposite]).flatten())
    .chain(["run"]);
  closest(key, launch.chain(view_keys.iter().map(String::as_str))).map(str::to_owned)
}

/// The candidate within [`SUGGESTION_MAX_DISTANCE`] edits of `word` with the fewest edits, the
/// first one on a tie.
fn closest<'a>(word: &str, candidates: impl Iterator<Item = &'a str>) -> Option<&'a str> {
  candidates
    .map(|c| (strsim::levenshtein(word, c), c))
    .filter(|(d, _)| *d <= SUGGESTION_MAX_DISTANCE)
    .min_by_key(|(d, _)| *d)
    .map(|(_, c)| c)
}

/// Check an `https:` URL: it has a host and no user name or password.
fn check_https(url: &str) -> Result<(), LocationError> {
  let parts = UrlParts::of(url).ok_or_else(|| LocationError::NotHttps(url.to_owned()))?;
  if parts.host.is_empty() {
    return Err(LocationError::NoHost(url.to_owned()));
  }
  if parts.has_user {
    return Err(LocationError::UserInfo(url.to_owned()));
  }
  Ok(())
}

/// The host and the path of an `https://` URL.
struct UrlParts<'a> {
  host: &'a str,
  /// The authority holds a user name or password (`user@host`).
  has_user: bool,
  path: &'a str,
}

impl<'a> UrlParts<'a> {
  fn of(url: &'a str) -> Option<Self> {
    let rest = url
      .get(..8)
      .filter(|scheme| scheme.eq_ignore_ascii_case("https://"))
      .and_then(|_| url.get(8..))?;
    let (authority, after) = rest.split_at(rest.find(['/', '?', '#']).unwrap_or(rest.len()));
    let path = after.split(['?', '#']).next().unwrap_or_default();
    let (has_user, host_port) = match authority.rsplit_once('@') {
      Some((_, host)) => (true, host),
      None => (false, authority),
    };
    let host = host_port.rsplit_once(':').map_or(host_port, |(h, _)| h);
    Some(UrlParts { host, has_user, path })
  }
}

/// The text of a `data:` location (RFC 2397): its data after the first `,`, decoded from base64
/// when the media type ends with `;base64`, and percent-decoded otherwise, then decoded by
/// [`decode_tree_bytes`].
fn decode_data_url(value: &str) -> Result<String, LocationError> {
  let (meta, data) = value
    .get(5..)
    .and_then(|rest| rest.split_once(','))
    .ok_or(LocationError::DataWithoutComma)?;
  let bytes = if meta.trim_end().to_ascii_lowercase().ends_with(";base64") {
    decode_base64(data)?
  } else {
    percent_decode_str(data).collect()
  };
  Ok(decode_tree_bytes(&bytes)?)
}

/// Bytes of base64 text in the standard or the URL-safe alphabet, with or without padding. A
/// space stands for `+`, which a reader of form data turns into a space.
fn decode_base64(text: &str) -> Result<Vec<u8>, LocationError> {
  let config = GeneralPurposeConfig::new().with_decode_padding_mode(DecodePaddingMode::Indifferent);
  let text = text.replace(' ', "+");
  let text = text.trim();
  GeneralPurpose::new(&alphabet::STANDARD, config)
    .decode(text)
    .or_else(|_standard_error| GeneralPurpose::new(&alphabet::URL_SAFE, config).decode(text))
    .map_err(LocationError::Base64)
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
      Some(TreeAddress::Url { url }) => Some((url.clone(), url_file_name(url))),
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

/// Which trees need an explicit label: the fewest trees such that [`link_labels`] gives the
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
    _ => (s.key.to_owned(), value),
  }
}

/// The key of a flag that differs from `base`: the key when it sets `value`, its opposite
/// otherwise.
fn flag_pair(s: &SettingKey, value: bool, base: bool) -> Option<String> {
  let KeyValue::Flag { key_sets } = s.value else {
    return None;
  };
  (value != base).then(|| {
    if value == key_sets {
      s.key.to_owned()
    } else {
      s.opposite.unwrap_or(s.key).to_owned()
    }
  })
}

/// `value` when it differs from `base`.
fn changed<'a, T: PartialEq>(value: &'a T, base: &T) -> Option<&'a T> {
  (value != base).then_some(value)
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::analysis::TreeText;
  use pretty_assertions::assert_eq;
  use rand::{Rng, SeedableRng};
  use rand_xoshiro::Xoshiro256PlusPlus;
  use rstest::rstest;
  use std::iter;
  use treeknit_testing::assert_err;

  fn pairs(query: &str) -> Vec<(String, String)> {
    query_pairs(query)
  }

  fn parse(query: &str) -> LaunchParse {
    parse_launch(
      &pairs(query),
      &["view".to_owned(), "auspice".to_owned(), "leaf".to_owned()],
    )
  }

  fn url(u: &str) -> LinkLocation {
    LinkLocation::Url {
      url: u.to_owned(),
      fetch: fetch_url(u),
    }
  }

  fn tree(label: &str, location: LinkLocation) -> LaunchTree {
    LaunchTree {
      label: label.to_owned(),
      location,
    }
  }

  fn error(key: &str, message: &str) -> ValidationError {
    ValidationError::at(Field::LinkKey { key: key.to_owned() }, message)
  }

  fn launch(input: LaunchInput, settings: SettingsPatch, run: bool) -> Option<Launch> {
    Some(Launch { input, settings, run })
  }

  fn gzip(bytes: &[u8]) -> Vec<u8> {
    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(bytes).unwrap();
    encoder.finish().unwrap()
  }

  #[test]
  fn example_with_settings_and_run() {
    let expected = LaunchParse {
      launch: launch(
        LaunchInput::Example {
          id: "h3n2-2017".to_owned(),
        },
        SettingsPatch {
          gamma: Some(3.0),
          resolve: Some(ResolveMode::Strict),
          final_round: Some(false),
          ..SettingsPatch::default()
        },
        true,
      ),
      errors: vec![],
      ignored: vec![],
    };
    assert_eq!(
      expected,
      parse("example=h3n2-2017&gamma=3&resolve=strict&no-final-round&run&view=mccs")
    );
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::label_before_first_equals("a=b=https://x/y",  (Some("a"), "b=https://x/y"))]
  #[case::query_with_equals(        "https://x/y?a=b",  (None,      "https://x/y?a=b"))]
  #[case::data_location(            "data:,(A=B,C);",   (None,      "data:,(A=B,C);"))]
  #[case::labeled_data(             "ha=data:,(A,B);",  (Some("ha"), "data:,(A,B);"))]
  #[case::empty_label(              "=https://x/y",     (None,      "=https://x/y"))]
  #[case::path_with_equals(         "./a=b.nwk",        (None,      "./a=b.nwk"))]
  #[case::backslash(                "a\\b=c.nwk",       (None,      "a\\b=c.nwk"))]
  #[trace]
  fn labels_are_split_at_the_first_equals(#[case] token: &str, #[case] (label, rest): (Option<&str>, &str)) {
    assert_eq!(LabeledToken { label, rest }, split_label(token));
  }

  #[test]
  fn trees_get_labels_from_their_file_names_next_to_given_labels() {
    let parsed =
      parse("tree=https://x/a/ha.nwk&tree=ha=https://x/b/ha.nwk&tree=https://x/c/&tree=data:,(A,B);&tree=data:,(A,C);");
    let expected = LaunchInput::Trees {
      trees: vec![
        tree("ha_2", url("https://x/a/ha.nwk")),
        tree("ha", url("https://x/b/ha.nwk")),
        tree("c", url("https://x/c/")),
        tree(
          "tree",
          LinkLocation::Data {
            text: "(A,B);".to_owned(),
          },
        ),
        tree(
          "tree_2",
          LinkLocation::Data {
            text: "(A,C);".to_owned(),
          },
        ),
      ],
    };
    assert_eq!(
      (Some(expected), vec![]),
      (parsed.launch.map(|l| l.input), parsed.errors)
    );
  }

  #[test]
  fn url_without_a_file_name_gives_the_label_tree() {
    let parsed = parse("tree=https://example.org&tree=https://example.org/");
    let labels: Vec<String> = match parsed.launch.unwrap().input {
      LaunchInput::Trees { trees } => trees.into_iter().map(|t| t.label).collect(),
      other => panic!("expected trees, got {other:?}"),
    };
    assert_eq!(vec!["tree", "tree_2"], labels);
  }

  #[test]
  fn file_names_are_percent_decoded() {
    assert_eq!("seg 4.nwk", url_file_name("https://x/a/seg%204.nwk?download=1"));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::github_blob(  "https://github.com/org/repo/blob/main/data/ha.nwk",          "https://raw.githubusercontent.com/org/repo/main/data/ha.nwk")]
  #[case::github_raw(   "https://github.com/org/repo/raw/v1.0/ha.nwk",                "https://raw.githubusercontent.com/org/repo/v1.0/ha.nwk")]
  #[case::github_query( "https://github.com/org/repo/blob/main/ha.nwk?raw=true",      "https://raw.githubusercontent.com/org/repo/main/ha.nwk")]
  #[case::zenodo(       "https://zenodo.org/records/123/files/ha.nwk",               "https://zenodo.org/api/records/123/files/ha.nwk/content")]
  #[case::zenodo_query( "https://zenodo.org/records/123/files/ha.nwk?download=1",    "https://zenodo.org/api/records/123/files/ha.nwk/content")]
  #[case::zenodo_folder("https://zenodo.org/records/123/files/org/ha.nwk",           "https://zenodo.org/api/records/123/files/org/ha.nwk/content")]
  #[case::raw_github(   "https://raw.githubusercontent.com/org/repo/main/ha.nwk",    "https://raw.githubusercontent.com/org/repo/main/ha.nwk")]
  #[case::github_repo(  "https://github.com/org/repo",                               "https://github.com/org/repo")]
  #[case::other_host(   "https://data.nextstrain.org/files/ha.nwk",                  "https://data.nextstrain.org/files/ha.nwk")]
  #[trace]
  fn copied_addresses_are_rewritten_to_their_file_addresses(#[case] given: &str, #[case] fetch: &str) {
    assert_eq!(fetch, fetch_url(given));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::http(      "http://x/ha.nwk",          "\"http://x/ha.nwk\" is an http: address; use https:, because a page served over https cannot read http: addresses")]
  #[case::ftp(       "ftp://x/ha.nwk",           "\"ftp://x/ha.nwk\" is neither an https: address nor a data: text")]
  #[case::file_name( "ha.nwk",                   "\"ha.nwk\" is neither an https: address nor a data: text")]
  #[case::user(      "https://me:pw@x/ha.nwk",   "\"https://me:pw@x/ha.nwk\" must not contain a user name or password")]
  #[case::no_host(   "https:///ha.nwk",          "\"https:///ha.nwk\" has no host")]
  #[case::no_slashes("https:ha.nwk",             "\"https:ha.nwk\" is not an https:// address")]
  #[case::no_comma(  "data:text/plain",          "a data: location needs a comma before its data, as in data:,((A,B),C);")]
  #[trace]
  fn invalid_locations_are_errors(#[case] value: &str, #[case] message: &str) {
    assert_err!(parse_location(value), message);
  }

  #[test]
  fn invalid_tree_location_is_an_error_at_its_tree_and_stops_the_launch() {
    let parsed = parse("tree=https://x/ha.nwk&tree=ha.nwk");
    let expected = vec![ValidationError::at(
      Field::LinkTree { index: 1 },
      "\"ha.nwk\" is neither an https: address nor a data: text",
    )];
    assert_eq!((None, expected), (parsed.launch, parsed.errors));
  }

  #[test]
  fn one_tree_is_too_few() {
    let parsed = parse("tree=https://x/ha.nwk&run");
    let expected = vec![error("tree", "a link needs at least two trees, got 1")];
    assert_eq!((None, expected), (parsed.launch, parsed.errors));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::trees_then_example( "tree=https://x/a&tree=https://x/b&example=h3n2-2017", "example")]
  #[case::example_then_tree(  "example=h3n2-2017&tree=https://x/a",                "tree")]
  #[case::example_then_from(  "example=h3n2-2017&from=opener",                     "from")]
  #[trace]
  fn a_second_input_kind_is_an_error_and_the_first_one_loads(#[case] query: &str, #[case] key: &str) {
    let parsed = parse(query);
    let expected = vec![error(key, "use one input: an example, trees, a session file, or a message")];
    assert_eq!((true, expected), (parsed.launch.is_some(), parsed.errors));
  }

  #[test]
  fn a_repeated_example_is_an_error() {
    let parsed = parse("example=h3n2-2017&example=5-leaves");
    let expected = launch(
      LaunchInput::Example {
        id: "h3n2-2017".to_owned(),
      },
      SettingsPatch::default(),
      false,
    );
    assert_eq!(
      (expected, vec![error("example", "example is given twice")]),
      (parsed.launch, parsed.errors)
    );
  }

  #[test]
  fn unknown_example_suggests_the_closest_id() {
    let parsed = parse("example=h3n2-2071");
    let expected = vec![error(
      "example",
      "there is no example \"h3n2-2071\"; did you mean \"h3n2-2017\"?",
    )];
    assert_eq!((None, expected), (parsed.launch, parsed.errors));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::absent(  "example=5-leaves",      true,  vec![])]
  #[case::one(     "example=5-leaves&v=1",  true,  vec![])]
  #[case::newer(   "example=5-leaves&v=2",  false, vec![error("v", "This link needs a newer version of TreeKnit")])]
  #[case::invalid( "example=5-leaves&v=x",  true,  vec![error("v", "v must be a whole number, got \"x\"")])]
  #[trace]
  fn version_one_and_absent_versions_load(#[case] query: &str, #[case] loads: bool, #[case] errors: Vec<ValidationError>) {
    let parsed = parse(query);
    assert_eq!((loads, errors), (parsed.launch.is_some(), parsed.errors));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::gamma_text(    "gamma=abc",       error("gamma", "gamma must be a number, got \"abc\""))]
  #[case::gamma_infinite("gamma=inf",       error("gamma", "gamma must be a number, got \"inf\""))]
  #[case::resolve(       "resolve=fast",    error("resolve", "resolve must be matched, strict, liberal, or none, got \"fast\""))]
  #[case::rounds(        "rounds=1.5",      error("rounds", "rounds must be a whole number, got \"1.5\""))]
  #[case::seed_large(    "seed=9007199254740992", error("seed", "seed must be at most 9007199254740991, got 9007199254740992"))]
  #[case::lengths(       "seq-lengths=1701 1410", error("seq-lengths", "seq-lengths must be numbers separated by commas, such as 1701,1410, got \"1701 1410\""))]
  #[case::flag_value(    "naive=1",         error("naive", "naive takes no value, got \"1\""))]
  #[case::run_value(     "run=true",        error("run", "run takes no value, got \"true\""))]
  #[trace]
  fn values_of_the_wrong_type_are_errors_and_leave_the_setting_unset(#[case] query: &str, #[case] expected: ValidationError) {
    let parsed = parse(&format!("example=5-leaves&{query}"));
    let launch = parsed.launch.map(|l| (l.settings, l.run));
    assert_eq!((vec![expected], Some((SettingsPatch::default(), false))), (parsed.errors, launch));
  }

  #[test]
  fn a_repeated_setting_is_an_error_and_keeps_the_first_value() {
    let parsed = parse("example=5-leaves&gamma=1&gamma=2");
    let expected = SettingsPatch {
      gamma: Some(1.0),
      ..SettingsPatch::default()
    };
    assert_eq!(
      (vec![error("gamma", "gamma is given twice")], Some(expected)),
      (parsed.errors, parsed.launch.map(|l| l.settings))
    );
  }

  #[test]
  fn values_outside_their_bounds_go_into_the_patch() {
    let parsed = parse("example=5-leaves&gamma=-1&rounds=0&seq-lengths=1701,0,-3");
    let expected = SettingsPatch {
      gamma: Some(-1.0),
      rounds: Some(0),
      seq_lengths: Some(vec![1701.0, 0.0, -3.0]),
      ..SettingsPatch::default()
    };
    assert_eq!(
      (vec![], Some(expected)),
      (parsed.errors, parsed.launch.map(|l| l.settings))
    );
  }

  #[test]
  fn the_last_of_a_flag_and_its_opposite_wins() {
    let parsed = parse("example=5-leaves&pre-resolve&no-pre-resolve&no-naive&naive&likelihood&final-round");
    let expected = SettingsPatch {
      pre_resolve: Some(false),
      naive: Some(true),
      likelihood: Some(true),
      final_round: Some(true),
      ..SettingsPatch::default()
    };
    assert_eq!(
      (vec![], Some(expected)),
      (parsed.errors, parsed.launch.map(|l| l.settings))
    );
  }

  #[test]
  fn settings_without_an_input_are_an_error_at_the_first_one() {
    let parsed = parse("view=mccs&gamma=3&run");
    let expected = vec![error(
      "gamma",
      "gamma needs an input: add example=, tree=, session=, or from= to the link",
    )];
    assert_eq!((None, expected), (parsed.launch, parsed.errors));
  }

  #[test]
  fn a_link_with_display_keys_only_has_no_launch() {
    let expected = LaunchParse {
      launch: None,
      errors: vec![],
      ignored: vec![],
    };
    assert_eq!(expected, parse("view=mccs&leaf=A&v=1"));
  }

  #[test]
  fn unknown_keys_are_ignored_with_a_suggestion() {
    let parsed = parse("example=5-leaves&gama=3&aupsice=c%3Dmcc&c=mcc&utm_source=paper");
    let ignored = |key: &str, suggestion: Option<&str>| IgnoredKey {
      key: key.to_owned(),
      suggestion: suggestion.map(str::to_owned),
      after_location_query: false,
    };
    let expected = vec![
      ignored("gama", Some("gamma")),
      ignored("aupsice", Some("auspice")),
      ignored("c", None),
      ignored("utm_source", None),
    ];
    assert_eq!(expected, parsed.ignored);
  }

  #[test]
  fn a_key_after_a_tree_location_with_a_query_carries_the_hint() {
    let parsed = parse("tree=https://x/ha.nwk?token=1&sig=2&tree=https://x/na.nwk&other=1");
    let expected = vec![
      IgnoredKey {
        key: "sig".to_owned(),
        suggestion: None,
        after_location_query: true,
      },
      IgnoredKey {
        key: "other".to_owned(),
        suggestion: None,
        after_location_query: false,
      },
    ];
    assert_eq!(expected, parsed.ignored);
  }

  #[test]
  fn session_and_message_inputs() {
    let session = parse("session=https://zenodo.org/records/1/files/s.json&seed=7");
    let expected = launch(
      LaunchInput::Session {
        location: url("https://zenodo.org/records/1/files/s.json"),
      },
      SettingsPatch {
        seed: Some(7),
        ..SettingsPatch::default()
      },
      false,
    );
    assert_eq!(expected, session.launch);
    let message = parse("from=parent&run");
    let expected = launch(
      LaunchInput::Message {
        source: MessageSource::Parent,
      },
      SettingsPatch::default(),
      true,
    );
    assert_eq!(expected, message.launch);
    assert_eq!(
      vec![error("from", "from must be opener or parent, got \"top\"")],
      parse("from=top").errors
    );
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::plain(          "data:,((A,B),C);",                     "((A,B),C);")]
  #[case::percent(        "data:,((A%20x,B),C)%3B",               "((A x,B),C);")]
  #[case::invalid_percent("data:,(A%zz,B);",                      "(A%zz,B);")]
  #[case::signed_percent( "data:,(A%+5,B);",                      "(A%+5,B);")]
  #[case::base64(         "data:text/plain;base64,KChBLEIpLEMpOw==", "((A,B),C);")]
  #[case::base64_unpadded("data:;base64,KChBLEIpLEMpOw",          "((A,B),C);")]
  #[case::base64_url(     "data:;base64,KChBLEIpLEMpOz8-",        "((A,B),C);?>")]
  #[case::base64_spaces(  "data:;base64,KChBLEIpLEMpOz8 ",        "((A,B),C);?>")]
  #[trace]
  fn data_locations_are_decoded(#[case] value: &str, #[case] text: &str) {
    assert_eq!(LinkLocation::Data { text: text.to_owned() }, parse_location(value).unwrap());
  }

  #[test]
  fn gzip_data_is_decompressed() {
    let encoded = general_purpose::STANDARD.encode(gzip(b"((A,B),C);"));
    assert_eq!(
      LinkLocation::Data {
        text: "((A,B),C);".to_owned()
      },
      parse_location(&format!("data:application/gzip;base64,{encoded}")).unwrap()
    );
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::doctype(   b"  <!DOCTYPE html><html></html>".as_slice(), Err("a web page, not a tree file".to_owned()))]
  #[case::html(      b"\n<HTML lang=en>".as_slice(),              Err("a web page, not a tree file".to_owned()))]
  #[case::not_utf8(  b"(A,\xff);".as_slice(),                     Err("not a UTF-8 text file: invalid utf-8 sequence of 1 bytes from index 3".to_owned()))]
  #[case::broken_gz( b"\x1f\x8b\x08\x00garbage".as_slice(),        Err("not a valid gzip file: incomplete deflate stream".to_owned()))]
  #[case::newick(    b"((A,B),C);\n".as_slice(),                  Ok("((A,B),C);\n".to_owned()))]
  #[case::html_name( b"(<html>,B);".as_slice(),                   Ok("(<html>,B);".to_owned()))]
  #[trace]
  fn tree_bytes_are_checked(#[case] bytes: &[u8], #[case] expected: Result<String, String>) {
    assert_eq!(expected, decode_tree_bytes(bytes).map_err(|e| e.to_string()));
  }

  #[test]
  fn gzip_members_are_concatenated() {
    let mut bytes = gzip(b"((A,B),");
    bytes.extend(gzip(b"C);"));
    assert_eq!("((A,B),C);", decode_tree_bytes(&bytes).unwrap());
  }

  #[test]
  fn apply_replaces_the_values_of_the_patch() {
    let base = Settings {
      seed: 9,
      pre_resolve: true,
      ..Settings::default()
    };
    let patch = SettingsPatch {
      gamma: Some(3.0),
      pre_resolve: Some(false),
      ..SettingsPatch::default()
    };
    let expected = Settings {
      gamma: 3.0,
      pre_resolve: false,
      ..base.clone()
    };
    assert_eq!(expected, apply(&base, &patch));
  }

  #[rustfmt::skip]
  #[rstest]
  #[case::leaf(       &[("leaf", "A/New York/392/2004")],         "leaf=A/New%20York/392/2004")]
  #[case::auspice(    &[("auspice", "legend=open&l=radial")],     "auspice=legend=open%26l=radial")]
  #[case::flag(       &[("example", "5-leaves"), ("run", "")],   "example=5-leaves&run")]
  #[case::readable(   &[("tree", "ha=https://x/a.nwk?b=c")],      "tree=ha=https://x/a.nwk?b=c")]
  #[case::plus_hash(  &[("s", "a+b#c%d")],                       "s=a%2Bb%23c%25d")]
  #[case::unicode(    &[("leaf", "Ä")],                           "leaf=%C3%84")]
  #[case::key_equals( &[("a=b", "c")],                            "a%3Db=c")]
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
  fn request_url_encodes_what_a_uri_cannot_hold() {
    assert_eq!(
      "https://x/a%20b/%C3%A9.nwk?q=1&r=%22#f",
      request_url("https://x/a b/é.nwk?q=1&r=\"#f")
    );
  }

  #[test]
  fn web_app_url_is_the_pages_site_of_the_repository() {
    assert_eq!("https://neherlab.github.io/treeknit-rs/", web_app_url());
  }

  #[test]
  fn launch_keys_list_inputs_settings_run_and_version() {
    let keys: Vec<String> = launch_keys().into_iter().map(|k| k.key).collect();
    let expected = [
      "example",
      "tree",
      "session",
      "from",
      "gamma",
      "seq-lengths",
      "n-mcmc-it",
      "resolve",
      "pre-resolve",
      "rounds",
      "no-final-round",
      "no-likelihood",
      "naive",
      "seed",
      "run",
      "v",
    ];
    assert_eq!(expected.to_vec(), keys);
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
  #[case::no_tree(  &[],     &[])]
  #[case::one_tree( &["ha"], &[Some(TreeAddress::Data)])]
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
          Some(TreeAddress::Url { url }) => url_file_name(url),
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
