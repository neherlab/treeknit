//! `treeknit`: infer maximally compatible clades (MCCs) and reassortment graphs from
//! segment trees.

use anyhow::{bail, Context, Result};
use clap::Parser;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use treeknit_core::{Options, Resolution, Taxa, Tree};
use treeknit_io::{arg, auspice, mccs, newick};

const RESOLVE_HELP: &str = "\
Resolution of the trees (--resolve):
  matched  (default) resolve during inference and with the inferred MCCs, then resolve all
           trees so that their topologies match within every MCC. Where splits from
           different trees conflict, trees given earlier take precedence, and MCCs that
           cannot be matched are split.
  strict   resolve during inference and with the inferred MCCs, unambiguous splits only.
  liberal  as strict, also adding ambiguous splits.
  none     no resolution with MCCs; MCCs then require identical topologies.
With strict or liberal and more than two trees, MCCs are re-inferred without resolution in a
final extra round, since resolving later pairs can invalidate earlier pairs' MCCs
(--resolve-all-rounds skips it).

--pre-resolve adds to each tree, before inference, the splits of other trees that are
compatible with all trees. It is mostly useful with --resolve none.

Former options (still accepted):
  --better-trees     = --resolve none --pre-resolve
  --better-MCCs      = --resolve strict --pre-resolve
  --no-resolve       = --resolve none
  --liberal-resolve  = --resolve liberal
  --match-topologies = --resolve matched
  --no-pre-resolve   is now the default";

/// How trees are resolved (see --help-resolve).
#[derive(clap::ValueEnum, Clone, Copy, Debug)]
enum ResolveMode {
    None,
    Strict,
    Liberal,
    Matched,
}

impl From<ResolveMode> for Resolution {
    fn from(m: ResolveMode) -> Resolution {
        match m {
            ResolveMode::None => Resolution::None,
            ResolveMode::Strict => Resolution::Strict,
            ResolveMode::Liberal => Resolution::Liberal,
            ResolveMode::Matched => Resolution::Matched,
        }
    }
}

/// Infer reassortment between segment trees: maximally compatible clades (MCCs) for every
/// pair of trees, resolved trees, and for two trees an ancestral reassortment graph (ARG).
#[derive(Parser, Debug)]
#[command(
    name = "treeknit",
    version,
    after_help = "Use --help-resolve for details on how trees are resolved."
)]
struct Cli {
    /// Newick files, one tree per segment (at least two).
    #[arg(required_unless_present_any = ["help_resolve", "help_defaults"])]
    trees: Vec<PathBuf>,

    /// Output directory.
    #[arg(short, long, default_value = "treeknit_results")]
    outdir: PathBuf,

    /// Cost γ of a reassortment (removing an MCC).
    #[arg(short, long, default_value_t = 2.0)]
    gamma: f64,

    /// Sequence lengths of the segments, e.g. "1500 2000" (used by the likelihood tie-break).
    #[arg(long, value_name = "LENGTHS")]
    seq_lengths: Option<String>,

    /// MCMC steps per leaf.
    #[arg(long, default_value_t = 50)]
    n_mcmc_it: usize,

    /// How trees are resolved: matched, strict, liberal or none (see --help-resolve).
    #[arg(long, value_enum, value_name = "MODE")]
    resolve: Option<ResolveMode>,

    /// Before inference, add to each tree the splits of other trees compatible with all trees.
    #[arg(long)]
    pre_resolve: bool,

    /// Rounds of pair inference.
    #[arg(long, default_value_t = 1)]
    rounds: usize,

    /// Seed of the random number generator.
    #[arg(long, default_value_t = 1)]
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
    /// With strict or liberal resolution and more than two trees, skip the final round that
    /// re-infers MCCs without resolution.
    #[arg(long)]
    resolve_all_rounds: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.help_resolve || cli.help_defaults {
        println!("{RESOLVE_HELP}");
        return Ok(());
    }
    if cli.trees.len() < 2 {
        bail!("need at least two tree files");
    }
    fs::create_dir_all(&cli.outdir).with_context(|| format!("creating {}", cli.outdir.display()))?;
    setup_logging(&cli)?;
    if cli.threads > 0 {
        rayon_threads(cli.threads)?;
    }

    log::info!("TreeKnit {}", env!("CARGO_PKG_VERSION"));
    log::info!(
        "input trees: {}",
        cli.trees
            .iter()
            .map(|p| p.display().to_string())
            .collect::<Vec<_>>()
            .join(" ")
    );
    log::info!("results directory: {}", cli.outdir.display());

