# Simulated ARGs for the accuracy comparison: segment trees and true MCCs.
# Usage: simulate.jl OUTDIR
# Writes OUTDIR/<setting>_rep<r>/tree<k>.nwk and truth.json, which maps "tree<i>-tree<j>" to the true
# MCCs of the pair. Same simulator, population size, and polytomy model as ../simulate.jl.
using ARGTools, JSON3, Random
include(joinpath(@__DIR__, "..", "sim_lib.jl"))

const N = 10_000

out = ARGS[1]
settings = [(K, n, ρ, c) for K in (2,) for n in (200, 500) for ρ in (0.05, 0.1, 0.2) for c in (0.0, 0.1)]
append!(settings, [(K, n, ρ, c) for K in (4,) for n in (200,) for ρ in (0.05, 0.1) for c in (0.0, 0.1)])
sims = [(s..., rep) for s in settings for rep in 1:3]
for (i, (K, n, ρ, c, rep)) in enumerate(sims)
    name = "k$(K)_n$(n)_r$(ρ)_c$(c)_rep$(rep)"
    Random.seed!(1000 + i)
    arg = ARGTools.SimulateARG.simulate(N, ARGTools.get_r(ρ, n, N, :kingman), n; K, simtype = :kingman)
    trees = ARGTools.trees_from_ARG(arg)
    c > 0 && foreach(t -> remove_branches!(t, c, N), trees)
    dir = joinpath(out, name)
    mkpath(dir)
    for (k, t) in enumerate(trees)
        write(joinpath(dir, "tree$k.nwk"), write_newick(t) * "\n")
    end
    truth = Dict("tree$a-tree$b" => [sort(String.(m)) for m in ARGTools.MCCs_from_arg(arg, a, b)] for a in 1:K for b in a+1:K)
    open(io -> JSON3.write(io, truth), joinpath(dir, "truth.json"), "w")
    println(name, ": true MCC counts ", Dict(k => length(v) for (k, v) in truth))
end
