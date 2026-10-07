//! `treeknit`: infer maximally compatible clades (MCCs) and reassortment graphs from
//! segment trees.

use clap::parser::ValueSource;
use clap::{ArgMatches, CommandFactory, FromArgMatches, Parser};
use color_eyre::config::{HookBuilder, Theme};
use ctor::ctor;
use eyre::{Result, WrapErr};
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger};
use std::collections::BTreeSet;
use std::fmt::Display;
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::sync::{Arc, OnceLock};
use std::time::{Duration, Instant};
use treeknit_core::{Options, Resolution};
use treeknit_io::analysis::{self, AnalysisRequest, Field, SettingKey, Settings, TreeText, ValidationError};
use treeknit_io::examples;
use treeknit_io::launch::{self, DecodeError, LaunchInput, LinkLocation, LocationError, SettingsPatch, TreeAddress};
use treeknit_io::output::{self, OutputFile, OutputOptions};
use treeknit_io::schema::{KeyValue, SETTING_KEYS};
use treeknit_io::wire::wire_name;
use treeknit_io::{run, schema};

#[cfg(all(target_os = "linux", any(target_arch = "x86_64", target_arch = "aarch64")))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

const FORMER_OPTIONS_HELP: &str = "\
Former options are still accepted with their TreeKnit.jl meaning, and reproduce its
results: the method preset depends on the number of trees (--better-MCCs for two,
--better-trees for more), and --rounds counts all rounds (with --better-MCCs and more than
two trees, the default 2 means one resolving round and a final one without). They cannot be
mixed with --resolve, --pre-resolve, --no-final-round or --final-round. Closest current
equivalents:
  --better-trees         --resolve none --pre-resolve
  --better-MCCs          --resolve strict --pre-resolve
  --liberal-resolve      --resolve liberal (in the --better-MCCs preset)
  --no-resolve           --resolve none
  --match-topologies     --resolve matched
  --resolve-all-rounds   resolve in the final round too";

/// Heading of the options that choose the trees.
const INPUT_HEADING: &str = "Input";

/// Heading of the analysis options: the settings of [`SETTING_KEYS`], with the names of the keys
/// of links.
const ANALYSIS_HEADING: &str = "Analysis";

/// Heading of the options that choose the output.
const OUTPUT_HEADING: &str = "Output";

/// The text of `--help-resolve`: the resolution modes, the final round, and pre-resolution
/// with the words of the web app (`treeknit_io::schema`), then the former options.
fn resolve_help() -> String {
  let modes = schema::modes()
    .iter()
    .map(|m| {
      let name = wire_name(&m.mode);
      let default = if m.mode == analysis::ResolveMode::default() {
        "(default) "
      } else {
        ""
      };
      format!("  {name:<8} {default}{}", m.effect)
    })
    .collect::<Vec<_>>()
    .join("\n");
  format!(
    "Resolution of the trees (--resolve):\n{modes}\n\nFinal round (--no-final-round skips it): {}\n\n--pre-resolve: {}\n\n{FORMER_OPTIONS_HELP}",
    schema::FINAL_ROUND_HELP,
    schema::PRE_RESOLVE_HELP,
  )
}

/// The text of `--list-examples`: the id, the group, and the trees of each example.
fn example_list() -> String {
  examples::EXAMPLES
    .iter()
    .map(|e| {
      let labels = e.labels().join(", ");
      format!("{:<26} {:<30} {labels}", e.id, e.group.name())
    })
    .collect::<Vec<_>>()
    .join("\n")
}

