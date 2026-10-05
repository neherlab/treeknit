# Accuracy against the true MCCs of simulated ARGs (see simulate.jl).
# Usage: accuracy.jl SIMDIR RUNDIR ARM...
# RUNDIR/<simulation>/<arm>/ holds one directory per run with MCCs.json. For each simulation, tree
# pair, and arm it averages over the runs the scaled VI (VI / ln n) between inferred and true MCCs and
# the difference of the MCC counts. It prints these per setting, then compares each arm with the
# first over all (simulation, tree pair) units: mean difference, 95% confidence interval, and
# sign-flip permutation p-value, for the VI and for the absolute count error.
using JSON3, Statistics, Printf, Random

load(dir) = Dict(join(sort(String.(v.trees)), "-") => [String.(m) for m in v.mccs] for v in values(JSON3.read(read(joinpath(dir, "MCCs.json"), String)).MCC_dict))

# Variation of information divided by ln n (Meila 2007).
function scaled_vi(a, b)
    la = Dict(x => i for (i, m) in enumerate(a) for x in m)
    lb = Dict(x => i for (i, m) in enumerate(b) for x in m)
    n = length(la)
    cont = Dict{Tuple{Int,Int},Int}()
    for (x, i) in la
        k = (i, lb[x])
        cont[k] = get(cont, k, 0) + 1
    end
    h(c) = -sum(v / n * log(v / n) for v in c)
    (2h(values(cont)) - h(length.(a)) - h(length.(b))) / log(n)
end

simdir, rundir = ARGS[1], ARGS[2]
arms = ARGS[3:end]
units = []
for name in sort(readdir(simdir))
    truth = JSON3.read(read(joinpath(simdir, name, "truth.json"), String))
    for (pair, true_mccs) in pairs(truth)
        true_mccs = [String.(m) for m in true_mccs]
        r = Dict{String,Tuple{Float64,Float64}}()
        for arm in arms
            root = joinpath(rundir, name, arm)
            isdir(root) || continue
            runs = [load(joinpath(root, d)) for d in readdir(root) if isfile(joinpath(root, d, "MCCs.json"))]
            isempty(runs) && continue
            r[arm] = (mean(scaled_vi(x[String(pair)], true_mccs) for x in runs), mean(length(x[String(pair)]) - length(true_mccs) for x in runs))
        end
        all(haskey(r, a) for a in arms) && push!(units, (replace(name, r"_rep\d+$" => ""), r))
    end
end
println("units (simulation x tree pair): ", length(units))
println("\nmean scaled VI to the truth / mean (#inferred - #true), by setting")
@printf("%-24s %4s", "setting", "n")
foreach(a -> @printf(" %22s", a), arms)
println()
for setting in unique(first.(units))
    xs = [u[2] for u in units if u[1] == setting]
    @printf("%-24s %4d", setting, length(xs))
    foreach(a -> @printf("      %.4f / %+7.2f", mean(x[a][1] for x in xs), mean(x[a][2] for x in xs)), arms)
    println()
end
rng = MersenneTwister(1)
for a in arms[2:end], (what, f) in (("VI", x -> x[1]), ("|#MCC error|", x -> abs(x[2])))
    d = [f(u[2][a]) - f(u[2][arms[1]]) for u in units]
    m, half = mean(d), 1.96std(d) / sqrt(length(d))
    p = (1 + count(_ -> abs(mean(d .* rand(rng, (-1, 1), length(d)))) >= abs(m) - 1e-15, 1:20000)) / 20001
    @printf("%s - %s, %s: mean difference %+.5f, 95%% CI [%+.5f, %+.5f], sign-flip p=%.4f, %s lower in %d units, higher in %d\n",
        a, arms[1], what, m, m - half, m + half, p, a, count(<(0), d), count(>(0), d))
end
