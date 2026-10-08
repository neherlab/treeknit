//! The input of a run: tree files and locations, a session file, an example, or a link, read
//! from files and `https:` addresses.

use crate::{Cli, InputError, fail};
use eyre::Result;
use std::collections::BTreeSet;
use std::fmt::Display;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::Duration;
use treeknit_io::analysis::{self, AnalysisRequest, Field, Settings, TreeText, ValidationError};
use treeknit_io::examples;
use treeknit_io::launch::{self, DecodeError, LaunchInput, LinkLocation, LocationError, TreeAddress};
use treeknit_io::output::OutputOptions;

/// Why the text of a file or location cannot be read.
#[derive(Debug, thiserror::Error)]
pub(crate) enum ReadError {
  #[error("{0}")]
  File(#[source] io::Error),
  #[error("{0}")]
  FileText(#[source] DecodeError),
  #[error(transparent)]
  Location(#[from] LocationError),
  #[error("cannot read {url}: {source}")]
  Download { url: String, source: DownloadError },
  #[error("cannot read {url}: {source}")]
  DownloadText { url: String, source: DecodeError },
}

/// Reads the bytes at an `https:` address, or the error with the reason.
pub(crate) type Fetch<'a> = &'a dyn Fn(&str) -> Result<Vec<u8>, DownloadError>;

/// The bytes at the `https:` address `url`, with the limits of the web app: an answer within
/// [`launch::FETCH_TIMEOUT_SECONDS`] and at most [`launch::MAX_DOWNLOAD_BYTES`].
pub(crate) fn http_get(url: &str) -> Result<Vec<u8>, DownloadError> {
  let agent: ureq::Agent = ureq::Agent::config_builder()
    .timeout_global(Some(Duration::from_secs(launch::FETCH_TIMEOUT_SECONDS.into())))
    .build()
    .into();
  let limit = u64::try_from(launch::MAX_DOWNLOAD_BYTES).unwrap_or(u64::MAX);
  agent
    .get(launch::request_url(url))
    .call()
    .and_then(|mut response| response.body_mut().with_config().limit(limit).read_to_vec())
    .map_err(DownloadError::from)
}

/// The reason of a failed download, in the words of the web app.
#[derive(Debug, thiserror::Error)]
pub(crate) enum DownloadError {
  /// The server answered with this error status.
  #[error("{}", status_text(*.0))]
  Status(u16),
  #[error("no answer within {} seconds", launch::FETCH_TIMEOUT_SECONDS)]
  Timeout,
  #[error("the file is larger than {} MiB", launch::MAX_DOWNLOAD_BYTES >> 20)]
  TooLarge,
  #[error(transparent)]
  Other(ureq::Error),
}

impl From<ureq::Error> for DownloadError {
  fn from(e: ureq::Error) -> Self {
    match e {
      ureq::Error::StatusCode(code) => Self::Status(code),
      ureq::Error::Timeout(_) => Self::Timeout,
      ureq::Error::BodyExceedsLimit(_) => Self::TooLarge,
      other => Self::Other(other),
    }
  }
}

/// An HTTP status with its reason phrase, such as `404 Not Found`, or the bare code.
fn status_text(code: u16) -> String {
  let reason = ureq::http::StatusCode::from_u16(code)
    .ok()
    .and_then(|s| s.canonical_reason());
  reason.map_or_else(|| format!("{code}"), |r| format!("{code} {r}"))
}

/// The trees of a run with what the command line needs to report on them, to name their output
/// files, and to write their link.
pub(crate) struct Input {
  pub(crate) texts: Vec<TreeText>,
  /// Where the trees and settings come from, for error messages.
  pub(crate) source: Source,
  /// Extension of each tree's output files.
  pub(crate) extensions: Vec<String>,
  /// The settings before the analysis options: the defaults, or the settings of the session file
  /// or of the link.
  pub(crate) base: Settings,
  /// Where a link gets each tree again.
  pub(crate) addresses: Vec<Option<TreeAddress>>,
  /// The trees come with their labels and settings (a session file, an example, or a link), so the
  /// results directory receives their session file.
  pub(crate) session: bool,
  /// The index and the error of each input file that cannot be read; its tree has an empty text.
  pub(crate) read_errors: Vec<(usize, ValidationError)>,
}

impl Input {
  /// The trees and settings of `request`, read from `source`, with the output names of the web app.
  fn of_request(request: AnalysisRequest, source: Source, addresses: Vec<Option<TreeAddress>>) -> Self {
    let k = request.trees.len();
    Input {
      texts: request.trees,
      source,
      extensions: OutputOptions::web(k).extensions,
      base: request.settings,
      addresses,
      session: true,
      read_errors: Vec::new(),
    }
  }

  /// The read errors, then `errors` without the errors of the Newick text of the files that
  /// cannot be read, which are about their empty text.
  pub(crate) fn with_read_errors(&self, errors: Vec<ValidationError>) -> Vec<ValidationError> {
    let unread: Vec<Field> = self
      .read_errors
      .iter()
      .map(|(i, _)| Field::TreeNewick { index: *i })
      .collect();
    let mut all: Vec<ValidationError> = self.read_errors.iter().map(|(_, e)| e.clone()).collect();
    all.extend(
      errors
        .into_iter()
        .filter(|e| !e.field.as_ref().is_some_and(|f| unread.contains(f))),
    );
    all
  }
}

/// Where the trees and the settings of a run come from.
pub(crate) enum Source {
  /// The name of each tree: its file path or its address; the settings come from the options.
  Trees(Vec<String>),
  /// The session file, example, or link that holds the trees and the settings, by name.
  Named(String),
}

/// The input that the options of `cli` name, read with `fetch` from `https:` addresses.
pub(crate) fn read_input(cli: &Cli, fetch: Fetch<'_>) -> Result<Input> {
  if let Some(id) = &cli.example {
    example_input(id)
  } else if let Some(path) = &cli.session {
    session_input(path, fetch)
  } else if let Some(url) = &cli.link {
    link_input(url, fetch)
  } else {
    Ok(tree_input(&cli.trees, fetch))
  }
}

/// The trees of the tree arguments: files and `https:` or `data:` locations, each with an
/// optional `<label>=`. Unlabeled files are labeled by path (`path_labels`) and unlabeled
/// locations by file name (`analysis::tree_labels`), next to the given labels.
fn tree_input(args: &[PathBuf], fetch: Fetch<'_>) -> Input {
  log::info!(
    "input trees: {}",
    args
      .iter()
      .map(|p| p.display().to_string())
      .collect::<Vec<_>>()
      .join(" ")
  );
  let trees: Vec<TreeArg> = args.iter().map(|a| TreeArg::of(a)).collect();
  let labels = tree_arg_labels(&trees);
  let mut read_errors = Vec::new();
  let texts = trees
    .iter()
    .zip(labels)
    .enumerate()
    .map(|(i, (tree, label))| {
      let newick = tree.read(i, fetch).unwrap_or_else(|e| {
        read_errors.push((i, e));
        String::new()
      });
      TreeText::new(label, newick)
    })
    .collect();
  Input {
    texts,
    source: Source::Trees(trees.iter().map(TreeArg::name).collect()),
    extensions: trees.iter().map(TreeArg::extension).collect(),
    base: Settings::default(),
    addresses: trees.iter().map(TreeArg::address).collect(),
    session: false,
    read_errors,
  }
}

/// A tree argument: a file or a location, with the label it gives.
struct TreeArg {
  label: Option<String>,
  tree: TreeArgKind,
}

enum TreeArgKind {
  File(PathBuf),
  Location(Result<LinkLocation, LocationError>),
}

impl TreeArg {
  /// The tree argument `arg`: `[<label>=]<file or location>`, with the label rule of links
  /// (`launch::split_label`); a location starts with `https:`, `http:`, or `data:`.
  fn of(arg: &Path) -> Self {
    let Some(text) = arg.to_str() else {
      return TreeArg {
        label: None,
        tree: TreeArgKind::File(arg.to_path_buf()),
      };
    };
    let token = launch::split_label(text);
    let tree = if is_location(token.rest) {
      TreeArgKind::Location(launch::parse_location(token.rest))
    } else {
      TreeArgKind::File(PathBuf::from(token.rest))
    };
    TreeArg {
      label: token.label.map(str::to_owned),
      tree,
    }
  }

  /// The Newick text, or the error of the tree `index` that says why it cannot be read.
  fn read(&self, index: usize, fetch: Fetch<'_>) -> Result<String, ValidationError> {
    match &self.tree {
      TreeArgKind::File(path) => read_file(path).map_err(|e| read_error(index, format!("cannot read the file: {e}"))),
      TreeArgKind::Location(Ok(location)) => read_location(location, fetch).map_err(|e| read_error(index, e)),
      TreeArgKind::Location(Err(e)) => Err(read_error(index, e)),
    }
  }

  fn name(&self) -> String {
    match &self.tree {
      TreeArgKind::File(path) => path.display().to_string(),
      TreeArgKind::Location(Ok(LinkLocation::Url { url, .. })) => url.clone(),
      TreeArgKind::Location(_) => "data: tree".to_owned(),
    }
  }

  /// The extension of the output trees: that of the input file, or `.nwk` for a location.
  fn extension(&self) -> String {
    match &self.tree {
      TreeArgKind::File(path) => extension(path),
      TreeArgKind::Location(_) => ".nwk".to_owned(),
    }
  }

  /// The address of the tree in a share link; none for a file, or for a location that cannot be
  /// read, whose error `read` reports.
  fn address(&self) -> Option<TreeAddress> {
    match &self.tree {
      #[cfg_attr(
        dylint_lib = "custom",
        expect(
          error_dropped_by_pattern,
          reason = "`TreeArg::read` reports the error of the location"
        )
      )]
      TreeArgKind::File(_) | TreeArgKind::Location(Err(_)) => None,
      TreeArgKind::Location(Ok(LinkLocation::Url { url, .. })) => Some(TreeAddress::Url { url: url.clone() }),
      TreeArgKind::Location(Ok(LinkLocation::Data { .. })) => Some(TreeAddress::Data),
    }
  }
}

/// Labels of the tree arguments: the given labels; files without one by path, next to the given
/// labels; locations without one by the file name of their address (`tree` for `data:`), next to
/// all of those.
fn tree_arg_labels(trees: &[TreeArg]) -> Vec<String> {
  let given: Vec<String> = trees.iter().filter_map(|t| t.label.clone()).collect();
  let files: Vec<PathBuf> = trees
    .iter()
    .filter_map(|t| match (&t.label, &t.tree) {
      (None, TreeArgKind::File(path)) => Some(path.clone()),
      _ => None,
    })
    .collect();
  let file_labels = path_labels(&files, &given);
  let names: Vec<String> = trees
    .iter()
    .filter_map(|t| match (&t.label, &t.tree) {
      (None, TreeArgKind::Location(Ok(LinkLocation::Url { file_name, .. }))) => Some(file_name.clone()),
      (None, TreeArgKind::Location(_)) => Some(String::new()),
      _ => None,
    })
    .collect();
  let taken: Vec<String> = given.iter().chain(&file_labels).cloned().collect();
  let mut file_labels = file_labels.into_iter();
  let mut location_labels = analysis::tree_labels(&names, &taken).into_iter();
  trees
    .iter()
    .map(|t| match (&t.label, &t.tree) {
      (Some(label), _) => label.clone(),
      (None, TreeArgKind::File(_)) => file_labels.next().unwrap_or_default(),
      (None, TreeArgKind::Location(_)) => location_labels.next().unwrap_or_default(),
    })
    .collect()
}

/// Whether a tree argument is a location rather than a path: it starts with `https:`, `http:`
/// (rejected with a reason), or `data:`.
fn is_location(text: &str) -> bool {
  let lower = text.get(..6).unwrap_or(text).to_ascii_lowercase();
  ["https:", "http:", "data:"].iter().any(|s| lower.starts_with(s))
}

/// The validation error of the tree `index`, which cannot be read for `reason`.
fn read_error(index: usize, reason: impl Display) -> ValidationError {
  ValidationError::at(Field::Tree { index }, reason.to_string())
}

/// The text of the file at `path`, decompressed when it is gzip-compressed.
fn read_file(path: &Path) -> Result<String, ReadError> {
  let bytes = fs::read(path).map_err(ReadError::File)?;
  launch::decode_tree_bytes(&bytes).map_err(ReadError::FileText)
}

/// The text at `location`: downloaded with `fetch` from its address, or the text of `data:`.
fn read_location(location: &LinkLocation, fetch: Fetch<'_>) -> Result<String, ReadError> {
  match location {
    LinkLocation::Url {
      url, fetch: address, ..
    } => {
      log::info!("reading {url}");
      let bytes = fetch(address).map_err(|source| ReadError::Download {
        url: url.clone(),
        source,
      })?;
      launch::decode_tree_bytes(&bytes).map_err(|source| ReadError::DownloadText {
        url: url.clone(),
        source,
      })
    },
    LinkLocation::Data { text } => Ok(text.clone()),
  }
}

/// The trees and settings of the session file at `path`, a file or a location. Its trees keep
/// their labels, and their output files get the extensions of the web app.
fn session_input(path: &Path, fetch: Fetch<'_>) -> Result<Input> {
  log::info!("session file: {}", path.display());
  let name = path.display().to_string();
  let text = match path.to_str().filter(|p| is_location(p)) {
    Some(location) => launch::parse_location(location)
      .map_err(ReadError::from)
      .and_then(|l| read_location(&l, fetch)),
    None => read_file(path),
  };
  let text = text.map_err(|source| InputError::UnreadableSession {
    name: name.clone(),
    source,
  })?;
  let source = Source::Named(name);
  let request = match analysis::read_session(&text) {
    Ok(r) => r,
    Err(errors) => fail(&errors, &source)?,
  };
  Ok(Input::of_request(request, source, Vec::new()))
}

/// The trees of the example `id`, with the default settings.
fn example_input(id: &str) -> Result<Input> {
  let Some(example) = examples::example(id) else {
    let parsed = launch::parse_launch(&[("example".to_owned(), id.to_owned())], &[]);
    let message = parsed.errors.first().map_or("no such example", |e| e.message.as_str());
    return Err(InputError::UnknownExample(message.to_owned()).into());
  };
  log::info!("example: {id}");
  let request = AnalysisRequest {
    trees: example.tree_texts(),
    settings: Settings::default(),
  };
  let addresses = example
    .trees
    .iter()
    .map(|t| {
      Some(TreeAddress::Example {
        id: id.to_owned(),
        file: t.file.to_owned(),
      })
    })
    .collect();
  Ok(Input::of_request(
    request,
    Source::Named(format!("example {id}")),
    addresses,
  ))
}

/// The trees and settings of the link `url` of the web app. Its settings apply to the defaults
/// or to the settings of its session file; keys of the display are ignored.
fn link_input(url: &str, fetch: Fetch<'_>) -> Result<Input> {
  log::info!("link: {url}");
  let parsed = launch::parse_launch(&launch::link_pairs(url), &[]);
  if !parsed.ignored.is_empty() {
    let keys: Vec<String> = parsed
      .ignored
      .iter()
      .map(|k| match &k.suggestion {
        Some(s) => format!("{} (did you mean {s}?)", k.key),
        None => k.key.clone(),
      })
      .collect();
    log::info!("keys of the link that the command line ignores: {}", keys.join(", "));
  }
  if !parsed.errors.is_empty() {
    fail(&parsed.errors, &Source::Named("link".to_owned()))?;
  }
  let Some(link) = parsed.launch else {
    return Err(InputError::LinkWithoutTrees.into());
  };
  let mut input = match link.input {
    LaunchInput::Example { id } => example_input(&id)?,
    LaunchInput::Trees { trees } => {
      let mut read_errors = Vec::new();
      let texts = trees
        .iter()
        .enumerate()
        .map(|(i, t)| {
          let newick = read_location(&t.location, fetch).unwrap_or_else(|e| {
            read_errors.push((i, read_error(i, e)));
            String::new()
          });
          TreeText::new(t.label.clone(), newick)
        })
        .collect();
      let addresses = trees
        .iter()
        .map(|t| match &t.location {
          LinkLocation::Url { url, .. } => Some(TreeAddress::Url { url: url.clone() }),
          LinkLocation::Data { .. } => Some(TreeAddress::Data),
        })
        .collect();
      let names = trees
        .iter()
        .map(|t| match &t.location {
          LinkLocation::Url { url, .. } => url.clone(),
          LinkLocation::Data { .. } => format!("data: tree {}", t.label),
        })
        .collect();
      let request = AnalysisRequest {
        trees: texts,
        settings: Settings::default(),
      };
      Input {
        read_errors,
        ..Input::of_request(request, Source::Trees(names), addresses)
      }
    },
    LaunchInput::Session { location } => {
      let source = Source::Named("session file of the link".to_owned());
      let text = read_location(&location, fetch).map_err(InputError::UnreadableLinkSession)?;
      let request = match analysis::read_session(&text) {
        Ok(r) => r,
        Err(errors) => fail(&errors, &source)?,
      };
      Input::of_request(request, source, Vec::new())
    },
    LaunchInput::Message { .. } => {
      return Err(InputError::MessageLink.into());
    },
  };
  input.base = launch::apply(&input.base, &link.settings);
  Ok(input)
}

/// Labels of the command line for tree files: the file stem, or, when stems collide with each
/// other or with the labels `taken`, the stem and the parent directory (`a/ha.nwk` and
/// `b/ha.nwk` give `ha_a` and `ha_b`). Labels collide when their `analysis::label_key` is equal,
/// as in the label check, so `a/HA.nwk` and `b/ha.nwk` also get the parent directory; the label
/// check reports a collision that remains. The web app labels trees by file name only
/// (`treeknit_io::analysis::tree_labels`), because it has no directories.
fn path_labels(paths: &[PathBuf], taken: &[String]) -> Vec<String> {
  let stem = |p: &Path| {
    p.file_stem()
      .map(|s| s.to_string_lossy().into_owned())
      .unwrap_or_default()
  };
  let mut labels: Vec<String> = paths.iter().map(|p| stem(p)).collect();
  let unique = |v: &[String]| {
    let keys: Vec<String> = v.iter().chain(taken).map(|l| analysis::label_key(l)).collect();
    keys.iter().collect::<BTreeSet<_>>().len() == keys.len()
  };
  if !unique(&labels) {
    labels = paths
      .iter()
      .map(|p| {
        let dir = p
          .parent()
          .and_then(|d| d.file_name())
          .map(|d| d.to_string_lossy().into_owned())
          .unwrap_or_default();
        format!("{}_{dir}", stem(p))
      })
      .collect();
  }
  labels
}

/// File extension of `path` with its dot, or empty without one: the output trees of an input
/// file keep its extension.
fn extension(path: &Path) -> String {
  path
    .extension()
    .map(|e| format!(".{}", e.to_string_lossy()))
    .unwrap_or_default()
}

#[cfg(test)]
mod tests {
  use super::*;
  use crate::share_link;
  use crate::tests::cli;
  use pretty_assertions::assert_eq;
  use std::collections::BTreeMap;