/// Infer reassortment between segment trees: maximally compatible clades (MCCs) for every
/// pair of trees, resolved trees, and for two trees an ancestral reassortment graph (ARG).
#[derive(Parser, Debug)]
#[command(
  name = "treeknit",
  version = env!("TREEKNIT_LONG_VERSION"),
  after_help = "Use --help-resolve for details on how trees are resolved. The analysis options have the names of \
                the keys of web app links: --gamma 3 is gamma=3."
)]
struct Cli {
  /// Trees, one per segment (at least two): Newick files, gzip-compressed or not, or https:
  /// addresses. `<label>=<tree>` labels a tree; by default, the file name labels it.
  #[arg(
    required_unless_present_any = ["help_resolve", "help_defaults", "session", "example", "link", "list_examples"],
    value_name = "TREE",
    help_heading = INPUT_HEADING,
  )]
  trees: Vec<PathBuf>,

  /// Run the trees and settings of a session file (`treeknit_session.json`, saved by the web
  /// app), a path or an https: address. Analysis options change its settings.
  #[arg(
    long,
    value_name = "FILE",
    value_hint = clap::ValueHint::FilePath,
    conflicts_with_all = [
      "trees", "example", "link", "better_trees", "better_mccs", "no_resolve", "liberal_resolve", "match_topologies",
      "resolve_all_rounds",
    ],
    help_heading = INPUT_HEADING,
  )]
  session: Option<PathBuf>,

  /// Run a built-in example (see --list-examples). Analysis options change its settings.
  #[arg(
    long,
    value_name = "ID",
    conflicts_with_all = [
      "trees", "link", "better_trees", "better_mccs", "no_resolve", "liberal_resolve", "match_topologies",
      "resolve_all_rounds",
    ],
    help_heading = INPUT_HEADING,
  )]
  example: Option<String>,

  /// Run the trees and settings of a link of the web app. Analysis options change its settings.
  #[arg(
    long,
    value_name = "URL",
    conflicts_with_all = [
      "trees", "better_trees", "better_mccs", "no_resolve", "liberal_resolve", "match_topologies",
      "resolve_all_rounds",
    ],
    help_heading = INPUT_HEADING,
  )]
  link: Option<String>,

  /// List the built-in examples.
  #[arg(long, help_heading = INPUT_HEADING)]
  list_examples: bool,

  /// Output directory.
  #[arg(short, long, default_value = output::RESULTS_DIR, help_heading = OUTPUT_HEADING)]
  outdir: PathBuf,

  /// Cost γ of a reassortment (removing an MCC).
  #[arg(short, long, default_value_t = Settings::default().gamma, help_heading = ANALYSIS_HEADING)]
  gamma: f64,

  /// Sequence lengths of the segments, e.g. 1500,2000 (used by the likelihood tie-break and to
  /// weigh the drawn length of a split that some trees lack).
  #[arg(long, value_name = "LENGTHS", help_heading = ANALYSIS_HEADING)]
  seq_lengths: Option<String>,

  /// MCMC steps per leaf.
  #[arg(long, default_value_t = Settings::default().n_mcmc_it, help_heading = ANALYSIS_HEADING)]
  n_mcmc_it: u64,

  /// How trees are resolved: matched, strict, liberal or none (see --help-resolve).
  #[arg(long, value_enum, value_name = "MODE", help_heading = ANALYSIS_HEADING)]
  resolve: Option<analysis::ResolveMode>,

  /// Before inference, add to each tree the splits of other trees compatible with all trees.
  #[arg(long, overrides_with = "no_pre_resolve", help_heading = ANALYSIS_HEADING)]
  pre_resolve: bool,

  /// Do not pre-resolve the trees (the default).
  #[arg(long, overrides_with = "pre_resolve", help_heading = ANALYSIS_HEADING)]
  no_pre_resolve: bool,

  // No clap default: without the flag, the former options take the rounds of their preset.
  #[arg(
    long,
    help = format!("Rounds of pair inference [default: {}]", Settings::default().rounds),
    help_heading = ANALYSIS_HEADING,
  )]
  rounds: Option<u64>,

  /// With strict or liberal resolution and more than two trees, skip the final round that
  /// re-infers MCCs without resolution.
  #[arg(long, overrides_with = "final_round", help_heading = ANALYSIS_HEADING)]
  no_final_round: bool,

  /// Run the final round without resolution (the default).
  #[arg(long, overrides_with = "no_final_round", help_heading = ANALYSIS_HEADING)]
  final_round: bool,

  /// Seed of the random number generator.
  #[arg(long, default_value_t = Settings::default().seed, help_heading = ANALYSIS_HEADING)]
  seed: u64,

  /// Naive MCCs (γ → ∞).
  #[arg(long, overrides_with = "no_naive", help_heading = ANALYSIS_HEADING)]
  naive: bool,

  /// Infer MCCs (the default).
  #[arg(long, overrides_with = "naive", help_heading = ANALYSIS_HEADING)]
  no_naive: bool,

  /// Do not break ties between configurations with branch lengths.
  #[arg(long, overrides_with = "likelihood", help_heading = ANALYSIS_HEADING)]
  no_likelihood: bool,

  /// Break ties between configurations with branch lengths (the default).
  #[arg(long, overrides_with = "no_likelihood", help_heading = ANALYSIS_HEADING)]
  likelihood: bool,

  /// Worker threads for independent tree pairs (0: all cores).
  #[arg(long, default_value_t = 0)]
  threads: usize,

  /// Verbosity: -1 silent, 0 normal, 1 detailed, 2 debug.
  #[arg(long, default_value_t = 0, allow_negative_numbers = true)]
  verbosity_level: i32,

  /// Set verbosity to 1.
  #[arg(short, long)]
  verbose: bool,

  /// Explain the --resolve modes and the former method options.
  #[arg(long)]
  help_resolve: bool,

  /// Write trees with leaves missing from them placed by imputation (`*_imputed.nwk`).
  #[arg(long, help_heading = OUTPUT_HEADING)]
  impute: bool,

  /// Write auspice JSON files for tanglegram visualisation.
  #[arg(long, help_heading = OUTPUT_HEADING)]
  auspice_view: bool,

  /// Write SVG figures: a tanglegram of the resolved trees of each pair
  /// (`tanglegram_<a>_<b>.svg`) and, for two trees, the ARG (`ARG/arg.svg`).
  #[arg(long, help_heading = OUTPUT_HEADING)]
  plot: bool,

  /// After the run, print the link of the web app that runs the same analysis, and write the
  /// session file into the output directory.
  #[arg(
    long,
    conflicts_with_all = [
      "better_trees", "better_mccs", "no_resolve", "liberal_resolve", "match_topologies", "resolve_all_rounds",
    ],
    help_heading = OUTPUT_HEADING,
  )]
  print_link: bool,

  /// Accepted for compatibility; independent pairs always run in parallel (see --threads).
  #[arg(long, hide = true)]
  parallel: bool,

  // Former options, still accepted (see --help-resolve).
  #[arg(long, hide = true)]
  help_defaults: bool,
  #[arg(long, hide = true, conflicts_with = "better_mccs")]
  better_trees: bool,
  #[arg(long = "better-MCCs", hide = true)]
  better_mccs: bool,
  #[arg(long, hide = true)]
  no_resolve: bool,
  #[arg(long, hide = true)]
  liberal_resolve: bool,
  #[arg(long, hide = true)]
  match_topologies: bool,
  #[arg(long, hide = true)]
  resolve_all_rounds: bool,
}

