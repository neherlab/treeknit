# Run the TreeKnit.jl command line once per seed in one process and print the time of each run.
# Usage: seeds.jl OUTDIR FIRST LAST TREE... [-- OPTION...]
# Each run writes the files of `treeknit --outdir` to OUTDIR/s<seed>/. TreeKnit.jl has no seed option,
# so each run starts with `Random.seed!(seed)`. The times exclude Julia start-up and, with a system
# image, compilation.
using TreeKnit, Random

out, seed_first, seed_last = ARGS[1], parse(Int, ARGS[2]), parse(Int, ARGS[3])
rest = ARGS[4:end]
k = findfirst(==("--"), rest)
trees, options = isnothing(k) ? (rest, String[]) : (rest[1:k-1], rest[k+1:end])
for seed in seed_first:seed_last
    Random.seed!(seed)
    t = @elapsed TreeKnit.command_main([trees..., "--outdir=$(joinpath(out, "s$seed"))", "--verbosity-level=-1", options...])
    println("seed=$seed time=$(round(t, digits = 3))")
    flush(stdout)
end