  const HA: &str = "((A,B),(C,(D,X)));";
  const NA: &str = "((A,(B,X)),(C,D));";

  /// A fake of `http_get` that serves `files` by address and answers 404 otherwise.
  fn server(files: &[(&str, &str)]) -> impl Fn(&str) -> Result<Vec<u8>, DownloadError> + use<> {
    let files: BTreeMap<String, Vec<u8>> = files
      .iter()
      .map(|(url, text)| ((*url).to_owned(), text.as_bytes().to_vec()))
      .collect();
    move |url| files.get(url).cloned().ok_or(DownloadError::Status(404))
  }

  fn trees(trees: &[(&str, &str)]) -> Vec<TreeText> {
    trees
      .iter()
      .map(|(label, newick)| TreeText::new(*label, *newick))
      .collect()
  }

  fn url(u: &str) -> Option<TreeAddress> {
    Some(TreeAddress::Url { url: u.to_owned() })
  }

  #[test]
  fn https_tree_arguments_are_labeled_like_link_trees() {
    let fetch = server(&[("https://x/seg4.nwk", HA), ("https://x/na.nwk", NA)]);
    let input = read_input(&cli(&["HA=https://x/seg4.nwk", "https://x/na.nwk"]), &fetch).unwrap();
    let expected = (
      trees(&[("HA", HA), ("na", NA)]),
      vec![url("https://x/seg4.nwk"), url("https://x/na.nwk")],
      vec![".nwk".to_owned(), ".nwk".to_owned()],
    );
    assert_eq!(expected, (input.texts, input.addresses, input.extensions));
  }

