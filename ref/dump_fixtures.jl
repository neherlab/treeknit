# Reference fixtures from docs / test examples / real data of the legacy Julia TreeKnit.
# Run: julia --project=/tmp/tkref/TreeKnit /workspace/treeknit-rs/ref/dump_fixtures.jl [--no-runs]
include(joinpath(@__DIR__, "fixture_lib.jl"))

const LEGACY = get(ENV, "TREEKNIT_LEGACY", "/workspace/legacy_julia_version")
const NRUNS = "--no-runs" in ARGS ? 0 : 20
nwkfile(p) = strip(read(joinpath(LEGACY, p), String))

# (name, newicks, given MCCs or nothing). Given MCCs come from the legacy tests.
const CASES = [
    # docs/src/mccs.md
    ("doc_mccs_1", ["((A,B),(C,(D,X)));", "((A,(B,X)),(C,D));"], nothing),
    ("doc_mccs_2", ["(((A1,A2),(B1,B2)),(C1,C2));", "(((A1,A2),(C1,C2)),(B1,B2));"], nothing),
    # docs/src/resolving.md
    ("doc_resolve_2", ["(A,(B,C));", "(A,B,C);"], nothing),
    ("doc_resolve_3", ["(A,(B,C));", "(A,B,C);", "(A,B,C);"], nothing),
    ("doc_resolve_4", ["(A,(B,C));", "(A,B,C);", "(A,B,C);", "((A,B),C);"], nothing),
    ("doc_resolve_mcc", ["((A,B),(C,(D,(E,X))));", "((A,(B,X)),(C,D,E));"], nothing),
    ("doc_resolve_liberal", ["((A,(B,C)),D);", "(A,B,C,D);"], nothing),
    # docs/src/options.md, runopt.md, opttrees.md, multitreeknit.md
    ("doc_options_shuffled", ["((((A,B),C),D),E);", "((((D,B),E),A),C);"], nothing),
    ("doc_options_bl", ["((A:2,B:2):2,C:4);", "(A:2,(B:1,C:1):1);"], nothing),
    ("doc_runopt", ["(Z,(G,(((A,X),(B,C)),((D,Y),(E,F)))));", "(G,((A,(B,(C,X))),((D,(E,(F,Y))),Z)));"], nothing),
    ("doc_opttrees", ["(((A1:1,A2:2):2,(B1:2,(B2:1,B3:1):1):2):2,(C1:1,C2:2):4);",
                      "((A1:1,A2:2):2,((B1:2,(B2:1,B3:1):1):1,(C1:1,C2:2):1):1);"], nothing),
    ("doc_multitreeknit_3", ["((A,(B,C)),(D,E));", "((A,B,C,D),E);", "((A,B),((C,D),E));"], nothing),
    # test/main
    ("test_output_bl", ["((1:1.0,2:1.0):1.0,((3:1.0,4:1.0):1.0,5:1.0,6:1.0):1.5);",
                        "((6:1,2:1):1,((3:1,4:1):1,5:1,1:1):1);"], nothing),
    ("test_run_treeknit_3", ["((A,B),C)R;", "(A,(B,C))R;", "(A,B,C)R;"], nothing),
    ("test_run_parallel_5", ["(((A,B),C),(D,E,F))R;", "((A,(B,C)),(D,E,F))R;", "((A,B,C),((D,E),F))R;",
                             "((A,B,C),(D,(E,F)))R;", "((A,(B,C)),(D,(E,F)))R;"], nothing),
    # test/splitgraph
    ("test_sg_basic", [nwkfile("test/splitgraph/basic/tree1.nwk"), nwkfile("test/splitgraph/basic/tree2.nwk")], nothing),
    ("test_sg_identical", ["((A,B),(C,D));", "((A,B),(C,D));"], nothing),
    ("test_sg_3solutions", [nwkfile("test/splitgraph/3solutions/t1.nwk"), nwkfile("test/splitgraph/3solutions/t2.nwk")], nothing),
    ("test_sg_w_resolution", ["((A,B),(D,(E,(F,C))));", "((A,(B,C)),(D,E,F));"], nothing),
    # test/SRG
    ("test_srg_1", ["(A:3,(B:2,C:2)BC:1)R;", "((A:3,B:3)AB:1,C:4)R;"], nothing),
    ("test_srg_2", ["(((((A:2,X3)AX3:1,X4)AX4:2,X5)AX5:2,B)AB:0.,X1,X2):0.R;",
                    "((((A:1,X2)AX2:3,X1)AX1:3,B)AB:0.,X3,X4,X5):0.R;"], nothing),
    ("test_srg_3", ["(((A:1,B:1)AB:1,C:2)ABC:1,D:3)R;", "((A:3,D:3)AD:1,(B:3,C:3)BC:1)R;"], nothing),
    ("test_srg_singletons", ["((A:1,B:1)AB:1,C:2)R:0;", "(A:4,(B:0,C:0):4)R:0;"], nothing),
    # test/resolving
    ("test_resolve_basic", ["(((A,B),C),D);", "(B,C,(A,D));"], nothing),
    ("test_resolve_files12", [nwkfile("test/resolving/tree1.nwk"), nwkfile("test/resolving/tree2.nwk")], nothing),
    ("test_resolve_3_compatible", ["(A,B,C,D);", "(A,(B,C,D));", "(A,B,(C,D));"], nothing),
    ("test_resolve_3_incompatible", ["(A,B,C,D);", "(A,(B,C,D));", "((A,B),(C,D));"], nothing),
    ("test_resolve_files34", [nwkfile("test/resolving/tree3.nwk"), nwkfile("test/resolving/tree4.nwk")],
        TreeKnit.read_mccs(joinpath(LEGACY, "test/resolving/mccs34.dat"))),
    ("test_strict_1a", ["((A,B,C,D),E);", "(((A,B),C),(D,E));"], [["A", "B", "C"], ["D", "E"]]),
    ("test_strict_1b", ["((A,B,C,D),E);", "(((A,B),C),(D,E));"], [["A", "B", "C"], ["D"], ["E"]]),
    ("test_strict_2a", ["((((A,B),C,D),E),F);", "((((A,B),(C,D)),E),F);"], [["A", "B", "E", "F"], ["C", "D"]]),
    ("test_strict_2b", ["(((A,B),C,D,E),F);", "((((A,B),(C,D)),E),F);"], [["A", "B", "E", "F"], ["C", "D"]]),
    ("test_strict_2c", ["((A,B),C,D,E,F);", "((((A,B),(C,D)),E),F);"], [["A", "B", "E", "F"], ["C", "D"]]),
    ("test_strict_sister", ["(((3,2),(4,6)),(7,1),5);", "(((3,2),(1,7)),(5,(4,6)));"], [["5"], ["4"], ["6"], ["1", "2", "3", "7"]]),
    ("test_strict_unneeded", ["((A,B,C,D,E),X);", "((((A,B),C),(D,E)),X);"], [["A", "B", "C"], ["D", "E", "X"]]),
    ("test_sortpoly_1", ["((A,B,C,(D,E)),X);", "((((A,B),C),(D,E)),X);"], [["A", "B", "C"], ["D", "E", "X"]]),
    ("test_sortpoly_2", ["(A,(C,D,E,B));", "(A,(E,(C,D),B));"], [["B"], ["A", "C", "D", "E"]]),
    ("test_sortpoly_3", ["(E,(B,C,D,A),K);", "(A,E,B,C,D,K);"], [["A", "B", "C", "D", "E"], ["K"]]),
    ("test_sortpoly_4", ["((A,B1),B2,C,D);", "((A,B1,B2,D),C);"], [["D"], ["A", "B1", "B2", "C"]]),
    ("test_sortpoly_5", ["(A,B,C,D);", "(A,C,(D,B));"], [["B"], ["A", "C", "D"]]),
    # real data
    ("real_ny", [nwkfile("test/NYdata/tree_ha.nwk"), nwkfile("test/NYdata/tree_na.nwk")], nothing),
    ("real_h3n2_examples", [nwkfile("examples/tree_h3n2_ha.nwk"), nwkfile("examples/tree_h3n2_na.nwk")], nothing),
    ("real_n18", [nwkfile("test/splitgraph/ondata/N18/tha.nwk"), nwkfile("test/splitgraph/ondata/N18/tna.nwk")], nothing),
]

only = filter(a -> !startswith(a, "--"), ARGS)
for (name, nwks, mccs) in CASES
    (isempty(only) || name in only) || continue
    make_fixture(name, nwks; mccs, nruns = NRUNS, nmulti = 5)
end
