//! `treeknit`: infer maximally compatible clades (MCCs) and reassortment graphs from
//! segment trees.

use anyhow::{Context, Result, bail};
use clap::{Parser, ValueEnum};
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger};
use std::collections::BTreeSet;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock};
use std::time::Instant;
use treeknit_core::{Options, Resolution};
use treeknit_io::analysis::{self, AnalysisRequest, Settings, TreeText, ValidationError};
use treeknit_io::output::{self, OutputFile, OutputOptions};
use treeknit_io::{run, schema};

const FORMER_OPTIONS_HELP: &str = "\
Former options are still accepted with their TreeKnit.jl meaning, and reproduce its
results: the method preset depends on the number of trees (--better-MCCs for two,
--better-trees for more), and --rounds counts all rounds (with --better-MCCs and more than
two trees, the default 2 means one resolving round and a final one without). They cannot be
mixed with --resolve, --pre-resolve or --no-final-round. Closest current equivalents:
  --better-trees         --resolve none --pre-resolve
  --better-MCCs          --resolve strict --pre-resolve
  --liberal-resolve      --resolve liberal (in the --better-MCCs preset)
  --no-resolve           --resolve none
  --match-topologies     --resolve matched
  --no-pre-resolve       the default
  --resolve-all-rounds   resolve in the final round too";

/// The text of `--help-resolve`: the resolution modes, the final round, and pre-resolution
/// with the words of the web app (`treeknit_io::schema`), then the former options.
fn resolve_help() -> String {
  let modes = schema::modes()
    .iter()
    .map(|m| {
      let name = resolve_value(m.mode);
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

/// The value of `--resolve` that selects `mode`.
fn resolve_value(mode: analysis::ResolveMode) -> String {
  #[expect(
    clippy::expect_used,
    reason = "`From<ResolveMode>` maps the variants one to one, and none is skipped"
  )]
  ResolveMode::value_variants()
    .iter()
    .filter(|&&v| analysis::ResolveMode::from(v) == mode)
    .find_map(ValueEnum::to_possible_value)
    .expect("every resolution mode of the settings has a --resolve value")
    .get_name()
    .to_owned()
}

/// How trees are resolved (see --help-resolve).
#[derive(ValueEnum, Clone, Copy, Debug)]
enum ResolveMode {
  None,
  Strict,
  Liberal,
  Matched,
}

impl From<ResolveMode> for analysis::ResolveMode {
  fn from(m: ResolveMode) -> analysis::ResolveMode {
    match m {
      ResolveMode::None => analysis::ResolveMode::None,
      ResolveMode::Strict => analysis::ResolveMode::Strict,
      ResolveMode::Liberal => analysis::ResolveMode::Liberal,
      ResolveMode::Matched => analysis::ResolveMode::Matched,
    }
  }
}

/// Infer reassortment between segment trees: maximally compatible clades (MCCs) for every
/// pair of trees, resolved trees, and for two trees an ancestral reassortment graph (ARG).
#[derive(Parser, Debug)]
#[command(
  name = "treeknit",
  version = env!("TREEKNIT_LONG_VERSION"),
  after_help = "Use --help-resolve for details on how trees are resolved."
)]
struct Cli {
  /// Newick files, one tree per segment (at least two).
  #[arg(required_unless_present_any = ["help_resolve", "help_defaults", "request"])]
  trees: Vec<PathBuf>,

