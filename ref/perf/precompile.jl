# Workload for the TreeKnit.jl system image: runs the command line on two small simulated fixtures,
# with three trees and with two. Passed to PackageCompiler as `precompile_execution_file`.
using TreeKnit

const SIM = normpath(joinpath(@__DIR__, "..", "..", "fixtures", "sim"))
out = mktempdir()
k3 = [joinpath(SIM, "sim_k3_n50_r0.1_poly", "tree$k.nwk") for k in 1:3]
k2 = [joinpath(SIM, "sim_k2_n100_r0.02_poly", "tree$k.nwk") for k in 1:2]
TreeKnit.command_main([k3..., "--outdir=$(joinpath(out, "k3"))", "--verbosity-level=-1"])
TreeKnit.command_main([k2..., "--outdir=$(joinpath(out, "k2"))", "--verbosity-level=-1"])
