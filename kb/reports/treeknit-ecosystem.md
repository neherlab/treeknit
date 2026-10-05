# TreeKnit.jl in practice: users, formats, and neighboring tools

This report describes how TreeKnit.jl is distributed and used, which downstream tools read its output, how well its output files conform to the formats of those tools, and how comparable tools present their interfaces. All facts were checked in primary sources: source code at a fixed commit, schemas, and official documentation. Source tiers follow the usual scale: primary (code, schemas, official docs), secondary (maintained repositories), tertiary (discussions).

## Summary

- **Small user base**: four pipelines on GitHub run the `treeknit` command. Two of them expect the file names of TreeKnit 0.4 (`MCCs.dat`, `*.resolved.nwk`) and cannot read the output of 0.5.x
- **No package outside Julia**: TreeKnit is installed only with the Julia package manager. A Bioconda recipe was abandoned
- **Auspice output is accepted by Auspice but not by the augur schema**: `meta.updated` is an empty string, which the schema rejects. The MCC values are strings, so Auspice draws a categorical scale instead of an ordinal one, and the string `"null"` becomes a legend entry instead of missing data
- **ARG output reads in IcyTree**: the format follows the extended Newick syntax plus a BEAST-style `[&segments=...]` annotation. TreeKnit-web pairs hybrid copies by their label string
- **Dagger does not order tasks that share plain mutable arguments**, which explains the race in TreeKnit's parallel mode

## Distribution