  /// Run the trees and settings of a session file (`treeknit_request.json`, saved by the web
  /// app) instead of tree files and analysis options.
  #[arg(
    long,
    value_name = "FILE",
    value_hint = clap::ValueHint::FilePath,
    conflicts_with_all = [
      "trees", "gamma", "seq_lengths", "n_mcmc_it", "resolve", "pre_resolve", "rounds", "no_final_round",
      "no_likelihood", "naive", "seed", "better_trees", "better_mccs", "no_pre_resolve", "no_resolve",
      "liberal_resolve", "match_topologies", "resolve_all_rounds",
    ],
  )]
  request: Option<PathBuf>,

  /// Output directory.
  #[arg(short, long, default_value = output::RESULTS_DIR)]
  outdir: PathBuf,

  /// Cost γ of a reassortment (removing an MCC).
  #[arg(short, long, default_value_t = Settings::default().gamma)]
  gamma: f64,

  /// Sequence lengths of the segments, e.g. "1500 2000" (used by the likelihood tie-break).
  #[arg(long, value_name = "LENGTHS")]
  seq_lengths: Option<String>,

  /// MCMC steps per leaf.
  #[arg(long, default_value_t = Settings::default().n_mcmc_it)]
  n_mcmc_it: usize,

  /// How trees are resolved: matched, strict, liberal or none (see --help-resolve).
  #[arg(long, value_enum, value_name = "MODE")]
  resolve: Option<ResolveMode>,

  /// Before inference, add to each tree the splits of other trees compatible with all trees.
  #[arg(long)]
  pre_resolve: bool,

  // No clap default: without the flag, the former options take the rounds of their preset.
  #[arg(long, help = format!("Rounds of pair inference [default: {}]", Settings::default().rounds))]
  rounds: Option<usize>,

  /// With strict or liberal resolution and more than two trees, skip the final round that
  /// re-infers MCCs without resolution.
  #[arg(long)]
  no_final_round: bool,

  /// Seed of the random number generator.
  #[arg(long, default_value_t = Settings::default().seed)]
  seed: u64,

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

  /// Naive MCCs (γ → ∞).
  #[arg(long)]
  naive: bool,

  /// Do not break ties between configurations with branch lengths.
  #[arg(long)]
  no_likelihood: bool,

  /// Write trees with leaves missing from them placed by imputation (`*_imputed.nwk`).
  #[arg(long)]
  impute: bool,

  /// Write auspice JSON files for tanglegram visualisation.
  #[arg(long)]
  auspice_view: bool,

  /// Write SVG figures: a tanglegram of the resolved trees of each pair
  /// (`tanglegram_<a>_<b>.svg`) and, for two trees, the ARG (`ARG/arg.svg`).
  #[arg(long)]
  plot: bool,

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
  no_pre_resolve: bool,
  #[arg(long, hide = true)]
  no_resolve: bool,
  #[arg(long, hide = true)]
  liberal_resolve: bool,
  #[arg(long, hide = true)]
  match_topologies: bool,
  #[arg(long, hide = true)]
  resolve_all_rounds: bool,
}