    let labels = tree_labels(&cli.trees)?;
    let mut trees: Vec<Tree> = Vec::new();
    for (path, label) in cli.trees.iter().zip(&labels) {
        let s = fs::read_to_string(path).with_context(|| format!("reading {}", path.display()))?;
        trees.push(newick::parse_first(&s, label).with_context(|| format!("parsing {}", path.display()))?);
    }
    let taxa = Taxa::from_trees(&trees);
    for t in trees.iter_mut() {
        t.assign_taxa(&taxa).map_err(anyhow::Error::msg)?;
    }
    report_overlap(&trees, &taxa);

    let opts = options(&cli, trees.len())?;
    log::debug!("parameters: {opts:?}");
    fs::write(
        cli.outdir.join("parameters.json"),
        serde_json::to_string_pretty(&params_json(&opts, cli.seed))?,
    )?;

    let start = Instant::now();
    let pairs = treeknit_core::run(&mut trees, &taxa, &opts, cli.seed);
    log::info!(
        "found {:?} MCCs (runtime {:.2}s)",
        pairs.iter().map(|p| p.mccs.len()).collect::<Vec<_>>(),
        start.elapsed().as_secs_f64()
    );

    log::info!("writing results in {}", cli.outdir.display());
    let json = mccs::to_json(&pairs, &trees, &taxa);
    fs::write(
        cli.outdir.join("MCCs.json"),
        serde_json::to_string_pretty(&json)? + "\n",
    )?;
    // Legacy text format of TreeKnit.jl < 0.5 (one MCC per line): `MCCs.dat` for two trees,
    // `MCCs_<a>_<b>.dat` per pair otherwise.
    for p in &pairs {
        let name = if trees.len() == 2 {
            "MCCs.dat".to_string()
        } else {
            format!("MCCs_{}_{}.dat", trees[p.i].label, trees[p.j].label)
        };
        let names: Vec<Vec<String>> = p.mccs.iter().map(|m| taxa.names_of(m)).collect();
        fs::write(cli.outdir.join(name), mccs::to_lines(&names))?;
    }
    for (t, path) in trees.iter().zip(&cli.trees) {
        fs::write(cli.outdir.join(out_name(path, "_resolved")), newick::write(t) + "\n")?;
    }
    if cli.impute {
        let imputed = treeknit_core::imputed_trees(&trees, &pairs, taxa.len());
        for (t, path) in imputed.iter().zip(&cli.trees) {
            fs::write(cli.outdir.join(out_name(path, "_imputed")), newick::write(t) + "\n")?;
        }
    }
    if cli.auspice_view {
        for (i, t) in trees.iter().enumerate() {
            let v = auspice::auspice_json(i, &trees, &pairs, &taxa);
            fs::write(
                cli.outdir.join(format!("auspice_{}.json", t.label)),
                serde_json::to_string_pretty(&v)?,
            )?;
        }
    }
    if trees.len() == 2 && !pairs[0].mccs.is_empty() {
        write_arg(&cli, &trees, &pairs[0], &taxa)?;
    }
    Ok(())
}

/// ARG of the two trees, the liberally resolved trees it was built from, and the node table.
fn write_arg(cli: &Cli, trees: &[Tree], pair: &treeknit_core::PairResult, taxa: &Taxa) -> Result<()> {
    log::debug!("building ARG from trees and MCCs");
    let (t1, t2, m) = treeknit_core::arg_inputs(trees, pair, taxa.len());
    let arg = match treeknit_core::arg::arg_from_trees(&t1, &t2, &m, taxa.len()) {
        Ok(a) => a,
        Err(e) => {
            log::error!("{e}; no ARG written");
            return Ok(());
        }
    };
    log::info!("found {} reassortments in the ARG", arg.n_hybrids());
    let dir = cli.outdir.join("ARG");
    fs::create_dir_all(&dir)?;
    fs::write(dir.join("arg.nwk"), arg::extended_newick(&arg) + "\n")?;
    fs::write(dir.join("nodes.dat"), arg::node_table(&arg) + "\n")?;
    for (t, path) in arg.trees.iter().zip(&cli.trees) {
        fs::write(dir.join(out_name(path, "_liberal_resolved")), newick::write(t) + "\n")?;
    }
    Ok(())
}

