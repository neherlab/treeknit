//! The former method options of TreeKnit.jl (`--better-trees`, `--better-MCCs`, ...), which
//! reproduce its results.

use crate::{Cli, parse_lengths};
use treeknit_core::{Options, Resolution};
use treeknit_io::analysis::{self, Settings, ValidationError};

/// Options of the former options for `k` trees, or every error of the flags and of the shared
/// settings checks. Independent pairs run in parallel.
pub(crate) fn former_options_checked(cli: &Cli, k: usize) -> Result<Options, Vec<ValidationError>> {
  let mut errors = Vec::new();
  let seq_lengths = match cli.seq_lengths.as_deref().map(parse_lengths).transpose() {
    Ok(v) => v,
    Err(e) => {
      errors.push(e);
      None
    },
  };
  if cli.resolve.is_some() || cli.pre_resolve || cli.no_final_round || cli.final_round {
    errors.push(ValidationError::new(
      "former method options (--better-trees, --better-MCCs, --no-resolve, --liberal-resolve, \
       --resolve-all-rounds, --match-topologies) cannot be combined with --resolve, --pre-resolve, \
       --no-final-round or --final-round; see --help-resolve",
    ));
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

pub(crate) fn uses_former_options(cli: &Cli) -> bool {
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