fn main() -> Result<()> {
  let cli = Cli::parse();
  if cli.help_resolve || cli.help_defaults {
    println!("{}", resolve_help());
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
  let input = match &cli.request {
    Some(path) => request_input(path)?,
    None => tree_file_input(&cli),
  };
  log::info!("results directory: {}", cli.outdir.display());
  let output_options = OutputOptions {
    extensions: input.extensions.clone(),
    imputed: cli.impute,
    auspice: cli.auspice_view,
    figures: cli.plot,
  };

  // A file that cannot be read has an empty text, which fails to parse; its read error replaces
  // that parse error, and the checks of the other trees still run.
  let parsed = analysis::parse_trees(&input.texts)
    .and_then(|p| {
      let errors = output::check_output_paths(&analysis::labels(&input.texts), &output_options);
      if errors.is_empty() { Ok(p) } else { Err(errors) }
    })
    .map_err(|errors| {
      let unread: Vec<String> = input
        .read_errors
        .iter()
        .filter_map(|e| e.field.as_ref().map(|f| format!("{f}.newick")))
        .collect();
      let mut all = input.read_errors.clone();
      all.extend(
        errors
          .into_iter()
          .filter(|e| !e.field.as_ref().is_some_and(|f| unread.contains(f))),
      );
      all
    });
  if let Ok(p) = &parsed {
    run::report_overlap(&p.trees, &p.taxa);
  }
  let k = input.texts.len();
  let opts = match &input.request {
    Some(r) => analysis::options(&r.settings, k, true),
    None => options(&cli, k),
  };
  let (parsed, opts) = match (parsed, opts) {
    (Ok(p), Ok(o)) => (p, o),
    (parsed, opts) => {
      let mut errors = parsed.err().unwrap_or_default();
      errors.extend(opts.err().unwrap_or_default());
      fail(&errors, &input.source)?
    },
  };
  fs::create_dir_all(&cli.outdir).with_context(|| format!("creating {}", cli.outdir.display()))?;
  let log_path = cli.outdir.join(output::LOG_FILE);
  log_file
    .open(&log_path)
    .with_context(|| format!("writing {}", log_path.display()))?;
  log_options(&opts, parsed.trees.len());
  log::debug!("parameters: {opts:?}");
  write_file(&cli.outdir, &output::parameters_file(&opts, input.seed))?;
  if let Some(r) = &input.request {
    write_file(&cli.outdir, &output::request_file(r))?;
  }

  let start = Instant::now();
  let result = run::run(parsed, &opts, input.seed, &|_| {});
  log::info!(
    "found {:?} MCCs (runtime {:.2}s)",
    result.pairs.iter().map(|p| p.mccs.len()).collect::<Vec<_>>(),
    start.elapsed().as_secs_f64()
  );

  log::info!("writing results in {}", cli.outdir.display());
  for file in output::output_files(&result, &opts, &output_options) {
    write_file(&cli.outdir, &file)?;
  }
  Ok(())
}

/// The trees of a run with what the command line needs to report on them and to name their
/// output files.
struct Input {
  texts: Vec<TreeText>,
  /// Where the trees and settings come from, for error messages.
  source: Source,
  /// Extension of each tree's output files.
  extensions: Vec<String>,
  seed: u64,
  /// The session file, whose settings replace the analysis options.
  request: Option<AnalysisRequest>,
  /// Errors of the input files that cannot be read; their trees have an empty text.
  read_errors: Vec<ValidationError>,
}

/// The trees of the positional tree files, labeled by path, with the seed of `--seed`, and the
/// error of each file that cannot be read.
fn tree_file_input(cli: &Cli) -> Input {
  log::info!(
    "input trees: {}",
    cli
      .trees
      .iter()
      .map(|p| p.display().to_string())
      .collect::<Vec<_>>()
      .join(" ")
  );
  let mut read_errors = Vec::new();
  let texts = cli
    .trees
    .iter()
    .zip(path_labels(&cli.trees))
    .enumerate()
    .map(|(i, (path, label))| {
      let newick = fs::read_to_string(path).unwrap_or_else(|e| {
        read_errors.push(ValidationError {
          field: Some(format!("trees[{i}]")),
          message: format!("cannot read the file: {e}"),
          line: None,
          column: None,
        });
        String::new()
      });
      TreeText { label, newick }
    })
    .collect();
  Input {
    texts,
    source: Source::TreeFiles(cli.trees.clone()),
    extensions: cli.trees.iter().map(|p| extension(p)).collect(),
    seed: cli.seed,
    request: None,
    read_errors,
  }
}

/// The trees and settings of the session file at `path`. Its trees keep their labels, and their
/// output files get the extension `.nwk`, as in the web app.
fn request_input(path: &Path) -> Result<Input> {
  log::info!("session file: {}", path.display());
  let text = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
  let request = match analysis::read_request(&text) {
    Ok(r) => r,
    Err(errors) => {
      let lines: Vec<String> = errors.iter().map(|e| format!("{}: {e}", path.display())).collect();
      bail!("{}", lines.join("\n"))
    },
  };
  Ok(Input {
    texts: request.trees.clone(),
    source: Source::Request(path.to_path_buf()),
    extensions: vec![".nwk".to_owned(); request.trees.len()],
    seed: request.settings.seed,
    request: Some(request),
    read_errors: Vec::new(),
  })
}

/// Where the trees and the settings of a run come from.
enum Source {
  /// The input file of each tree; the settings come from the flags.
  TreeFiles(Vec<PathBuf>),
  /// The session file that holds the trees and the settings.
  Request(PathBuf),
}

/// Write `file` at its path below `dir`, creating its parent directories.
fn write_file(dir: &Path, file: &OutputFile) -> Result<()> {
  let path = dir.join(&file.path);
  if let Some(parent) = path.parent() {
    fs::create_dir_all(parent).with_context(|| format!("creating {}", parent.display()))?;
  }
  fs::write(&path, &file.text).with_context(|| format!("writing {}", path.display()))
}

/// Stop with every validation error, one per line. For tree files, an error of a tree starts
/// with the path of its input file, and with the line and column in the file when it has them.
/// For a session file, every error starts with the path of the session file and the field of the
/// error, and an error in a tree's Newick text gives the line and column in that text.
fn fail<T>(errors: &[ValidationError], source: &Source) -> Result<T> {
  let lines: Vec<String> = errors
    .iter()
    .map(|e| match source {
      Source::TreeFiles(paths) => {
        let path = e.field.as_deref().and_then(tree_index).and_then(|i| paths.get(i));
        match (path, e.line, e.column) {
          (Some(p), Some(line), Some(column)) => format!("{}:{line}:{column}: {e}", p.display()),
          (Some(p), ..) => format!("{}: {e}", p.display()),
          (None, ..) => e.to_string(),
        }
      },
      Source::Request(path) => match (&e.field, e.line, e.column) {
        (Some(field), Some(line), Some(column)) => {
          format!("{}: {field}: line {line}, column {column}: {e}", path.display())
        },
        (Some(field), ..) => format!("{}: {field}: {e}", path.display()),
        (None, ..) => format!("{}: {e}", path.display()),
      },
    })
    .collect();
  bail!("{}", lines.join("\n"))
}

/// Index `i` of an error field `trees[i]` or `trees[i].<field>`.
fn tree_index(field: &str) -> Option<usize> {
  field.strip_prefix("trees[")?.split_once(']')?.0.parse().ok()
}

/// Options of the flags for `k` trees, or every error of the flags and of the shared settings
/// checks. Independent pairs run in parallel.
fn options(cli: &Cli, k: usize) -> Result<Options, Vec<ValidationError>> {
  let mut errors = Vec::new();
  let seq_lengths = match cli.seq_lengths.as_deref().map(parse_lengths).transpose() {
    Ok(v) => v,
    Err(e) => {
      errors.push(e);
      None
    },
  };
  if !uses_former_options(cli) {
    let s = Settings {
      gamma: cli.gamma,
      seq_lengths,
      n_mcmc_it: cli.n_mcmc_it,
      resolve: cli.resolve.map_or_else(analysis::ResolveMode::default, Into::into),
      pre_resolve: cli.pre_resolve,
      rounds: cli.rounds.unwrap_or_else(|| Settings::default().rounds),
      final_round: !cli.no_final_round,
      likelihood: !cli.no_likelihood,
      naive: cli.naive,
      seed: cli.seed,
    };
    return match analysis::options(&s, k, true) {
      Ok(o) if errors.is_empty() => Ok(o),
      result => {
        errors.extend(result.err().unwrap_or_default());
        Err(errors)
      },
    };
  }
  if cli.resolve.is_some() || cli.pre_resolve || cli.no_final_round {
    errors.push(ValidationError {
      field: None,
      message: "former method options (--better-trees, --better-MCCs, --no-resolve, --liberal-resolve, \
                --resolve-all-rounds, --no-pre-resolve, --match-topologies) cannot be combined with \
                --resolve, --pre-resolve or --no-final-round; see --help-resolve"
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
  errors.extend(analysis::check_settings(&shared, k));
  if !errors.is_empty() {
    return Err(errors);
  }
  let mut o = former_options(cli, k);
  o.gamma = cli.gamma;
  o.n_mcmc = cli.n_mcmc_it;
  o.likelihood_sort = !cli.no_likelihood;
  o.naive = cli.naive;
  if let Some(v) = shared.seq_lengths {
    o.seq_lengths = v;
  }
  Ok(o)
}

/// Sequence lengths of `--seq-lengths`, numbers separated by whitespace.
fn parse_lengths(s: &str) -> Result<Vec<f64>, ValidationError> {
  s.split_whitespace()
    .map(str::parse::<f64>)
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| ValidationError {
      field: Some("settings.seqLengths".to_owned()),
      message: format!("--seq-lengths should look like \"1500 2000\", got {s:?}: {e}"),
      line: None,
      column: None,
    })
}

fn log_options(o: &Options, k: usize) {
  let extra = matches!(o.resolution, Resolution::Strict | Resolution::Liberal) && k > 2 && o.final_unresolved_round;
  log::info!(
    "γ = {}, resolution: {}, pre-resolve: {}, {} round(s){}",
    o.gamma,
    format!("{:?}", o.resolution).to_lowercase(),
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
    || cli.no_pre_resolve
    || cli.match_topologies
}

/// Options for the former command line (TreeKnit.jl semantics), which reproduce its results:
/// method presets depending on the number of trees, `--rounds` counting all rounds, and
/// `--resolve-all-rounds` resolving in the final round too.
fn former_options(cli: &Cli, k: usize) -> Options {
  for (used, flag) in [
    (cli.better_trees, "--better-trees"),
    (cli.better_mccs, "--better-MCCs"),
    (cli.no_resolve, "--no-resolve"),
    (cli.liberal_resolve, "--liberal-resolve"),
    (cli.resolve_all_rounds, "--resolve-all-rounds"),
    (cli.no_pre_resolve, "--no-pre-resolve"),
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
  if let Some(r) = cli.rounds {
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

/// Labels of the command line for tree files: the file stem, or, when stems collide, the stem
/// and the parent directory (`a/ha.nwk` and `b/ha.nwk` give `ha_a` and `ha_b`). Labels collide
/// when their `analysis::label_key` is equal, as in the label check, so `a/HA.nwk` and
/// `b/ha.nwk` also get the parent directory; the label check reports a collision that remains.
/// The web app labels trees by file name only (`treeknit_io::analysis::tree_labels`), because it
/// has no directories.
fn path_labels(paths: &[PathBuf]) -> Vec<String> {
  let stem = |p: &Path| {
    p.file_stem()
      .map(|s| s.to_string_lossy().into_owned())
      .unwrap_or_default()
  };
  let mut labels: Vec<String> = paths.iter().map(|p| stem(p)).collect();
  let unique = |v: &[String]| v.iter().map(|l| analysis::label_key(l)).collect::<BTreeSet<_>>().len() == v.len();
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
    .context("configuring threads")
}
