# Cost of the full topological energy in TreeKnit.jl, on the split graph of the naive MCCs of two trees.
# Usage: energy.jl TREE1 TREE2
# Counterpart of the port's `just example energy_cost TREE1 TREE2`. Prints the energy of the
# configuration that keeps every naive MCC and the time of one computation, without and with
# resolution.
using TreeKnit, TreeTools
const SG = TreeKnit.SplitGraph

trees = [read_tree(f) for f in ARGS[1:2]]
mccs = naive_mccs(trees...)
TreeKnit.name_mcc_clades!(trees, mccs)
foreach(t -> TreeKnit.reduce_to_mcc!(t, mccs), trees)
g = SG.trees2graph(trees)
conf = ones(Bool, length(g.leaves))
println("naive MCCs ", length(mccs))
for resolve in (false, true)
    SG.set_resolve(resolve)
    energy = SG.compute_energy(conf, g)
    reps = 200
    t = @elapsed for _ in 1:reps
        SG.compute_energy(conf, g)
    end
    println("resolve $resolve: full energy $energy, $(round(t / reps * 1e6, digits = 1)) µs per full computation")
end