  #[test]
  fn github_file_pages_are_read_from_their_raw_files() {
    let fetch = server(&[
      ("https://raw.githubusercontent.com/o/r/main/ha.nwk", HA),
      ("https://raw.githubusercontent.com/o/r/main/na.nwk", NA),
    ]);
    let args = [
      "https://github.com/o/r/blob/main/ha.nwk",
      "https://github.com/o/r/blob/main/na.nwk",
    ];
    let input = read_input(&cli(&args), &fetch).unwrap();
    assert_eq!(trees(&[("ha", HA), ("na", NA)]), input.texts);
  }

  #[test]
  fn session_file_at_an_address_is_read() {
    let session = format!(
      r#"{{"trees": [{{"label": "ha", "newick": "{HA}"}}, {{"label": "na", "newick": "{NA}"}}], "settings": {{"gamma": 3}}}}"#
    );
    let fetch = server(&[("https://x/s.json", &session)]);
    let input = read_input(&cli(&["--session", "https://x/s.json"]), &fetch).unwrap();
    let expected = (trees(&[("ha", HA), ("na", NA)]), 3.0_f64.to_bits(), true);
    assert_eq!(expected, (input.texts, input.base.gamma.to_bits(), input.session));
  }

  #[test]
  fn link_with_https_trees_is_read_and_its_printed_link_reads_back() {
    let fetch = server(&[("https://x/seg4.nwk", HA), ("https://x/na.nwk", NA)]);
    let link =
      "https://neherlab.github.io/treeknit/?tree=HA=https://x/seg4.nwk&tree=https://x/na.nwk&gamma=3&view=mccs";
    let input = read_input(&cli(&["--link", link]), &fetch).unwrap();
    assert_eq!(trees(&[("HA", HA), ("na", NA)]), input.texts);
    let request = AnalysisRequest {
      trees: input.texts.clone(),
      settings: input.base.clone(),
    };
    let printed = share_link(&request, &input.addresses).unwrap();
    assert_eq!(
      "https://neherlab.github.io/treeknit/?tree=HA=https://x/seg4.nwk&tree=https://x/na.nwk&gamma=3&run",
      printed
    );
    let again = read_input(&cli(&["--link", &printed]), &fetch).unwrap();
    assert_eq!((request.trees, request.settings), (again.texts, again.base));
  }

