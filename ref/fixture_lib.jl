# Shared helpers: compute all reference fields for one case and write fixtures/<name>.json.
# Included by dump_fixtures.jl and simulate.jl.

using TreeKnit, TreeTools, JSON3, Random
using TreeKnit.SplitGraph
using TreeKnit: SRG

const FIXDIR = normpath(joinpath(@__DIR__, "..", "fixtures"))

## Canonical forms
canon_set(x) = sort!(String[string(l) for l in x])
mcc_sort(ms) = sort!([canon_set(m) for m in ms]; by = m -> (length(m), m[1]))
split_sort(ss) = sort!([canon_set(s) for s in ss])
clade(n::TreeNode) = canon_set(l.label for l in POTleaves(n))
# all internal-node clades except the root
tree_splits(t::Tree) = split_sort([clade(n) for n in internals(t) if !isroot(n)])
splitlist_leaves(S::SplitList) = split_sort([leaves(S, i) for i in 1:length(S)])
leaf_order(t::Tree) = [n.label for n in POTleaves(t)]
jnum(x) = (ismissing(x) || !isfinite(x)) ? nothing : x

read_nwk(nwk::AbstractString, k) = parse_newick_string(nwk; label = "t$k")

# Run `f`; on success store result in d[key], on failure store d["<key>_error"].
function try_field!(f, d, key)
    try
        d[key] = f()
    catch e
        msg = sprint(showerror, e)
        @warn "case $(get(d, "name", "?")): $key failed" msg
        d[key*"_error"] = first(msg, 2000)
    end
end

shares_leaves(trees) = all(t -> Set(keys(t.lleaves)) == Set(keys(trees[1].lleaves)), trees)

function resolve_record(trees_in, f)
    ts = map(copy, trees_in)
    ns = f(ts)
    Dict("new_splits" => [splitlist_leaves(S) for S in ns], "splits_after" => [tree_splits(t) for t in ts])
end

function energy_records(t1, t2, seq_lengths; nrand = 20, seed = 1)
    treelist = [copy(t1), copy(t2)]
    mcc = naive_mccs(treelist...)
    mcc_names = TreeKnit.name_mcc_clades!(treelist, mcc)
    for t in treelist
        TreeKnit.reduce_to_mcc!(t, mcc)
    end
    g = SplitGraph.trees2graph(treelist)
    n = length(g.labels)
    confs = Vector{Vector{Bool}}([ones(Bool, n)])
    for i in 1:n                      # remove single MCCs
        c = ones(Bool, n); c[i] = false; push!(confs, c)
    end
    rng = MersenneTwister(seed)
    for k in 1:nrand                  # random configurations, removal probability varies
        p = (0.1, 0.25, 0.5)[mod1(k, 3)]
        push!(confs, rand(rng, n) .> p)
    end
    unique!(confs)
    map(confs) do conf
        SplitGraph.set_resolve(true)
        Er = SplitGraph.compute_energy(conf, g)
        lk = SplitGraph.conf_likelihood(conf, g, seq_lengths, treelist)
        SplitGraph.set_resolve(false)
        En = SplitGraph.compute_energy(conf, g)
        SplitGraph.set_resolve(true)
        Dict(
            "removed" => mcc_sort([mcc_names[x] for x in g.labels[.!conf]]),
            "E_resolve" => Er, "E_noresolve" => En, "lk" => jnum(lk),
        )
    end
end

function fitch_records(t1, mccs)
    m = map_mccs(t1, mccs)
    recs = [Dict("clade" => clade(n), "mcc" => (isnothing(m[n.label]) ? nothing : m[n.label] - 1))
            for n in values(t1.lnodes)]
    sort!(recs; by = r -> (length(r["clade"]), r["clade"]))
end

function arg_record(t1, t2, mccs)
    arg = SRG.arg_from_trees(t1, t2, mccs)[1]
    s1, s2 = SRG.trees_from_arg(arg)
    nosing(t) = (t = copy(t); TreeTools.remove_internal_singletons!(t); tree_splits(t))
    Dict(
        "n_hybrids" => length(arg.hybrids),
        # raw segment trees: hybrid nodes appear as internal singletons (duplicate clades)
        "segment_trees" => [tree_splits(s1), tree_splits(s2)],
        "segment_trees_no_singletons" => [nosing(s1), nosing(s2)],
        "segment_leaf_sets" => [canon_set(keys(s1.lleaves)), canon_set(keys(s2.lleaves))],
    )
