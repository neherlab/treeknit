//! `treeknit`: infer maximally compatible clades (MCCs) and reassortment graphs from
//! segment trees.

use anyhow::{bail, Context, Result};
use clap::Parser;
use simplelog::{ColorChoice, CombinedLogger, ConfigBuilder, LevelFilter, TermLogger, TerminalMode, WriteLogger};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Instant;
use treeknit_core::{Method, Options, Taxa, Tree};
use treeknit_io::{arg, auspice, mccs, newick};

const DEFAULTS_HELP: &str = "\
--better-trees (default for more than two trees):
  * resolve all trees compatibly before inferring MCCs;
  * run one round of TreeKnit on all tree pairs independently, without resolving.
  Produces better resolved trees; MCCs are homogeneous but more numerous than needed.
  Equivalent to `treeknit t1 t2 --no-resolve`.

--better-MCCs (default for two trees):
  * resolve all trees compatibly before inferring MCCs;
  * run one round of TreeKnit on all pairs sequentially, resolving trees on the way
    (the order of trees matters);
  * for more than two trees, run a second round without resolving.
  Produces the most accurate MCCs; output trees may contain more wrong splits.
  Equivalent to `treeknit t1 t2 --resolve-all-rounds` (two trees) or
  `treeknit t1 t2 t3 --rounds 2` (more trees).";

/// Infer reassortment between segment trees: maximally compatible clades (MCCs) for every
/// pair of trees, resolved trees, and for two trees an ancestral reassortment graph (ARG).
#[derive(Parser, Debug)]
#[command(
    name = "treeknit",
    version,
    after_help = "Use --help-defaults for details on --better-trees and --better-MCCs."
)]
struct Cli {
    /// Newick files, one tree per segment (at least two).
    #[arg(required_unless_present = "help_defaults")]
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

    /// Rounds of pair inference (default given by the method).
    #[arg(long)]
    rounds: Option<usize>,

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

    /// Explain --better-trees and --better-MCCs.
    #[arg(long)]
    help_defaults: bool,

    /// Use the --better-trees method.
    #[arg(long, conflicts_with = "better_mccs")]
    better_trees: bool,

    /// Use the --better-MCCs method.
    #[arg(long = "better-MCCs")]
    better_mccs: bool,

    /// Naive MCCs (γ → ∞).
    #[arg(long)]
    naive: bool,

    /// Do not resolve all trees with each other before inference.
    #[arg(long)]
    no_pre_resolve: bool,

    /// Do not resolve trees during pair inference.
    #[arg(long)]
    no_resolve: bool,

    /// Resolve ambiguous splits too, choosing the most parsimonious placement.
    #[arg(long)]
    liberal_resolve: bool,

    /// Resolve in all rounds, including the last.
    #[arg(long)]
    resolve_all_rounds: bool,

    /// Do not break ties between configurations with branch lengths.
    #[arg(long)]
    no_likelihood: bool,

    /// After inference, resolve all trees so that their topologies match within every MCC
    /// (liberally, also where strict resolution would not). Where splits from different trees
    /// conflict, trees given earlier take precedence, and MCCs that cannot be matched are split.
    /// Implies resolving trees in all rounds, including the last.
    #[arg(long)]
    match_topologies: bool,

    /// Write trees with leaves missing from them placed by imputation (`*_imputed.nwk`).
    #[arg(long)]
    impute: bool,

    /// Write auspice JSON files for tanglegram visualisation.
    #[arg(long)]
    auspice_view: bool,

    /// Accepted for compatibility; independent pairs always run in parallel (see --threads).
    #[arg(long, hide = true)]
    parallel: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    if cli.help_defaults {
        println!("{DEFAULTS_HELP}");
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
    let method = if cli.better_trees {
        Some(Method::BetterTrees)
    } else if cli.better_mccs {
        Some(Method::BetterMccs)
    } else {
        None
    };
    let mut o = Options::for_trees(k, method);
    log::info!(
        "method: {}",
        match method.unwrap_or(if k > 2 { Method::BetterTrees } else { Method::BetterMccs }) {
            Method::BetterTrees => "--better-trees",
            Method::BetterMccs => "--better-MCCs",
        }
    );
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
    if cli.no_pre_resolve {
        o.pre_resolve = false;
    }
    if cli.no_resolve {
        o.resolve = false;
    }
    if cli.liberal_resolve {
        o.strict = false;
    }
    if cli.resolve_all_rounds {
        o.final_no_resolve = false;
    }
    if let Some(r) = cli.rounds {
        if r == 0 {
            bail!("--rounds must be at least 1");
        }
        o.rounds = r;
    }
    if cli.match_topologies {
        // MCCs inferred without resolution already have identical topologies; matching is
        // meaningful for MCCs inferred up to resolution, and replaces a final unresolved round.
        o.match_topologies = true;
        o.final_no_resolve = false;
        if cli.no_resolve {
            log::warn!("--match-topologies with --no-resolve: MCCs already match, nothing to do");
        } else {
            o.resolve = true;
        }
    }
    if k > 2 && o.resolve && !o.final_no_resolve && !o.match_topologies {
        log::warn!("for more than two trees, resolving in the final round is not recommended (--resolve-all-rounds)");
    }
    log::info!(
        "γ = {}, {} round(s), pre-resolve: {}, resolve: {} ({})",
        o.gamma,
        o.rounds,
        o.pre_resolve,
        o.resolve,
        if o.strict { "strict" } else { "liberal" }
    );
    Ok(o)
}

fn params_json(o: &Options, seed: u64) -> serde_json::Value {
    serde_json::json!({
        "gamma": o.gamma,
        "itmax": o.itmax,
        "likelihood_sort": o.likelihood_sort,
        "resolve": o.resolve,
        "strict": o.strict,
        "seq_lengths": o.seq_lengths,
        "pre_resolve": o.pre_resolve,
        "rounds": o.rounds,
        "final_no_resolve": o.final_no_resolve,
        "nMCMC": o.n_mcmc,
        "sa_rep": o.sa_rep,
        "Tmin": o.t_min,
        "Tmax": o.t_max,
        "nT": o.n_t,
        "cooling_schedule": format!("{:?}", o.cooling).to_lowercase(),
        "naive": o.naive,
        "match_topologies": o.match_topologies,
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