  #[test]
  fn missing_address_is_a_read_error_with_its_status() {
    let fetch = server(&[("https://x/na.nwk", NA)]);
    let input = read_input(&cli(&["https://x/ha.nwk", "https://x/na.nwk"]), &fetch).unwrap();
    let expected = vec![(
      0,
      ValidationError::at(Field::Tree { index: 0 }, "cannot read https://x/ha.nwk: 404 Not Found"),
    )];
    assert_eq!(expected, input.read_errors);
  }

  #[test]
  fn download_errors_name_the_status_and_the_limits() {
    let actual = [
      ureq::Error::StatusCode(404),
      ureq::Error::StatusCode(599),
      ureq::Error::BodyExceedsLimit(1),
      ureq::Error::HostNotFound,
    ]
    .map(|e| DownloadError::from(e).to_string());
    let expected = [
      "404 Not Found",
      "599",
      "the file is larger than 64 MiB",
      "host not found",
    ];
    assert_eq!(expected.map(str::to_owned), actual);
  }

  #[test]
  fn labeled_and_unlabeled_tree_arguments_share_the_labels() {
    // The explicit label "ha" is taken, so the files are labeled by path with their directory; a
    // `/` before the first `=` makes `c/a=b.nwk` a path; a data: tree gets the label `tree`.
    let args = [
      PathBuf::from("ha=a/na.nwk"),
      PathBuf::from("b/ha.nwk"),
      PathBuf::from("data:,(A,B);"),
      PathBuf::from("c/a=b.nwk"),
    ];
    let parsed: Vec<TreeArg> = args.iter().map(|a| TreeArg::of(a)).collect();
    assert_eq!(vec!["ha", "ha_b", "tree", "a=b_c"], tree_arg_labels(&parsed));
  }
}