#[ctor(unsafe)]
fn init() {
  global_init();
}

/// Install the color-eyre handler of errors and panics. Its colors are off when stderr is not a
/// terminal or `NO_COLOR` is set, because color-eyre writes escape codes into pipes otherwise.
fn global_init() {
  let theme = if io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none() {
    Theme::dark()
  } else {
    Theme::new()
  };
  #[expect(
    clippy::expect_used,
    reason = "runs once before main, when no other handler is installed"
  )]
  HookBuilder::default()
    .display_env_section(false)
    .theme(theme)
    .panic_section(format!("Report this bug at {}/issues", env!("CARGO_PKG_REPOSITORY")))
    .install()
    .expect("the color-eyre handler installs once");
}

/// Run the command line. A mistake in the input prints its plain list with exit code 1; color-eyre
/// renders any other error with its chain of causes.
fn main() -> Result<ExitCode> {
  match run() {
    Ok(()) => Ok(ExitCode::SUCCESS),
    Err(report) => match report.downcast_ref::<InputError>() {
      Some(error) => {
        #[cfg_attr(
          dylint_lib = "custom",
          expect(debug_remnants, reason = "the CLI reports input errors on stderr")
        )]
        eprintln!("Error: {error}");
        Ok(ExitCode::FAILURE)
      },
      None => Err(report),
    },
  }
}

