# Simulation helpers shared by simulate.jl and perf/simulate.jl.
using Distributions, TreeTools

# Port of TestRecombTools.remove_branches!: delete each internal branch shorter than an
# Exponential(c*N) draw (one draw per node, pre-order); leaf branches shorter than the draw get tau=0.
function remove_branches!(n::TreeNode, p::Distribution)
    child_list = copy(n.child)
    r = rand(p)
    if !isroot(n) && !ismissing(n.tau) && n.tau < r   # root guard: ARGTools roots have tau=0
        n.isleaf ? (n.tau = 0.0) : TreeTools.delete_node!(n)
    end
    foreach(c -> remove_branches!(c, p), child_list)
end
remove_branches!(t::Tree, c, N) = (remove_branches!(t.root, Exponential(c * N)); node2tree!(t, t.root); t)
