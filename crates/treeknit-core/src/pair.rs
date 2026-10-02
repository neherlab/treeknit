//! MCC inference for one pair of trees: repeated naive MCCs, annealing, and pruning.

use crate::anneal;
use crate::bits::Bits;
use crate::naive::{naive_mccs, sort_mccs, Mcc};
use crate::resolve::resolve_trees;
use crate::splitgraph::Graph;
use crate::tree::{NodeId, Tree};
use rand::seq::SliceRandom;
use rand::Rng;

/// Parameters of pair inference (a subset of [`crate::Options`]).
#[derive(Clone, Debug)]
pub struct PairParams {
    pub gamma: f64,
    pub itmax: usize,
    pub likelihood_sort: bool,
    pub resolve: bool,
    pub seq_lengths: [f64; 2],
    pub n_mcmc: usize,
    pub sa_rep: usize,
    pub temperatures: Vec<f64>,
}

/// Infer MCCs of `t1` and `t2`, which must have the same leaf set. Taxa are in `0..n_taxa`.
pub fn infer_pair(t1: &Tree, t2: &Tree, n_taxa: usize, p: &PairParams, rng: &mut impl Rng) -> Vec<Mcc> {
    let mut trees = [t1.clone(), t2.clone()];
    if p.resolve {
        resolve_trees(&mut trees, n_taxa);
    }
    log::debug!(
        "initial state: {} naive MCCs",
        naive_mccs(&[&trees[0], &trees[1]], n_taxa).len()
    );
    let mut found: Vec<Mcc> = Vec::new();
    for it in 1.. {
        let n_leaves = trees[0].n_leaves();
        let m = (n_leaves as f64 * p.n_mcmc as f64 / p.temperatures.len() as f64).ceil() as usize;
        log::debug!(
            "iteration {it} (max. {}): {n_leaves} leaves, {m} steps per temperature",
            p.itmax
        );
        let new = remove_mccs(&trees[0], &trees[1], n_taxa, p, m, rng);
        log::debug!("found {} new MCCs", new.len());
        found.extend(new.iter().cloned());

        if new.is_empty() {
            found.extend(naive_mccs(&[&trees[0], &trees[1]], n_taxa));
            break;
        }
        if new.iter().map(|m| m.len()).sum::<usize>() == n_leaves {
            break;
        }
        for t in trees.iter_mut() {
            prune_mccs(t, &new, n_taxa);
        }
        if p.resolve {
            resolve_trees(&mut trees, n_taxa);
        }
        let remaining = naive_mccs(&[&trees[0], &trees[1]], n_taxa);
        if remaining.len() == 1 || it > p.itmax {
            found.extend(remaining);
            break;
        }
    }
    sort_mccs(found)
}

/// One annealing step: the naive MCCs that should be removed from the trees.
fn remove_mccs(t1: &Tree, t2: &Tree, n_taxa: usize, p: &PairParams, m: usize, rng: &mut impl Rng) -> Vec<Mcc> {
    let mccs = naive_mccs(&[t1, t2], n_taxa);
    if mccs.len() == 1 {
        return mccs;
    }
    let r1 = reduce_to_mccs(t1, &mccs, n_taxa);
    let r2 = reduce_to_mccs(t2, &mccs, n_taxa);
    let g = Graph::new(&[&r1, &r2], mccs.len());
    let confs = anneal::optimize(&g, p.gamma, &p.temperatures, m, p.sa_rep, p.resolve, rng);
    let conf = if confs.len() == 1 {
        confs.into_iter().next().unwrap()
    } else {
        choose_conf(confs, &g, &[&r1, &r2], p, rng)
    };
    (0..mccs.len())
        .filter(|&i| !conf.contains(i))
        .map(|i| mccs[i].clone())
        .collect()
}

/// Pick one of several optimal configurations: drop the trivial one, then maximise the
/// branch-length likelihood, then minimise energy, then choose at random.
fn choose_conf(confs: Vec<Bits>, g: &Graph, trees: &[&Tree], p: &PairParams, rng: &mut impl Rng) -> Bits {
    let mut confs: Vec<Bits> = confs.into_iter().filter(|c| c.count_ones(..) < g.n).collect();
    if confs.len() > 1 && p.likelihood_sort {
        let lk: Vec<f64> = confs
            .iter()
            .map(|c| g.likelihood(c, p.resolve, trees, &p.seq_lengths))
            .collect();
        log::trace!("likelihoods of {} configurations: {lk:?}", confs.len());
        let lmax = lk.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        confs = confs
            .into_iter()
            .zip(&lk)
            .filter(|(_, &l)| l == lmax)
            .map(|(c, _)| c)
            .collect();
        if confs.len() > 1 {
            let e: Vec<usize> = confs.iter().map(|c| g.energy(c, p.resolve)).collect();
            let emin = *e.iter().min().unwrap();
            confs = confs
                .into_iter()
                .zip(&e)
                .filter(|(_, &x)| x == emin)
                .map(|(c, _)| c)
                .collect();
        }
    }
    confs.choose(rng).unwrap().clone()
}