end

run_mccs(trees, seed) = (Random.seed!(seed); M = run_treeknit!(map(copy, trees), OptArgs()); mcc_sort(get(M, 1, 2)))

function run_multi(trees, seed)
    Random.seed!(seed)
    K = length(trees)
    M = run_treeknit!(map(copy, trees), OptArgs(K))
    Dict("$(i-1)-$(j-1)" => mcc_sort(get(M, i, j)) for i in 1:K for j in i+1:K)
end

"""
    make_fixture(name, nwks; mccs=nothing, mccs_source=..., extra=Dict(), nruns=20, nmulti=0)

`nwks`: newick strings. `mccs`: fixed MCC set for K=2 (computed from seeded run if `nothing`).
"""
function make_fixture(name, nwks; mccs = nothing, mccs_source = "given", extra = Dict(),
                      nruns = 20, nmulti = 0, energy_seed = 1)
    trees = [read_nwk(s, k) for (k, s) in enumerate(nwks)]
    K = length(trees)
    seq_lengths = ones(Int, K)
    d = Dict{String,Any}("name" => name, "trees" => collect(String, nwks), "seq_lengths" => seq_lengths)
    merge!(d, extra)
    shared = shares_leaves(trees)
    d["n_leaves"] = [length(t.lleaves) for t in trees]
    shared && try_field!(() -> mcc_sort(naive_mccs(trees...)), d, "naive_mccs")
    if K > 2 && shared   # legacy naive_mccs(t1,t2,t3...) errors whenever a common clade exists; record pairs too
        try_field!(() -> Dict("$(i-1)-$(j-1)" => mcc_sort(naive_mccs(trees[i], trees[j])) for i in 1:K for j in i+1:K),
                   d, "pairwise_naive_mccs")
    end
    try_field!(() -> resolve_record(trees, ts -> resolve!(ts...)), d, "pre_resolve")
    if K == 2 && shared
        t1, t2 = trees
        if isnothing(mccs)
            mccs_source = "run_treeknit!(copies, OptArgs()) with Random.seed!(1)"
            try_field!(() -> run_mccs(trees, 1), d, "mccs")
            mccs = get(d, "mccs", nothing)
        else
            d["mccs"] = mccs = mcc_sort(mccs)
        end
        d["mccs_source"] = mccs_source
        if !isnothing(mccs)
            try_field!(() -> resolve_record(trees, ts -> TreeKnit.resolve!(ts[1], ts[2], mccs; strict = true)), d, "mcc_resolve_strict")
            try_field!(() -> resolve_record(trees, ts -> TreeKnit.resolve!(ts[1], ts[2], mccs; strict = false)), d, "mcc_resolve_liberal")
            try_field!(() -> fitch_records(t1, mccs), d, "fitch")
            try_field!(d, "sorted_leaf_order") do
                c1, c2 = copy(t1), copy(t2)
                TreeTools.ladderize!(c1)
                TreeKnit.sort_polytomies!(c1, c2, mccs; strict = false)
                [leaf_order(c1), leaf_order(c2)]
            end
            try_field!(() -> arg_record(t1, t2, mccs), d, "arg")
        end
        if haskey(d, "naive_mccs") && length(d["naive_mccs"]) > 1
            try_field!(() -> energy_records(t1, t2, seq_lengths; seed = energy_seed), d, "energy")
        end
        nruns > 0 && try_field!(() -> [run_mccs(trees, i) for i in 1:nruns], d, "runs")
    end
    if K > 2 && shared && nmulti > 0
        try_field!(() -> [run_multi(trees, i) for i in 1:nmulti], d, "multi_runs")
    end
    mkpath(FIXDIR)
    open(joinpath(FIXDIR, name * ".json"), "w") do io
        JSON3.pretty(io, JSON3.write(d))
    end
    errs = [k for k in keys(d) if endswith(k, "_error")]
    println(rpad(name, 32), " K=$K leaves=$(d["n_leaves"]) naive=",
            haskey(d, "naive_mccs") ? length(d["naive_mccs"]) : "-",
            isempty(errs) ? "" : "  ERRORS: $(errs)")
    return d
end
