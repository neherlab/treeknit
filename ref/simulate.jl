# Simulated reference fixtures: ARGs from ARGTools, true MCCs, optional polytomies.
# Run: julia --project=/tmp/tkref/simenv /workspace/treeknit-rs/ref/simulate.jl [--no-runs]
using ARGTools, Distributions
include(joinpath(@__DIR__, "fixture_lib.jl"))

const N = 10_000
const NRUNS = "--no-runs" in ARGS ? 0 : 20

include(joinpath(@__DIR__, "sim_lib.jl"))

# (name, K, n, ρ, polytomy cutoff c (0 = none), seed)
const SIMS = [
    ("sim_k2_n50_r0.02",     2,  50, 0.02, 0.0, 11),
    ("sim_k2_n50_r0.02_poly",2,  50, 0.02, 0.1, 12),
    ("sim_k2_n50_r0.05",     2,  50, 0.05, 0.0, 13),
    ("sim_k2_n50_r0.05_poly",2,  50, 0.05, 0.1, 14),
    ("sim_k2_n50_r0.1",      2,  50, 0.1,  0.0, 15),
    ("sim_k2_n50_r0.1_poly", 2,  50, 0.1,  0.1, 16),
    ("sim_k2_n100_r0.01",     2, 100, 0.01, 0.0, 21),
    ("sim_k2_n100_r0.01_poly",2, 100, 0.01, 0.1, 22),
    ("sim_k2_n100_r0.03",     2, 100, 0.03, 0.0, 23),
    ("sim_k2_n100_r0.03_poly",2, 100, 0.03, 0.1, 24),
    ("sim_k2_n100_r0.02",     2, 100, 0.02, 0.0, 25),
    ("sim_k2_n100_r0.02_poly",2, 100, 0.02, 0.1, 26),
    ("sim_k3_n50_r0.05",      3,  50, 0.05, 0.0, 31),
    ("sim_k3_n50_r0.05_poly", 3,  50, 0.05, 0.1, 32),
    ("sim_k3_n50_r0.1",      3,  50, 0.1,  0.0, 33),
    ("sim_k3_n50_r0.1_poly", 3,  50, 0.1,  0.1, 34),
]

only = filter(a -> !startswith(a, "--"), ARGS)
for (name, K, n, ρ, c, seed) in SIMS
    (isempty(only) || name in only) || continue
    Random.seed!(seed)
    r = ARGTools.get_r(ρ, n, N, :kingman)
    arg = ARGTools.SimulateARG.simulate(N, r, n; K, simtype = :kingman)
    trees = ARGTools.trees_from_ARG(arg)
    c > 0 && foreach(t -> remove_branches!(t, c, N), trees)
    nwks = [write_newick(t) for t in trees]
    true_mccs = Dict("$(i-1)-$(j-1)" => mcc_sort(ARGTools.MCCs_from_arg(arg, i, j)) for i in 1:K for j in i+1:K)

    dir = joinpath(FIXDIR, "sim", name)
    mkpath(dir)
    for (k, s) in enumerate(nwks)
        write(joinpath(dir, "tree$k.nwk"), s * "\n")
    end
    extra = Dict("true_mccs" => true_mccs,
                 "sim_params" => Dict("N" => N, "n" => n, "rho" => ρ, "K" => K, "simtype" => "kingman",
                                      "polytomy_cutoff_c" => c, "seed" => seed))
    make_fixture(name, nwks; extra, nruns = NRUNS, nmulti = 5,
                 mccs = K == 2 ? true_mccs["0-1"] : nothing, mccs_source = "true (ARGTools.MCCs_from_arg)")
    println("    true MCC counts: ", Dict(k => length(v) for (k, v) in true_mccs))
end
