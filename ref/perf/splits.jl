# Compare the splits of the resolved trees written by two runs.
# Usage: splits.jl DIR_A DIR_B
# For each *_resolved.nwk in DIR_A, prints the number of non-root splits in both files and the
# splits found in only one of them.
using TreeTools

function splits(t)
    out = Set{Set{String}}()
    function walk(n)
        n.isleaf && return Set([n.label])
        s = union((walk(c) for c in n.child)...)
        n.isroot || push!(out, s)
        s
    end
    walk(t.root)
    out
end

a, b = ARGS
for f in sort(filter(x -> endswith(x, "_resolved.nwk"), readdir(a)))
    sa, sb = splits(read_tree(joinpath(a, f))), splits(read_tree(joinpath(b, f)))
    println(rpad(f, 22), " splits A=", length(sa), " B=", length(sb), " only A=", length(setdiff(sa, sb)), " only B=", length(setdiff(sb, sa)))
end
