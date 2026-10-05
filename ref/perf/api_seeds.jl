# Run TreeKnit.jl `run_treeknit!` once per seed, with options that its command line ignores, and
# print the time of each run.
# Usage: api_seeds.jl OUTDIR FIRST LAST TREE... [-- OPTION...]
# Options: --better-MCCs (default: --better-trees), --no-pre-resolve, --no-likelihood.
# Each run writes OUTDIR/s<seed>/MCCs.json in the format of the command line; tree labels are the
# file names without extension.
using TreeKnit, TreeTools, Random

out, seed_first, seed_last = ARGS[1], parse(Int, ARGS[2]), parse(Int, ARGS[3])
rest = ARGS[4:end]
k = findfirst(==("--"), rest)
files, options = isnothing(k) ? (rest, String[]) : (rest[1:k-1], rest[k+1:end])
known = ["--better-MCCs", "--no-pre-resolve", "--no-likelihood"]
unknown = setdiff(options, known)
isempty(unknown) || error("unknown options $(unknown); known: $(known)")
for seed in seed_first:seed_last
    Random.seed!(seed)
    trees = [read_tree(f; label = splitext(basename(f))[1]) for f in files]
    oa = OptArgs(length(trees); method = "--better-MCCs" in options ? :better_MCCs : :better_trees)
    "--no-pre-resolve" in options && (oa.pre_resolve = false)
    "--no-likelihood" in options && (oa.likelihood_sort = false)
    t = @elapsed M = run_treeknit!(trees, oa)
    mkpath(joinpath(out, "s$seed"))
    TreeKnit.write_mccs(joinpath(out, "s$seed", "MCCs.json"), M)
    println("seed=$seed time=$(round(t, digits = 3))")
    flush(stdout)
end