fn run() -> Result<()> {
  let matches = Cli::command().get_matches();
  let cli = Cli::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());
  if cli.help_resolve || cli.help_defaults {
    println!("{}", resolve_help());
    return Ok(());
  }
  if cli.list_examples {
    println!("{}", example_list());
    return Ok(());
  }
  // The log stays in memory until the input passes validation, so that a run with invalid input
  // leaves no results directory behind.
  let log_file = LogFile::default();
  setup_logging(&cli, &log_file)?;
  if cli.threads > 0 {
    rayon_threads(cli.threads)?;
  }

  log::info!("TreeKnit {}", env!("TREEKNIT_LONG_VERSION"));
  let input = read_input(&cli, &http_get)?;
  log::info!("results directory: {}", cli.outdir.display());
  let output_options = OutputOptions {
    extensions: input.extensions.clone(),
    imputed: cli.impute,
    auspice: cli.auspice_view,
    figures: cli.plot,
  };

  let k = input.texts.len();
  let settings = (!uses_former_options(&cli)).then(|| run_settings(&cli, &matches, &input));
  let opts = match &settings {
    Some((s, errors)) => match analysis::options(s, k, true) {
      Ok(o) if errors.is_empty() => Ok(o),
      result => Err(errors.iter().cloned().chain(result.err().unwrap_or_default()).collect()),
    },
    None => former_options_checked(&cli, k),
  };
  // A file that cannot be read stops the run with its read error, and the checks of the other
  // trees still run.
  let (parsed, opts) = match analysis::prepare(&input.texts, &output_options, opts) {
    Ok(prepared) if input.read_errors.is_empty() => prepared,
    result => fail(&input.with_read_errors(result.err().unwrap_or_default()), &input.source)?,
  };
  let settings = settings.map(|(s, _)| s);
  let seed = settings.as_ref().map_or(cli.seed, |s| s.seed);
  run::report_overlap(&parsed.trees, &parsed.taxa);
  fs::create_dir_all(&cli.outdir).wrap_err_with(|| format!("creating {}", cli.outdir.display()))?;
  let log_path = cli.outdir.join(output::LOG_FILE);
  log_file
    .open(&log_path)
    .wrap_err_with(|| format!("writing {}", log_path.display()))?;
  log_options(&opts, parsed.trees.len());
  log::debug!("parameters: {opts:?}");
  write_file(&cli.outdir, &output::parameters_file(&opts, seed))?;
  let request = settings.map(|settings| AnalysisRequest {
    trees: input.texts.clone(),
    settings,
  });
  if let Some(r) = request.as_ref().filter(|_| input.session || cli.print_link) {
    write_file(&cli.outdir, &output::session_file(r))?;
  }

  let start = Instant::now();
  let result = run::run(parsed, &opts, seed, &|_| {});
  log::info!(
    "found {:?} MCCs (runtime {:.2}s)",
    result.pairs().iter().map(|p| p.mccs.len()).collect::<Vec<_>>(),
    start.elapsed().as_secs_f64()
  );

  log::info!("writing results in {}", cli.outdir.display());
  for file in output::output_files(&result, &output_options)? {
    write_file(&cli.outdir, &file)?;
  }
  if let Some(r) = request.as_ref().filter(|_| cli.print_link) {
    match share_link(r, &input.addresses) {
      Ok(link) => println!("{link}"),
      Err(length) => log::warn!(
        "the link would have {length} characters, more than {}; share the session file {} instead",
        launch::MAX_LINK_CHARS,
        cli.outdir.join(output::SESSION_FILE).display()
      ),
    }
  }
  Ok(())
}

