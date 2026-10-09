//! `treeknit`: infer maximally compatible clades (MCCs) and reassortment graphs from
//! segment trees.

mod former;
mod input;
mod logging;

use clap::parser::ValueSource;
use clap::{ArgMatches, CommandFactory, FromArgMatches, Parser};
use color_eyre::config::{HookBuilder, Theme};
use ctor::ctor;
use eyre::{Result, WrapErr};
use former::{former_options_checked, uses_former_options};
use input::{Input, ReadError, Source, http_get, read_input};
use logging::{LogFile, setup_logging};
use std::fs;
use std::io::{self, IsTerminal, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Instant;
use treeknit_core::Options;
use treeknit_io::analysis::{self, AnalysisRequest, Field, SettingKey, Settings, ValidationError};
use treeknit_io::examples;
use treeknit_io::launch::{self, SettingsPatch, TreeAddress};
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

#[ctor(unsafe)]
fn init() {
  global_init();
}

/// Install the color-eyre handler of errors and panics, with colors only when `stderr_has_colors`.
fn global_init() {
  let theme = if stderr_has_colors() {
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

/// Whether the error report and the log color their text: stderr is a terminal and `NO_COLOR` is
/// not set. Neither color-eyre nor termcolor checks for a terminal, so both would write escape
/// codes into pipes and files.
fn stderr_has_colors() -> bool {
  io::stderr().is_terminal() && std::env::var_os("NO_COLOR").is_none()
}

/// Run the command line. A mistake in the input prints its plain list with exit code 1; color-eyre
/// renders any other error with its chain of causes.
fn main() -> Result<ExitCode> {
  match run() {
    Ok(()) => Ok(ExitCode::SUCCESS),
    Err(report) => match report.downcast_ref::<InputError>() {
      #[cfg_attr(
        dylint_lib = "custom",
        expect(debug_remnants, reason = "the CLI reports input errors on stderr")
      )]
      Some(error) => {
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
    return write_stdout(&format!("{}\n", resolve_help()));
  }
  if cli.help_markdown {
    return write_stdout(&reference_markdown());
  }
  if cli.list_examples {
    return write_stdout(&format!("{}\n", example_list()));
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
      Ok(link) => write_stdout(&format!("{link}\n"))?,
      Err(length) => log::warn!(
        "the link would have {length} characters, more than {}; share the session file {} instead",
        launch::MAX_LINK_CHARS,
        cli.outdir.join(output::SESSION_FILE).display()
      ),
    }
  }
  Ok(())
}

/// Write `text` to stdout, which carries the texts and the link that the user asks for. A closed
/// pipe, such as `treeknit --help-markdown | head`, is an error instead of the panic of `print!`.
fn write_stdout(text: &str) -> Result<()> {
  io::stdout()
    .lock()
    .write_all(text.as_bytes())
    .wrap_err("cannot write to stdout")
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
  let flag = |value: Option<bool>, key: &'static str, opposite: &'static str, key_sets: bool| {
    value.map(|v| if v == key_sets { key } else { opposite })
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
        (schema::SettingName::PreResolve, KeyValue::Flag { key_sets, opposite }) => {
          flag(patch.pre_resolve, s.key, opposite, key_sets)
        },
        (schema::SettingName::FinalRound, KeyValue::Flag { key_sets, opposite }) => {
          flag(patch.final_round, s.key, opposite, key_sets)
        },
        (schema::SettingName::Likelihood, KeyValue::Flag { key_sets, opposite }) => {
          flag(patch.likelihood, s.key, opposite, key_sets)
        },
        (schema::SettingName::Naive, KeyValue::Flag { key_sets, opposite }) => {
          flag(patch.naive, s.key, opposite, key_sets)
        },
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

/// Sequence lengths of `--seq-lengths`: numbers separated by commas or whitespace, so the value of
/// a link (`1701,1410`) works as well as `"1701 1410"`.
fn parse_lengths(s: &str) -> Result<Vec<f64>, ValidationError> {
  s.split(|c: char| c == ',' || c.is_whitespace())
    .filter(|v| !v.is_empty())
    .map(str::parse::<f64>)
    .collect::<Result<Vec<_>, _>>()
    .map_err(|e| {
      ValidationError::at(
        Field::Setting {
          key: SettingKey::SeqLengths,
        },
        format!("--seq-lengths should look like 1500,2000, got {s:?}: {e}"),
      )
    })
}

fn log_options(o: &Options, k: usize) {
  let extra = o.extra_unresolved_round(k);
  log::info!(
    "γ = {}, resolution: {}, pre-resolve: {}, {} round(s){}",
    o.gamma,
    wire_name(&analysis::ResolveMode::from(o.resolution)),
    o.pre_resolve,
    o.rounds,
    if extra { " + final round without resolution" } else { "" }
  );
}

fn rayon_threads(n: usize) -> Result<()> {
  rayon::ThreadPoolBuilder::new()
    .num_threads(n)
    .build_global()
    .wrap_err("configuring threads")
}

/// The command-line reference of `docs/user/cli.md`: the text of `--help` and of `--help-resolve`,
/// and the table of output files of `treeknit_io::output::file_reference`.
fn reference_markdown() -> String {
  let help = Cli::command().render_help().to_string();
  let files = output::file_reference()
    .into_iter()
    .map(|file| {
      let paths = file
        .paths
        .iter()
        .map(|path| format!("`{path}`"))
        .collect::<Vec<_>>()
        .join(", ");
      format!("| {paths} | {} |", file.description)
    })
    .collect::<Vec<_>>()
    .join("\n");
  format!(
    "<!-- Written by `just gen` from `treeknit --help-markdown`. Edit the command-line definitions in \
     `packages/treeknit-cli/src/main.rs` and the file list in `packages/treeknit-io/src/output/kinds.rs` \
     instead of this file. -->\n\n\
     # Command-line reference\n\n\
     This page holds the help texts of `treeknit` and the list of the files that it writes. \
     `treeknit --help` and `treeknit --help-resolve` print the same texts.\n\n\
     ## Options\n\n```text\n{help}\n```\n\n\
     ## Resolution of the trees\n\n```text\n{resolve}\n```\n\n\
     ## Output files\n\n\
     The files go into the results directory, `{dir}` unless `--outdir` names another one. In the \
     names, `<tree>` stands for the label of a tree, and `<a>` and `<b>` for the labels of the two \
     trees of a pair.\n\n\
     | File | Content |\n|---|---|\n{files}\n",
    help = help.trim_end(),
    resolve = resolve_help(),
    dir = output::RESULTS_DIR,
  )
}

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

/// Infers reassortment from the trees of the segments of a virus, such as the HA and NA trees of influenza
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
    required_unless_present_any = [
      "help_resolve", "help_defaults", "help_markdown", "session", "example", "link", "list_examples",
    ],
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

  #[arg(short, long, default_value_t = Settings::default().gamma, help = schema::GAMMA_HELP, help_heading = ANALYSIS_HEADING)]
  gamma: f64,

  /// Sequence lengths of the segments, e.g. 1500,2000 (used by the likelihood tie-break and to
  /// weigh the drawn length of a split that some trees lack).
  #[arg(long, value_name = "LENGTHS", help_heading = ANALYSIS_HEADING)]
  seq_lengths: Option<String>,

  #[arg(long, default_value_t = Settings::default().n_mcmc_it, help = schema::N_MCMC_IT_HELP, help_heading = ANALYSIS_HEADING)]
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
    help = format!("{} [default: {}]", schema::ROUNDS_HELP, Settings::default().rounds),
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

  #[arg(long, default_value_t = Settings::default().seed, help = schema::SEED_HELP, help_heading = ANALYSIS_HEADING)]
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

  /// Worker threads for independent tree pairs (0: all cores). Pairs run in parallel only in
  /// rounds without resolution: with `--resolve none`, and in the final round of `strict` and
  /// `liberal` with more than two trees.
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

  /// Print the command-line reference in Markdown, as committed in `docs/user/cli.md`.
  #[arg(long, hide = true)]
  help_markdown: bool,

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

  /// Accepted for compatibility and without effect; independent pairs run in parallel where
  /// that gives the same result (see --threads).
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

#[cfg(test)]
mod tests {
  use super::*;
  use clap::ValueEnum;
  use pretty_assertions::assert_eq;
  use std::collections::BTreeSet;
  use std::iter;
  use strum::VariantArray;

  pub(crate) fn cli(args: &[&str]) -> Cli {
    Cli::try_parse_from(iter::once("treeknit").chain(args.iter().copied())).unwrap()
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
      .flat_map(|s| iter::once(s.key).chain(s.opposite()))
      .collect();
    assert_eq!(keys, options);
  }
}
