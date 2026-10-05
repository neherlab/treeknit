# Restrict several trees with the same leaves to one random subset of the leaves.
# Usage: subset.jl OUTDIR N SEED TREE...
# Prunes the same leaves from every tree, removes the unary nodes left behind (TreeTools sums their
# branch lengths), and writes each tree to OUTDIR under its file name.
using TreeTools, Random

out, n, seed = ARGS[1], parse(Int, ARGS[2]), parse(Int, ARGS[3])
files = ARGS[4:end]
trees = [read_tree(f) for f in files]
leaves = sort(collect(keys(trees[1].lleaves)))
all(t -> Set(keys(t.lleaves)) == Set(leaves), trees) || error("the trees must have the same leaves")
keep = Set(shuffle(MersenneTwister(seed), leaves)[1:n])
mkpath(out)
for (f, t) in zip(files, trees)
    for l in [l for l in keys(t.lleaves) if !(l in keep)]
        TreeTools.prunesubtree!(t, l; remove_singletons = true)
    end
    TreeTools.remove_internal_singletons!(t)
    Set(keys(t.lleaves)) == keep || error("$f: pruning left other leaves than the subset")
    write_newick(joinpath(out, basename(f)), t)
end
println("$(length(files)) trees with $(length(keep)) leaves in $out")