/// Copy of `tree` in which each MCC (a clade) becomes a single leaf with taxon = MCC index.
pub fn reduce_to_mccs(tree: &Tree, mccs: &[Mcc], n_taxa: usize) -> Tree {
    let leaf_of = tree.leaf_of(n_taxa);
    let mut mcc_of: Vec<Option<usize>> = vec![None; tree.nodes.len()];
    for (i, m) in mccs.iter().enumerate() {
        let r = tree.lca_of(m.iter().map(|&x| leaf_of[x].unwrap())).unwrap();
        mcc_of[r] = Some(i);
    }
    let mut out = tree.clone();
    for n in tree.preorder() {
        if let Some(i) = mcc_of[n] {
            out.nodes[n].children.clear();
            out.nodes[n].taxon = Some(i);
            if !tree.is_leaf(n) {
                out.nodes[n].name = format!("MCC_{}", i + 1);
            }
        }
    }
    out.compacted()
}

/// Remove the clades `mccs` from `t`.
pub fn prune_mccs(t: &mut Tree, mccs: &[Mcc], n_taxa: usize) {
    for m in mccs {
        let leaf_of = t.leaf_of(n_taxa);
        let r: NodeId = t.lca_of(m.iter().map(|&x| leaf_of[x].unwrap())).unwrap();
        t.prune(r);
    }
    *t = t.compacted();
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::options::Options;
    use crate::tree::test_util::trees;
    use rand::SeedableRng;
    use rand_xoshiro::Xoshiro256PlusPlus;

    fn params(o: &Options) -> PairParams {
        PairParams {
            gamma: o.gamma,
            itmax: o.itmax,
            likelihood_sort: o.likelihood_sort,
            resolve: o.resolve,
            seq_lengths: [1.0, 1.0],
            n_mcmc: o.n_mcmc,
            sa_rep: o.sa_rep,
            temperatures: o.temperatures(),
        }
    }

    fn run(nwk: [&str; 2], o: &Options, seed: u64) -> Vec<Vec<String>> {
        let (ts, taxa) = trees(&nwk);
        let mut rng = Xoshiro256PlusPlus::seed_from_u64(seed);
        let m = infer_pair(&ts[0], &ts[1], taxa.len(), &params(o), &mut rng);
        m.iter().map(|x| taxa.names_of(x)).collect()
    }

    #[test]
    fn single_reassorted_leaf() {
        let o = Options::default();
        for seed in 0..5 {
            let m = run(["((A,B),(C,(D,X)));", "((A,(B,X)),(C,D));"], &o, seed);
            assert_eq!(m, vec![vec!["X"], vec!["A", "B", "C", "D"]]);
        }
    }

    #[test]
    fn gamma_threshold() {
        // Removing two leaves fixes 5 incompatibilities: accepted for γ = 2, not for γ = 3.
        let nwk = ["((((A,B),C),D),E);", "((((D,B),E),A),C);"];
        let o2 = Options {
            gamma: 2.0,
            ..Options::default()
        };
        let m = run(nwk, &o2, 1);
        assert_eq!(m.len(), 3);
        assert_eq!(m[2].len(), 3);
        let o3 = Options {
            gamma: 3.0,
            ..Options::default()
        };
        assert_eq!(run(nwk, &o3, 1).len(), 5);
    }

    #[test]
    fn likelihood_breaks_degeneracy() {
        let o = Options::default();
        for seed in 0..10 {
            let m = run(["((A:2,B:2):2,C:4);", "(A:2,(B:1,C:1):1);"], &o, seed);
            assert_eq!(m, vec![vec!["C"], vec!["A", "B"]], "seed {seed}");
        }
        let nwk = [
            "(((A1,A2):3.,(B1,B2):3.):5.,(C1,C2):5.);",
            "((A1,A2):3.,((B1,B2):2.,(C1,C2):2.):1.);",
        ];
        let o = Options {
            resolve: false,
            ..Options::default()
        };
        for seed in 0..10 {
            let m = run(nwk, &o, seed);
            assert_eq!(m[0], vec!["C1", "C2"], "seed {seed}");
        }
    }

    #[test]
    fn resolution_during_inference() {
        let nwk = ["((A,B),(C,(D,(E,X))));", "((A,(B,X)),(C,D,E));"];
        let m = run(nwk, &Options::default(), 3);
        assert_eq!(m, vec![vec!["X"], vec!["A", "B", "C", "D", "E"]]);
    }
}