fn options(cli: &Cli, k: usize) -> Result<Options> {
    let mut o = Options::for_trees(k);
    // Former method options, applied before the explicit --resolve / --pre-resolve.
    let former: [(bool, &str, &str); 6] = [
        (cli.better_trees, "--better-trees", "--resolve none --pre-resolve"),
        (cli.better_mccs, "--better-MCCs", "--resolve strict --pre-resolve"),
        (cli.no_resolve, "--no-resolve", "--resolve none"),
        (cli.liberal_resolve, "--liberal-resolve", "--resolve liberal"),
        (cli.match_topologies, "--match-topologies", "--resolve matched"),
        (cli.no_pre_resolve, "--no-pre-resolve", "the default (no --pre-resolve)"),
    ];
    for (used, flag, now) in former {
        if used {
            log::warn!("{flag} is deprecated; it now means {now}");
        }
    }
    if cli.better_trees {
        o.resolution = Resolution::None;
        o.pre_resolve = true;
    }
    if cli.better_mccs {
        o.resolution = Resolution::Strict;
        o.pre_resolve = true;
    }
    if cli.no_resolve {
        o.resolution = Resolution::None;
    }
    if cli.liberal_resolve {
        o.resolution = Resolution::Liberal;
    }
    if cli.match_topologies {
        o.resolution = Resolution::Matched;
    }
    if let Some(m) = cli.resolve {
        o.resolution = m.into();
    }
    if cli.pre_resolve {
        o.pre_resolve = true;
    }
    if cli.no_pre_resolve {
        o.pre_resolve = false;
    }
    if cli.resolve_all_rounds {
        o.final_unresolved_round = false;
    }
    if cli.rounds == 0 {
        bail!("--rounds must be at least 1");
    }
    o.rounds = cli.rounds;
    o.gamma = cli.gamma;
    o.n_mcmc = cli.n_mcmc_it;
    o.likelihood_sort = !cli.no_likelihood;
    o.naive = cli.naive;
    if let Some(s) = &cli.seq_lengths {
        let v: Vec<f64> = s
            .split_whitespace()
            .map(|x| x.parse::<f64>())
            .collect::<Result<_, _>>()
            .context("--seq-lengths should look like \"1500 2000\"")?;
        if v.len() != k {
            bail!("--seq-lengths: got {} values for {k} trees", v.len());
        }
        o.seq_lengths = v;
    }
    let extra = matches!(o.resolution, Resolution::Strict | Resolution::Liberal) && k > 2 && o.final_unresolved_round;
    log::info!(
        "γ = {}, resolution: {}, pre-resolve: {}, {} round(s){}",
        o.gamma,
        format!("{:?}", o.resolution).to_lowercase(),
        o.pre_resolve,
        o.rounds,
        if extra { " + final round without resolution" } else { "" }
    );
    Ok(o)
}

fn params_json(o: &Options, seed: u64) -> serde_json::Value {
    serde_json::json!({
        "gamma": o.gamma,
        "itmax": o.itmax,
        "likelihood_sort": o.likelihood_sort,
        "resolution": format!("{:?}", o.resolution).to_lowercase(),
        "seq_lengths": o.seq_lengths,
        "pre_resolve": o.pre_resolve,
        "rounds": o.rounds,
        "final_unresolved_round": o.final_unresolved_round,
        "nMCMC": o.n_mcmc,
        "sa_rep": o.sa_rep,
        "Tmin": o.t_min,
        "Tmax": o.t_max,
        "nT": o.n_t,
        "cooling_schedule": format!("{:?}", o.cooling).to_lowercase(),
        "naive": o.naive,
        "seed": seed,
    })
}

/// Tree labels from file names; if names collide, append the parent directory name.
fn tree_labels(paths: &[PathBuf]) -> Result<Vec<String>> {
    let stem = |p: &Path| {
        p.file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default()
    };
    let mut labels: Vec<String> = paths.iter().map(|p| stem(p)).collect();
    let unique = |v: &[String]| v.iter().collect::<std::collections::HashSet<_>>().len() == v.len();
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
    if !unique(&labels) {
        bail!("input trees must be identifiable by file name");
    }
    Ok(labels)
}

fn out_name(path: &Path, suffix: &str) -> String {
    let stem = path.file_stem().unwrap().to_string_lossy();
    let ext = path
        .extension()
        .map(|e| format!(".{}", e.to_string_lossy()))
        .unwrap_or_default();
    format!("{stem}{suffix}{ext}")
}

fn report_overlap(trees: &[Tree], taxa: &Taxa) {
    let n = taxa.len();
    for t in trees {
        let missing = n - t.n_leaves();
        if missing > 0 {
            log::info!(
                "tree {}: {} of {n} leaves missing (placed by imputation)",
                t.label,
                missing
            );
        }
    }
}

fn setup_logging(cli: &Cli) -> Result<()> {
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
    let file = fs::File::create(cli.outdir.join("log.txt"))?;
    CombinedLogger::init(vec![
        TermLogger::new(term_level, config.clone(), TerminalMode::Stderr, ColorChoice::Auto),
        WriteLogger::new(LevelFilter::Debug.max(term_level), config, file),
    ])?;
    Ok(())
}

fn rayon_threads(n: usize) -> Result<()> {
    rayon::ThreadPoolBuilder::new()
        .num_threads(n)
        .build_global()
        .context("configuring threads")
}
