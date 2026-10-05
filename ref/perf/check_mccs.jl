# Check MCCs against the input trees, without the code of either implementation.
# Usage: check_mccs.jl TREEDIR ROOT...
# TREEDIR holds the input trees as <label>.nwk; each ROOT holds one directory per run with
# MCCs.json. An MCC is invalid when, restricted to its leaves, a split of one tree is incompatible
# with a split of the other tree of its pair, so that the two topologies differ beyond resolution.
# Prints, per root and tree pair, the runs with an invalid MCC and the number of invalid MCCs.
using JSON3, TreeTools

function clades(t)
    out = Set{String}[]
    function walk(n)
        n.isleaf && return Set([n.label])
        s = union((walk(c) for c in n.child)...)
        push!(out, s)
        s
    end
    walk(t.root)
    out
end

compatible(x, y) = isempty(intersect(x, y)) || issubset(x, y) || issubset(y, x)

function restricted(cl, m)
    r = Set{Set{String}}()
    for c in cl
        s = intersect(c, m)
        1 < length(s) < length(m) && push!(r, s)
    end
    r
end

invalid_count(mccs, ca, cb) = count(mccs) do m
    m = Set(String.(m))
    ra, rb = restricted(ca, m), restricted(cb, m)
    any(!compatible(x, y) for x in ra for y in rb)
end

treedir = ARGS[1]
cl = Dict(splitext(f)[1] => clades(read_tree(joinpath(treedir, f))) for f in readdir(treedir) if endswith(f, ".nwk"))
for root in ARGS[2:end]
    per_pair = Dict{String,Vector{Int}}()
    for d in sort(readdir(root))
        f = joinpath(root, d, "MCCs.json")
        isfile(f) || continue
        for v in values(JSON3.read(read(f, String)).MCC_dict)
            a, b = String.(v.trees)
            push!(get!(per_pair, join(sort([a, b]), "-"), Int[]), invalid_count(v.mccs, cl[a], cl[b]))
        end
    end
    for k in sort(collect(keys(per_pair)))
        v = per_pair[k]
        println(rpad(root, 40), " ", rpad(k, 9), " runs=", length(v), " runs_with_invalid=", count(>(0), v), " invalid_mccs=", sum(v))
    end
end
