# Runtime comparison: simulate pairs of trees with ARGTools, time Julia TreeKnit (after a
# warm-up run, so compilation is excluded) and write the trees for timing the Rust CLI.
# Run: julia --project=/tmp/tkref/simenv ref/bench.jl OUTDIR
using ARGTools, TreeTools, TreeKnit, Random
const N = 10_000
out = ARGS[1]
TreeKnit.run_treeknit!([copy(t) for t in ARGTools.trees_from_ARG(ARGTools.SimulateARG.simulate(N, 1e-5, 20; K=2, simtype=:kingman))], OptArgs())
for (n, rho) in [(100, 0.05), (200, 0.05), (500, 0.05), (1000, 0.05), (2000, 0.05)]
    Random.seed!(n)
    arg = ARGTools.SimulateARG.simulate(N, ARGTools.get_r(rho, n, N, :kingman), n; K=2, simtype=:kingman)
    trees = ARGTools.trees_from_ARG(arg)
    dir = joinpath(out, "n$n"); mkpath(dir)
    for (k, t) in enumerate(trees)
        write(joinpath(dir, "tree$k.nwk"), write_newick(t) * "\n")
    end
    t = @elapsed m = TreeKnit.run_treeknit!([copy(x) for x in trees], OptArgs())
    println("n=$n true=$(length(ARGTools.MCCs_from_arg(arg, 1, 2))) julia_mccs=$(length(first(values(m.mccs)))) julia_time=$(round(t, digits=2))s")
    flush(stdout)
end