- **Julia General registry**: 22 versions from 0.1.0 to 0.5.8 (2024-08-09) [[primary](https://github.com/JuliaRegistries/General/tree/master/T/TreeKnit)]. The compatibility bound `julia = "1.7"` allows every Julia 1.x from 1.7, while the documentation says TreeKnit "may not work with higher or lower versions" (see [`cli.md`](../feat/cli.md))
- **Command-line install**: `deps/build.jl` runs Comonicon's installer, which writes `<depot>/bin/treeknit`, normally `~/.julia/bin/treeknit`. The script runs Julia with `--startup-file=no` and `--optimize=2`, single-threaded unless `nthreads` is set in `Comonicon.toml`, which TreeKnit does not do [[primary](https://github.com/comonicon/Comonicon.jl/blob/fe08cdb1a458f08df9b40bd515f3e165d3afc8c9/src/builder/install.jl#L6-L138)]. Completion is written for zsh or bash only
- **Option syntax**: Comonicon accepts `-o=val`, `-o val`, and `-oval` for short options and `--opt=val` for long ones [[primary](https://github.com/comonicon/Comonicon.jl/blob/fe08cdb1a458f08df9b40bd515f3e165d3afc8c9/src/codegen/julia.jl#L426-L438)]. At least one pipeline uses `-o=` and `-g=`
- **Bioconda**: the recipe pull request bioconda/bioconda-recipes#34130 (opened 2022-04-05) was closed without merge on 2024-01-31. A Nextstrain maintainer wrote in nextstrain/docker-base#222 (2024-07-18) about "trying unsuccessfully to get a TreeKnit Bioconda package built". Searches of anaconda.org, Docker Hub, and quay.io return no TreeKnit package

## Downstream users

Each entry links the call site at a fixed commit.

- **nextstrain/seasonal-flu**: a Snakemake rule runs `treeknit t1 t2 --outdir ...` and declares the outputs `tree_common_{segment}.resolved.nwk` and `MCCs.dat`, then calls `treetime arg` with the MCC file [[primary](https://github.com/nextstrain/seasonal-flu/blob/f34feaa4650fe3cfd1fa41b11574f9647a92065a/workflow/snakemake_rules/treeknit.smk#L4-L52)]. These are TreeKnit 0.4 names; TreeKnit 0.5 writes `<name>_resolved.nwk` and `MCCs.json`. No Snakefile on that commit includes the rule
- **neherlab/treetime `treetime arg`**: reads MCCs as one comma-separated list per line, the 0.4 `MCCs.dat` format, so it cannot read `MCCs.json` [[primary](https://github.com/neherlab/treetime/blob/892f3e7ecaff7954ffc985840fe824db13417a73/treetime/arg.py#L29-L34)]
- **blab/cartography**: runs `~/.julia/bin/treeknit ha na --better-MCCs -o=... -g=2` and reads `MCCs.json` and `*_resolved.nwk` [[primary](https://github.com/blab/cartography/blob/3fcac4ef2a73e5929ea018f11555ec84b88371dc/ha-na-nextstrain/Snakefile#L198-L217)]. Its converter takes the first entry of `MCC_dict` and labels the last MCC, which is the largest because of TreeKnit's sort order, as "unassigned" [[primary](https://github.com/blab/cartography/blob/3fcac4ef2a73e5929ea018f11555ec84b88371dc/ha-na-nextstrain/scripts/convert_mccs_to_node_json.py#L23-L56)]. Its Docker image installs TreeKnit on Julia 1.10
- **neherlab/CCHFV**: runs three segments with `treeknit S L M --better-trees --rounds 3 -o results` and passes `tree_*_resolved.nwk` to augur [[primary](https://github.com/neherlab/CCHFV/blob/f4dcc448ba106ff0453bdba8225a64bfae283d2e/Snakefile#L240-L256)]. Its README offers a workflow without TreeKnit because the Julia setup is hard on some platforms
- **mmolari/ecoliST131-structural-evo**: runs `treeknit a b -o dir --auspice-view` on bacterial trees, not influenza [[secondary](https://github.com/mmolari/ecoliST131-structural-evo/blob/2d41d70a155c8753a42e2f28e908dd4654c8d68c/exploration/2305a_SNPs_vs_edges/3_treeknit.py#L94-L100)]

The output properties that these users depend on are the command name and install path, the option syntax, the file names of 0.5, the JSON shape `MCC_dict -> "1" -> mccs`, and the order of the MCC list.

## Auspice JSON

TreeKnit writes `version`, `meta.updated = ""`, `meta.colorings` with type `ordinal`, `meta.filters`, `meta.panels = ["tree"]`, and per node `name`, `node_attrs.div`, one `node_attrs.mcc_<a>_<b>.value` string per other tree, and an empty `branch_attrs` (see [`formats.md`](../feat/formats.md)).

- **Schema**: the augur export schema requires `version`, `meta`, and `tree`, requires `meta.updated` and `meta.panels`, and restricts `updated` to the pattern `^[0-9X]{4}-[0-9X]{2}-[0-9X]{2}$` [[primary](https://github.com/nextstrain/augur/blob/0d287496eed3816f94d674f5a08201273f479487/augur/data/schema-export-v2.json#L18-L28)]. The empty string fails this pattern. The other fields conform
- **Auspice tolerance**: Auspice replaces a non-string `updated` with an empty string and draws the tree, so the files load
- **Ordinal scale**: Auspice builds an ordinal scale only when all values are integers, and otherwise warns "Using a categorical scale as currently ordinal scales must only contain integers" [[primary](https://github.com/nextstrain/auspice/blob/37bf9ce1e3b9a8cbdf1ebfd77bd8d7bdb6dfdc2d/src/util/colorScale.ts#L210-L257)]. TreeKnit's string values always give a categorical scale
- **Missing values**: Auspice treats non-scalar values and the strings `unknown`, `?`, `nan`, `na`, `n/a`, the empty string, and `unassigned` as missing [[primary](https://github.com/nextstrain/auspice/blob/37bf9ce1e3b9a8cbdf1ebfd77bd8d7bdb6dfdc2d/src/util/globals.js#L173-L194)]. The string `"null"` is a normal value with its own color
- **<a id="gloss-use-1"></a>Tanglegram <sup>[1](#gloss-1)</sup>**: auspice.us treats any dropped JSON with `meta` and `tree` as a dataset and, with two or more, loads the first two in drop order as the left and right trees ("we have no sensible way of ordering these") [[primary](https://github.com/nextstrain/auspice.us/blob/f8beac16f71ea0f38e81f7a1449e3e26b287dfcb/auspice_client_customisation/handleDroppedFiles.js#L80-L171)]. File names do not matter. Tips are paired by identical `name` [[primary](https://docs.nextstrain.org/projects/auspice/en/stable/advanced-functionality/second-trees.html)]

## Extended Newick ARG

- **Cardona syntax**: a <a id="gloss-use-2"></a>hybrid node <sup>[2](#gloss-2)</sup> is written once per parent as `label#<type><i>`, with the type `R`, `H`, or `LGT`, and the first copy carries the subtree <a id="cite-1"></a>[Cardona et al. 2008](https://doi.org/10.1186/1471-2105-9-532) [[1](#ref-1)]. TreeKnit writes `label#H<i>`, with the subtree on the first copy. The `[&segments={...}]` comment is outside the Cardona grammar
- **IcyTree**: the parser reads label, `#` id, `[&...]` annotations before or after the length, and the length [[primary](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treeparsing.js#L219-L345)]. It reads `segments={0,1}` as a list of strings, which can serve as a color trait. It merges hybrid copies by id: the copy with children is the source, and copies without children become reticulation edges [[primary](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/tree.js#L223-L268)]. Unquoted labels must not contain `,():;[]#`; TreeKnit never quotes labels, so strain names with these characters break the file
- **TreeKnit-web**: a Next.js prototype with all commits on 2022-12-02, deployed at <https://treeknit-web.vercel.app>. It parses with IcyTree's parser and then pairs nodes whose **label strings are equal**, ignoring the `#` id [[primary](https://github.com/neherlab/TreeKnit-web/blob/61a489f7618582959db086838ef31ddf98bb6900/src/components/Tree/nwk/convertIcyTreeToGraph.ts#L54-L71)]. This works for TreeKnit's ARGs because every node there has a unique label

## Comparable tools and their interfaces

- **TreeSort** (Python, MIT; 0.3.1, 2025-06-26, on PyPI): `treesort -i descriptor.csv -o out.tre`, where the descriptor lists segment, alignment, and tree per segment and marks the reference segment. The output is one annotated tree for FigTree or IcyTree [[primary](https://github.com/flu-crew/TreeSort/blob/85aad631f0bc5b645ed7eab3d808886165fe844b/treesort/options.py#L29-L67)]. It needs alignments; TreeKnit needs only trees
- **CoalRe** (Java, BEAST 2 package; 3.0.3, 2026-01-29): configured through BEAST XML or BEAUti, with no command line comparable to TreeKnit's [[primary](https://github.com/nicfel/CoalRe)]
- **GiRaF**: reads MrBayes tree samples; the repositories have no activity since 2016 [[primary](https://github.com/CSB5/GiRaF)]
- **RF-Net 2**, **VReassort**: research code accompanying the papers in [`reassortment-methods-literature.md`](reassortment-methods-literature.md)

## Julia libraries behind TreeKnit features

- **Dagger task dependencies**: in Dagger 0.16, only task results passed as arguments create dependencies, and any other object is "passed as-is" [[primary](https://github.com/JuliaParallel/Dagger.jl/blob/66d51472cdb60cf18dcddbc6536a0a4abd97402d/docs/src/index.md#L20-L23)]. Ordered access to shared mutable data needs the datadeps API, which arrived in Dagger 0.18.7 (2024-01-29), after TreeKnit's `Dagger = 0.16` bound [[primary](https://github.com/JuliaParallel/Dagger.jl/blob/f910695cf783589b5756922921f5823a232332b4/docs/src/datadeps.md#L1-L45)]. TreeKnit's parallel mode passes the trees and the options as plain objects (see [`pipeline.md`](../feat/pipeline.md))
- **ProgressMeter**: writes to `stderr` by default [[primary](https://github.com/timholy/ProgressMeter.jl)], which is where the annealing progress bar appears

## Gaps

- **Configurations.jl**: whether the empty `[sysimg]` table in `Comonicon.toml` triggers a system image build was not checked
- **Community discussion**: Reddit and Hacker News searches for TreeKnit returned nothing relevant
- **Private use**: only public GitHub code was searched

## Glossary

1. <a id="gloss-1"></a> **Tanglegram.** A drawing of two rooted trees facing each other, with lines that join the leaves of the same name. [↩](#gloss-use-1)
2. <a id="gloss-2"></a> **Hybrid node.** A node of a phylogenetic network with more than one parent; in a reassortment graph, the node where the segments of one virus have different parents ([Cardona et al. 2008](https://doi.org/10.1186/1471-2105-9-532) [[1](#ref-1)]). [↩](#gloss-use-2)

## References

1. <a id="ref-1"></a> Cardona, Gabriel, Francesc Rosselló, and Gabriel Valiente. 2008. "Extended Newick: It Is Time for a Standard Representation of Phylogenetic Networks." _BMC Bioinformatics_ 9: 532. https://doi.org/10.1186/1471-2105-9-532 [↩](#cite-1)