/// A mistake in the input: the validation errors of the trees and settings, a link or example
/// that names nothing to run, or a session file that cannot be read. `main` prints it as a plain
/// list, while any other error is a failure that color-eyre renders with its chain of causes.
#[derive(Debug, thiserror::Error)]
enum InputError {
  /// The validation errors, one per line.
  #[error("{}", .0.join("\n"))]
  Invalid(Vec<String>),
  #[error("{0} (see --list-examples)")]
  UnknownExample(String),
  #[error("the link names no trees: it needs example=, tree=, or session=")]
  LinkWithoutTrees,
  #[error("from= receives the trees from another page of the browser; open this link in the web app")]
  MessageLink,
  /// The session file or address given on the command line.
  #[error("reading {name}: {source}")]
  UnreadableSession { name: String, source: ReadError },
  /// The session file of a link.
  #[error("{0}")]
  UnreadableLinkSession(#[source] ReadError),
}

/// Why the text of a file or location cannot be read.
#[derive(Debug, thiserror::Error)]
enum ReadError {
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
type Fetch<'a> = &'a dyn Fn(&str) -> Result<Vec<u8>, DownloadError>;

/// The bytes at the `https:` address `url`, with the limits of the web app: an answer within
/// [`launch::FETCH_TIMEOUT_SECONDS`] and at most [`launch::MAX_DOWNLOAD_BYTES`].
fn http_get(url: &str) -> Result<Vec<u8>, DownloadError> {
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
enum DownloadError {
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
struct Input {
  texts: Vec<TreeText>,
  /// Where the trees and settings come from, for error messages.
  source: Source,
  /// Extension of each tree's output files.
  extensions: Vec<String>,
  /// The settings before the analysis options: the defaults, or the settings of the session file
  /// or of the link.
  base: Settings,
  /// Where a link gets each tree again.
  addresses: Vec<Option<TreeAddress>>,
  /// The trees come with their labels and settings (a session file, an example, or a link), so the
  /// results directory receives their session file.
  session: bool,
  /// The index and the error of each input file that cannot be read; its tree has an empty text.
  read_errors: Vec<(usize, ValidationError)>,
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
  fn with_read_errors(&self, errors: Vec<ValidationError>) -> Vec<ValidationError> {
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
enum Source {
  /// The name of each tree: its file path or its address; the settings come from the options.
  Trees(Vec<String>),
  /// The session file, example, or link that holds the trees and the settings, by name.
  Named(String),
}

/// The input that the options of `cli` name, read with `fetch` from `https:` addresses.
fn read_input(cli: &Cli, fetch: Fetch<'_>) -> Result<Input> {
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
      TreeText { label, newick }
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

  fn address(&self) -> Option<TreeAddress> {
    match &self.tree {
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
      (None, TreeArgKind::Location(Ok(LinkLocation::Url { url, .. }))) => Some(launch::url_file_name(url)),
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
  ValidationError {
    field: Some(Field::Tree { index }),
    message: reason.to_string(),
    line: None,
    column: None,
  }
}

/// The text of the file at `path`, decompressed when it is gzip-compressed.
fn read_file(path: &Path) -> Result<String, ReadError> {
  let bytes = fs::read(path).map_err(ReadError::File)?;
  launch::decode_tree_bytes(&bytes).map_err(ReadError::FileText)
}

/// The text at `location`: downloaded with `fetch` from its address, or the text of `data:`.
fn read_location(location: &LinkLocation, fetch: Fetch<'_>) -> Result<String, ReadError> {
  match location {
    LinkLocation::Url { url, fetch: address } => {
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
          TreeText {
            label: t.label.clone(),
            newick,
          }
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

/// The settings of the run: the analysis options given on the command line applied to the
/// settings of the input, with the error of `--seq-lengths`, which leaves the sequence lengths of
/// the input. The log names each option that changes the settings of a session file, an example,
/// or a link.
fn run_settings(cli: &Cli, matches: &ArgMatches, input: &Input) -> (Settings, Vec<ValidationError>) {
  let (patch, errors) = match options_patch(cli, matches, cli.seq_lengths.as_deref().map(parse_lengths)) {
    (patch, Some(Err(e))) => (patch, vec![e]),
    (patch, _) => (patch, Vec::new()),
  };
  if input.session {
    for key in patch_keys(&patch) {
      log::info!("--{key} changes the setting of the input");
    }
  }
  (launch::apply(&input.base, &patch), errors)
}

/// The analysis options given on the command line as a patch of the settings, with the sequence
/// lengths `lengths` when they parse; the parse result comes back. Each flag and its opposite
/// override each other, so at most one is set.
fn options_patch(
  cli: &Cli,
  matches: &ArgMatches,
  lengths: Option<Result<Vec<f64>, ValidationError>>,
) -> (SettingsPatch, Option<Result<Vec<f64>, ValidationError>>) {
  let given = |id: &str| matches.value_source(id) == Some(ValueSource::CommandLine);
  let pair = |on: bool, off: bool| {
    if on {
      Some(true)
    } else if off {
      Some(false)
    } else {
      None
    }
  };
  let patch = SettingsPatch {
    gamma: given("gamma").then_some(cli.gamma),
    seq_lengths: lengths.clone().and_then(Result::ok),
    n_mcmc_it: given("n_mcmc_it").then_some(cli.n_mcmc_it),
    resolve: cli.resolve,
    pre_resolve: pair(cli.pre_resolve, cli.no_pre_resolve),
    rounds: cli.rounds,
    final_round: pair(cli.final_round, cli.no_final_round),
    likelihood: pair(cli.likelihood, cli.no_likelihood),
    naive: pair(cli.naive, cli.no_naive),
    seed: given("seed").then_some(cli.seed),
  };
  (patch, lengths)
}

/// The keys of the settings that `patch` sets, as the options that set them: `gamma`,
/// `no-final-round`.
fn patch_keys(patch: &SettingsPatch) -> Vec<&'static str> {
  let flag = |value: Option<bool>, key: &'static str, opposite: Option<&'static str>, key_sets: bool| {
    value.map(|v| if v == key_sets { key } else { opposite.unwrap_or(key) })
  };
  SETTING_KEYS
    .iter()
    .filter_map(|s| {
      let set = |v: bool| v.then_some(s.key);
      match (s.setting, s.value) {
        (schema::SettingName::Gamma, _) => set(patch.gamma.is_some()),
        (schema::SettingName::SeqLengths, _) => set(patch.seq_lengths.is_some()),
        (schema::SettingName::NMcmcIt, _) => set(patch.n_mcmc_it.is_some()),
        (schema::SettingName::Resolve, _) => set(patch.resolve.is_some()),
        (schema::SettingName::Rounds, _) => set(patch.rounds.is_some()),
        (schema::SettingName::Seed, _) => set(patch.seed.is_some()),
        (schema::SettingName::PreResolve, KeyValue::Flag { key_sets }) => {
          flag(patch.pre_resolve, s.key, s.opposite, key_sets)
        },
        (schema::SettingName::FinalRound, KeyValue::Flag { key_sets }) => {
          flag(patch.final_round, s.key, s.opposite, key_sets)
        },
        (schema::SettingName::Likelihood, KeyValue::Flag { key_sets }) => {
          flag(patch.likelihood, s.key, s.opposite, key_sets)
        },
        (schema::SettingName::Naive, KeyValue::Flag { key_sets }) => flag(patch.naive, s.key, s.opposite, key_sets),
        (
          schema::SettingName::PreResolve
          | schema::SettingName::FinalRound
          | schema::SettingName::Likelihood
          | schema::SettingName::Naive,
          _,
        ) => None,
      }
    })
    .collect()
}

/// The link of the web app that runs `request` (`run` included), whose trees have `addresses`:
/// the canonical link when every tree has an address, otherwise an inline session in the
/// fragment; or the length of that link when it is longer than [`launch::MAX_LINK_CHARS`].
fn share_link(request: &AnalysisRequest, addresses: &[Option<TreeAddress>]) -> Result<String, usize> {
  if let Some(pairs) = launch::launch_pairs(request, addresses, &Settings::default(), true) {
    return Ok(launch::web_link(&pairs, &[]));
  }
  let link = launch::web_link(
    &[("run".to_owned(), String::new())],
    &[("session".to_owned(), launch::inline_session(request))],
  );
  let length = link.chars().count();
  if length > launch::MAX_LINK_CHARS {
    Err(length)
  } else {
    Ok(link)
  }
}

/// Write `file` at its path below `dir`, creating its parent directories.
fn write_file(dir: &Path, file: &OutputFile) -> Result<()> {
  let path = dir.join(&file.path);
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).wrap_err_with(|| format!("creating {}", parent.display()))?;
  }
  fs::write(&path, &file.text).wrap_err_with(|| format!("writing {}", path.display()))
}

/// Stop with every validation error, one per line. For tree arguments, an error of a tree starts
/// with its path or address, and with the line and column in the file when it has them. For a
/// session file, an example, or a link, every error starts with its name and the field of the
/// error, and an error in a tree's Newick text gives the line and column in that text.
fn fail<T>(errors: &[ValidationError], source: &Source) -> Result<T> {
  let lines: Vec<String> = errors
    .iter()
    .map(|e| match source {
      Source::Trees(names) => {
        let name = e.field.as_ref().and_then(Field::tree_index).and_then(|i| names.get(i));
        match (name, e.line, e.column) {
          (Some(n), Some(line), Some(column)) => format!("{n}:{line}:{column}: {e}"),
          (Some(n), ..) => format!("{n}: {e}"),
          (None, ..) => e.to_string(),
        }
      },
      Source::Named(name) => match (&e.field, e.line, e.column) {
        (Some(field), Some(line), Some(column)) => {
          format!("{name}: {}: line {line}, column {column}: {e}", field.path())
        },
        (Some(field), ..) => format!("{name}: {}: {e}", field.path()),
        (None, ..) => format!("{name}: {e}"),
      },
    })
    .collect();
  Err(InputError::Invalid(lines).into())
}

/// Options of the former options for `k` trees, or every error of the flags and of the shared
/// settings checks. Independent pairs run in parallel.
fn former_options_checked(cli: &Cli, k: usize) -> Result<Options, Vec<ValidationError>> {
  let mut errors = Vec::new();
  let seq_lengths = match cli.seq_lengths.as_deref().map(parse_lengths).transpose() {
    Ok(v) => v,
    Err(e) => {
      errors.push(e);
      None
    },
  };
  if cli.resolve.is_some() || cli.pre_resolve || cli.no_final_round || cli.final_round {
    errors.push(ValidationError {
      field: None,
      message: "former method options (--better-trees, --better-MCCs, --no-resolve, --liberal-resolve, \
                --resolve-all-rounds, --match-topologies) cannot be combined with --resolve, --pre-resolve, \
                --no-final-round or --final-round; see --help-resolve"
        .to_owned(),
      line: None,
      column: None,
    });
  }
  // The former options have no settings of their own; the values they share with the
  // settings get the same checks. The other fields keep valid defaults, and the rounds of a
  // preset (1 or 2) are valid.
  let shared = Settings {
    gamma: cli.gamma,
    seq_lengths,
    n_mcmc_it: cli.n_mcmc_it,
    rounds: cli.rounds.unwrap_or_else(|| Settings::default().rounds),
    seed: cli.seed,
    ..Settings::default()
  };
  // The options of the shared values, which the settings checks bound, so they convert exactly.
  let checked = match analysis::options(&shared, k, true) {
    Ok(o) if errors.is_empty() => o,
    result => {
      errors.extend(result.err().unwrap_or_default());
      return Err(errors);
    },
  };
  let mut o = former_options(cli, k, cli.rounds.map(|_| checked.rounds));
  o.gamma = checked.gamma;
  o.n_mcmc = checked.n_mcmc;
  o.likelihood_sort = !cli.no_likelihood;
  o.naive = cli.naive;
  o.seq_lengths = checked.seq_lengths;
  Ok(o)
}

/// Sequence lengths of `--seq-lengths`: numbers separated by commas or whitespace, so the value of
/// a link (`1701,1410`) works as well as `"1701 1410"`.
fn parse_lengths(s: &str) -> Result<Vec<f64>, ValidationError> {
  s.split(|c: char| c == ',' || c.is_whitespace())
    .filter(|v| !v.is_empty())
    .map(str::parse::<f64>)
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| ValidationError {
      field: Some(Field::Setting {
        key: SettingKey::SeqLengths,
      }),
      message: format!("--seq-lengths should look like 1500,2000, got {s:?}: {e}"),
      line: None,
      column: None,
    })
}

fn log_options(o: &Options, k: usize) {
  let extra = matches!(o.resolution, Resolution::Strict | Resolution::Liberal) && k > 2 && o.final_unresolved_round;
  log::info!(
    "γ = {}, resolution: {}, pre-resolve: {}, {} round(s){}",
    o.gamma,
    wire_name(&analysis::ResolveMode::from(o.resolution)),
    o.pre_resolve,
    o.rounds,
    if extra { " + final round without resolution" } else { "" }
  );
}

fn uses_former_options(cli: &Cli) -> bool {
  cli.better_trees
    || cli.better_mccs
    || cli.no_resolve
    || cli.liberal_resolve
    || cli.resolve_all_rounds
    || cli.match_topologies
}

/// Options for the former command line (TreeKnit.jl semantics), which reproduce its results:
/// method presets depending on the number of trees, `--rounds` counting all rounds, and
/// `--resolve-all-rounds` resolving in the final round too. `rounds_flag` is the checked value of
/// `--rounds` when it is given.
fn former_options(cli: &Cli, k: usize, rounds_flag: Option<usize>) -> Options {
  for (used, flag) in [
    (cli.better_trees, "--better-trees"),
    (cli.better_mccs, "--better-MCCs"),
    (cli.no_resolve, "--no-resolve"),
    (cli.liberal_resolve, "--liberal-resolve"),
    (cli.resolve_all_rounds, "--resolve-all-rounds"),
    (cli.match_topologies, "--match-topologies"),
  ] {
    if used {
      log::warn!("{flag} is deprecated; see --help-resolve for the --resolve options");
    }
  }
  // The former configuration: method preset, then individual flags.
  let better_trees = cli.better_trees || (!cli.better_mccs && k > 2);
  let (mut resolve, mut final_no_resolve, mut rounds) = if better_trees {
    (false, true, 1)
  } else if k > 2 {
    (true, true, 2)
  } else {
    (true, false, 1)
  };
  if final_no_resolve && rounds == 1 && resolve {
    resolve = false;
    final_no_resolve = false;
  }
  let mut strict = true;
  let mut pre_resolve = true;
  let mut matched = false;
  if cli.no_pre_resolve {
    pre_resolve = false;
  }
  if cli.no_resolve {
    resolve = false;
  }
  if cli.liberal_resolve {
    strict = false;
  }
  if cli.resolve_all_rounds {
    final_no_resolve = false;
  }
  if let Some(r) = rounds_flag {
    rounds = r;
  }
  if cli.match_topologies {
    matched = true;
    final_no_resolve = false;
    if !cli.no_resolve {
      resolve = true;
    }
  }
  // The same in terms of the current options. In the former pipeline, a round resolved
  // unless it was the final one with `final_no_resolve`.
  let mut o = Options::for_trees(k);
  o.pre_resolve = pre_resolve;
  o.sort_strict = Some(strict && !final_no_resolve);
  let mode = if strict {
    Resolution::Strict
  } else {
    Resolution::Liberal
  };
  if matched && !resolve {
    // matching after unresolved inference changes nothing, but sorted liberally
    o.sort_strict = Some(false);
  }
  (o.resolution, o.rounds, o.final_unresolved_round) = if matched && resolve {
    if !strict {
      log::warn!("--liberal-resolve has no effect with --match-topologies");
    }
    (Resolution::Matched, rounds, false)
  } else if !resolve || (final_no_resolve && rounds == 1) {
    (Resolution::None, rounds, false)
  } else if final_no_resolve {
    (mode, rounds - 1, true)
  } else {
    (mode, rounds, false)
  };
  o
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

/// Configure the log: the terminal at the verbosity of `cli`, and `file` with debug records too.
fn setup_logging(cli: &Cli, file: &LogFile) -> Result<()> {
  let v = if cli.verbose && cli.verbosity_level == 0 {
    1
  } else {
    cli.verbosity_level
  };
  let term_level = match v {
    i32::MIN..=-1 => LevelFilter::Off,
    0 => LevelFilter::Info,
    1 => LevelFilter::Debug,
    _ => LevelFilter::Trace,
  };
  let config = ConfigBuilder::new()
    .set_time_format_rfc3339()
    .set_target_level(LevelFilter::Off)
    .build();
  CombinedLogger::init(vec![
    TermLogger::new(term_level, config.clone(), TerminalMode::Stderr, ColorChoice::Auto),
    WriteLogger::new(LevelFilter::Debug.max(term_level), config, file.writer()),
  ])?;
  Ok(())
}

/// The file `log.txt`, named once the input has passed validation.
#[derive(Default)]
struct LogFile(Arc<OnceLock<fs::File>>);

impl LogFile {
  /// Create the file at `path`. The records so far reach it before the next record.
  fn open(&self, path: &Path) -> io::Result<()> {
    let file = fs::File::create(path)?;
    self
      .0
      .set(file)
      .map_err(|_unused_file| io::Error::other("the log file is already open"))
  }

  /// The writer of the logger: it keeps the records in memory until the file is open.
  fn writer(&self) -> LogWriter {
    LogWriter {
      file: Arc::clone(&self.0),
      memory: Vec::new(),
    }
  }
}

/// Writer of `log.txt` for the logger; see `LogFile`.
struct LogWriter {
  file: Arc<OnceLock<fs::File>>,
  /// The records written before the file was open.
  memory: Vec<u8>,
}

impl Write for LogWriter {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match self.file.get() {
      Some(mut file) => {
        if !self.memory.is_empty() {
          file.write_all(&std::mem::take(&mut self.memory))?;
        }
        file.write(buf)
      },
      None => self.memory.write(buf),
    }
  }

  fn flush(&mut self) -> io::Result<()> {
    self.file.get().map_or(Ok(()), |mut file| file.flush())
  }
}

fn rayon_threads(n: usize) -> Result<()> {
  rayon::ThreadPoolBuilder::new()
    .num_threads(n)
    .build_global()
    .wrap_err("configuring threads")
}

#[cfg(test)]
mod tests {
  use super::*;
  use clap::ValueEnum;
  use pretty_assertions::assert_eq;
  use std::collections::BTreeMap;
  use std::iter;
  use strum::VariantArray;

  const HA: &str = "((A,B),(C,(D,X)));";
  const NA: &str = "((A,(B,X)),(C,D));";

  fn cli(args: &[&str]) -> Cli {
    Cli::try_parse_from(iter::once("treeknit").chain(args.iter().copied())).unwrap()
  }

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
      .map(|(label, newick)| TreeText {
        label: (*label).to_owned(),
        newick: (*newick).to_owned(),
      })
      .collect()
  }

  fn url(u: &str) -> Option<TreeAddress> {
    Some(TreeAddress::Url { url: u.to_owned() })
  }

  #[test]
  fn resolve_values_are_the_serde_names_of_the_modes() {
    // Oracle: the names that session files and links write.
    let values: Vec<String> = analysis::ResolveMode::value_variants()
      .iter()
      .filter_map(ValueEnum::to_possible_value)
      .map(|v| v.get_name().to_owned())
      .collect();
    let names: Vec<String> = analysis::ResolveMode::VARIANTS.iter().map(wire_name).collect();
    assert_eq!(names, values);
  }

  #[test]
  fn setting_keys_are_the_analysis_options() {
    // A renamed flag fails here instead of breaking the links that name it.
    let command = Cli::command();
    let options: BTreeSet<&str> = command
      .get_arguments()
      .filter(|a| a.get_help_heading() == Some(ANALYSIS_HEADING))
      .filter_map(|a| a.get_long())
      .collect();
    let keys: BTreeSet<&str> = SETTING_KEYS
      .iter()
      .flat_map(|s| iter::once(s.key).chain(s.opposite))
      .collect();
    assert_eq!(keys, options);
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
      "https://neherlab.github.io/treeknit-rs/?tree=HA=https://x/seg4.nwk&tree=https://x/na.nwk&gamma=3&view=mccs";
    let input = read_input(&cli(&["--link", link]), &fetch).unwrap();
    assert_eq!(trees(&[("HA", HA), ("na", NA)]), input.texts);
    let request = AnalysisRequest {
      trees: input.texts.clone(),
      settings: input.base.clone(),
    };
    let printed = share_link(&request, &input.addresses).unwrap();
    assert_eq!(
      "https://neherlab.github.io/treeknit-rs/?tree=HA=https://x/seg4.nwk&tree=https://x/na.nwk&gamma=3&run",
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
      ValidationError {
        field: Some(Field::Tree { index: 0 }),
        message: "cannot read https://x/ha.nwk: 404 Not Found".to_owned(),
        line: None,
        column: None,
      },
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
