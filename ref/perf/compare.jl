# Compare the MCCs of two sets of runs, for example TreeKnit.jl and the port with different seeds.
# Usage: compare.jl ROOT_A ROOT_B [NPERM] [--freq]
# Each root holds one directory per run with MCCs.json. For each tree pair it prints:
# - the mean and standard deviation of the MCC count, and the permutation p-value of the difference
#   of the means
# - the scaled variation of information (VI / ln n) between every two runs, within A, within B, and
#   across, and the energy-distance statistic on these distances with its permutation p-value
#   (null hypothesis: both sets of runs come from the same distribution of partitions)
# - the number of distinct partitions, and how many runs of each set the other set also produced
# - the number of runs whose MCCs do not partition the leaves
# With --freq it also prints how often each distinct partition occurs in each set.
using JSON3, Statistics, Printf, Random

function load(dir)
    j = JSON3.read(read(joinpath(dir, "MCCs.json"), String))
    Dict(join(sort(String.(v.trees)), "-") => sort!([sort!(String.(m)) for m in v.mccs]) for v in values(j.MCC_dict))
end

runs(root) = [load(joinpath(root, d)) for d in sort(readdir(root)) if isfile(joinpath(root, d, "MCCs.json"))]

function labels(p)
    d = Dict{String,Int}()
    for (i, m) in enumerate(p), x in m
        d[x] = i
    end
    d
end

# Variation of information divided by ln n (Meila 2007).
function scaled_vi(a, b)
    la, lb = labels(a), labels(b)
    n = length(la)
    cont = Dict{Tuple{Int,Int},Int}()
    for (x, i) in la
        k = (i, lb[x])
        cont[k] = get(cont, k, 0) + 1
    end
    h(c) = -sum(v / n * log(v / n) for v in c)
    (2h(values(cont)) - h(length.(a)) - h(length.(b))) / log(n)
end

is_partition(p, leaves) = sum(length, p) == length(leaves) && Set(Iterators.flatten(p)) == leaves

mean_offdiag(D, I, J) = mean(D[i, j] for i in I for j in J if i != j)

# Energy distance (Szekely and Rizzo 2013) of the groups `g` and `.!g` from the distance matrix `D`.
energy_stat(D, g) = (a = findall(g); b = findall(.!g); 2mean_offdiag(D, a, b) - mean_offdiag(D, a, a) - mean_offdiag(D, b, b))

function compare(A, B; nperm, freq, rng = MersenneTwister(1))
    for key in sort(collect(keys(A[1])))
        PA = [r[key] for r in A]
        PB = [r[key] for r in B]
        leaves = Set(Iterators.flatten(PA[1]))
        invalid = count(p -> !is_partition(p, leaves), [PA; PB])
        ca, cb = length.(PA), length.(PB)
        na = length(ca)
        counts = [ca; cb]
        obs = mean(ca) - mean(cb)
        p_count = (1 + count(_ -> (s = shuffle(rng, counts); abs(mean(s[1:na]) - mean(s[na+1:end])) >= abs(obs) - 1e-12), 1:nperm)) / (nperm + 1)
        P = [PA; PB]
        N = length(P)
        D = [i < j ? scaled_vi(P[i], P[j]) : 0.0 for i in 1:N, j in 1:N]
        D = D + D'
        g = [trues(na); falses(length(PB))]
        e = energy_stat(D, g)
        p_energy = (1 + count(_ -> energy_stat(D, shuffle(rng, g)) >= e - 1e-12, 1:nperm)) / (nperm + 1)
        ia, ib = 1:na, na+1:N
        sa, sb = Set(PA), Set(PB)
        @printf("%-9s n=%-5d runs A/B=%d/%d  #MCC A %.2f±%.2f B %.2f±%.2f p=%.3f | VI AA %.4f BB %.4f AB %.4f E=%.5f p=%.3f | distinct A %d B %d, B in A %d/%d, A in B %d/%d | invalid %d\n",
            key, length(leaves), na, length(PB), mean(ca), std(ca), mean(cb), std(cb), p_count,
            mean_offdiag(D, ia, ia), mean_offdiag(D, ib, ib), mean_offdiag(D, ia, ib), e, p_energy,
            length(sa), length(sb), count(in(sa), PB), length(PB), count(in(sb), PA), na, invalid)
        if freq
            for p in sort(collect(union(sa, sb)); by = length)
                @printf("    #MCC %-5d A %-5d B %d\n", length(p), count(==(p), PA), count(==(p), PB))
            end
        end
    end
end

args = filter(!=("--freq"), ARGS)
compare(runs(args[1]), runs(args[2]); nperm = length(args) >= 3 ? parse(Int, args[3]) : 2000, freq = "--freq" in ARGS)
