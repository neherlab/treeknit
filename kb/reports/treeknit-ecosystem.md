# TreeKnit.jl in practice: users, formats, and neighboring tools

This report describes how TreeKnit.jl is distributed and used, which downstream tools read its output, how well its output files conform to the formats of those tools, and how comparable tools present their interfaces. Each statement links to source code at a fixed commit, a schema, or official documentation.

## Summary

- **Small user base**: four public repositories run the `treeknit` command. One of them, a Nextstrain seasonal-flu rule that no workflow includes, expects the file names of TreeKnit 0.4 (`MCCs.dat`, `*.resolved.nwk`). TreeTime `arg` reads the 0.4 <a id="gloss-use-1"></a>MCC <sup>[1](#gloss-1)</sup> file and cannot read the `MCCs.json` of TreeKnit 0.5
- **No package outside Julia**: TreeKnit is installed only with the Julia package manager. A Bioconda recipe was abandoned
- **Auspice reads the JSON output, the augur schema does not accept it**: `meta.updated` is an empty string, which fails the schema pattern. The MCC values are strings, so Auspice draws a <a id="gloss-use-2"></a>categorical color scale <sup>[2](#gloss-2)</sup>, and it shows the string `"null"` as one more category
- **IcyTree reads the <a id="gloss-use-3"></a>ARG <sup>[3](#gloss-3)</sup> output**: the format follows the <a id="gloss-use-4"></a>extended Newick <sup>[4](#gloss-4)</sup> syntax plus a BEAST-style `[&segments=...]` annotation. TreeKnit-web pairs the copies of a <a id="gloss-use-5"></a>hybrid node <sup>[5](#gloss-5)</sup> by their label
- **Dagger does not order tasks that share plain mutable arguments**, which explains the race in TreeKnit's parallel mode

## Distribution

- **Julia General registry**: 22 versions from 0.1.0 to 0.5.8 (2024-08-09) [[src](https://github.com/JuliaRegistries/General/blob/50eb035ce3411d6a0a4e5f93f062f7275f72c166/T/TreeKnit/Versions.toml)]. The compatibility bound `julia = "1.7"` allows every Julia 1.x from 1.7 (1.7.0 released 2021-11-30), and the documentation says TreeKnit "may not work with higher or lower versions" (see [`cli.md`](../feat/v0/cli.md))
- **Command-line install**: `deps/build.jl` runs the installer of Comonicon 1.0.8 (2024-04-22), which writes `<depot>/bin/treeknit`, normally `~/.julia/bin/treeknit` [[src](https://github.com/comonicon/Comonicon.jl/blob/fe08cdb1a458f08df9b40bd515f3e165d3afc8c9/src/builder/install.jl#L6-L138)]:
  - The script runs Julia with `--startup-file=no` and `--optimize=2`
  - It runs single-threaded unless `nthreads` is set in `Comonicon.toml`, and TreeKnit does not set it
  - Shell completion is written for zsh or bash only
- **Option syntax**: Comonicon accepts `-o=val`, `-o val`, and `-oval` for short options [[src](https://github.com/comonicon/Comonicon.jl/blob/fe08cdb1a458f08df9b40bd515f3e165d3afc8c9/src/codegen/julia.jl#L414-L448)], and `--opt=val` and `--opt val` for long ones [[src](https://github.com/comonicon/Comonicon.jl/blob/fe08cdb1a458f08df9b40bd515f3e165d3afc8c9/src/codegen/julia.jl#L499-L522)]. The `blab/cartography` pipeline uses `-o=` and `-g=`
- **Bioconda**: the recipe pull request [bioconda/bioconda-recipes#34130](https://github.com/bioconda/bioconda-recipes/pull/34130) (opened 2022-04-05) was closed without merge on 2024-01-31. A Nextstrain maintainer wrote in [nextstrain/docker-base#222](https://github.com/nextstrain/docker-base/issues/222#issuecomment-2237103863) (2024-07-18) about "trying unsuccessfully to get a TreeKnit Bioconda package built". Searches of anaconda.org, Docker Hub, and quay.io return no TreeKnit package

## Downstream users

Each entry links the call site at a fixed commit.

- **nextstrain/seasonal-flu**: a Snakemake rule runs `treeknit t1 t2 --outdir ...` and declares the outputs `tree_common_{segment}.resolved.nwk` and `MCCs.dat`, then calls `treetime arg` with the MCC file [[src](https://github.com/nextstrain/seasonal-flu/blob/f34feaa4650fe3cfd1fa41b11574f9647a92065a/workflow/snakemake_rules/treeknit.smk#L4-L52)]. These are TreeKnit 0.4 names. TreeKnit 0.5 writes `<name>_resolved.nwk` and `MCCs.json`. No Snakefile on that commit includes the rule
- **neherlab/treetime `treetime arg`**: reads TreeKnit output and does not run the command
  - **Format**: it reads MCCs as one comma-separated list per line, the 0.4 `MCCs.dat` format, so it cannot read `MCCs.json` [[src](https://github.com/neherlab/treetime/blob/892f3e7ecaff7954ffc985840fe824db13417a73/treetime/arg.py#L29-L34)]
  - **Number of trees**: its command line takes exactly two trees and two alignments [[src](https://github.com/neherlab/treetime/blob/892f3e7ecaff7954ffc985840fe824db13417a73/treetime/argument_parser.py#L475-L477)]
  - **Open pull requests for more trees**:
    - [#219](https://github.com/neherlab/treetime/pull/219): "Feature: Extend ARG functionality to MultiTreeKnit", last updated 2022-12-21
    - [#961](https://github.com/neherlab/treetime/pull/961), last updated 2026-10-04: reads `MCCs.json` and analyzes each tree in turn as the focal tree. Each branch of the focal tree gets the alignments of the other segments whose MCC with the focal tree contains the branch [[src](https://github.com/neherlab/treetime/blob/1343ea8f927ec2c1d89ed1cf1e782d1853f8d293/treetime/arg.py#L138-L160)]. It uses only the pairs that contain the focal tree and reads no ARG
- **blab/cartography**: runs `~/.julia/bin/treeknit ha na --better-MCCs -o=... -g=2` and reads `MCCs.json` and `*_resolved.nwk` [[src](https://github.com/blab/cartography/blob/3fcac4ef2a73e5929ea018f11555ec84b88371dc/ha-na-nextstrain/Snakefile#L198-L217)]. Its converter takes the first entry of `MCC_dict` and labels the last MCC, which is the largest because of TreeKnit's sort order, as "unassigned" [[src](https://github.com/blab/cartography/blob/3fcac4ef2a73e5929ea018f11555ec84b88371dc/ha-na-nextstrain/scripts/convert_mccs_to_node_json.py#L23-L56)]. Its Docker image installs TreeKnit on Julia 1.10 (1.10.0 released 2023-12-26)
- **neherlab/CCHFV**: runs three segments with `treeknit S L M --better-trees --rounds 3 -o results` and passes `tree_*_resolved.nwk` to augur [[src](https://github.com/neherlab/CCHFV/blob/f4dcc448ba106ff0453bdba8225a64bfae283d2e/Snakefile#L240-L256)]. Its README offers a workflow without TreeKnit because the Julia setup is hard on some platforms
- **mmolari/ecoliST131-structural-evo**: runs `treeknit a b -o dir --auspice-view` on bacterial trees [[src](https://github.com/mmolari/ecoliST131-structural-evo/blob/2d41d70a155c8753a42e2f28e908dd4654c8d68c/exploration/2305a_SNPs_vs_edges/3_treeknit.py#L94-L100)]

These users depend on the command name and install path, the option syntax, the file names of 0.5, the JSON shape `MCC_dict -> "1" -> mccs`, and the order of the MCC list.

## Auspice JSON

TreeKnit writes `version`, `meta.updated = ""`, `meta.colorings` with type `ordinal`, `meta.filters`, `meta.panels = ["tree"]`, and per node `name`, `node_attrs.div`, one `node_attrs.mcc_<a>_<b>.value` string per other tree, and an empty `branch_attrs` (see [`formats.md`](../feat/v0/formats.md)).

- **Schema**: the augur export schema requires `version`, `meta`, and `tree`, requires `meta.updated` and `meta.panels`, and restricts `updated` to the pattern `^[0-9X]{4}-[0-9X]{2}-[0-9X]{2}$` [[src](https://github.com/nextstrain/augur/blob/0d287496eed3816f94d674f5a08201273f479487/augur/data/schema-export-v2.json#L18-L28)]. The empty string fails this pattern. The other fields conform
- **Auspice handling of `updated`**: Auspice keeps a string value of `updated` and replaces any other value with an empty string, so the files load [[src](https://github.com/nextstrain/auspice/blob/37bf9ce1e3b9a8cbdf1ebfd77bd8d7bdb6dfdc2d/src/util/metadataJsonParsing.ts#L26)]
- **Ordinal scale**: Auspice builds an ordinal scale only when all values are integers, and otherwise warns "Using a categorical scale as currently ordinal scales must only contain integers" [[src](https://github.com/nextstrain/auspice/blob/37bf9ce1e3b9a8cbdf1ebfd77bd8d7bdb6dfdc2d/src/util/colorScale.ts#L210-L257)]. TreeKnit's string values always give a categorical scale
- **Missing values**: Auspice treats non-scalar values and the strings `unknown`, `?`, `nan`, `na`, `n/a`, the empty string, and `unassigned` as missing [[src](https://github.com/nextstrain/auspice/blob/37bf9ce1e3b9a8cbdf1ebfd77bd8d7bdb6dfdc2d/src/util/globals.js#L173-L194)]. The string `"null"` is a normal value with its own color
- **<a id="gloss-use-6"></a>Tanglegram <sup>[6](#gloss-6)</sup>**: auspice.us treats any dropped JSON with `meta` and `tree` as a dataset. With two or more datasets, it loads the first two in drop order as the left and right trees ("we have no sensible way of ordering these") [[src](https://github.com/nextstrain/auspice.us/blob/f8beac16f71ea0f38e81f7a1449e3e26b287dfcb/auspice_client_customisation/handleDroppedFiles.js#L80-L171)]. File names have no effect. Tips are paired by identical name [[doc](https://docs.nextstrain.org/projects/auspice/en/stable/advanced-functionality/second-trees.html#:~:text=draw%20lines%20between%20tips%20with%20the%20same%20name)]

## Extended Newick ARG

- **Cardona syntax**: a hybrid node is written once per parent as `label#<type><i>`, with the type `R`, `H`, or `LGT`, and the first copy carries the subtree <a id="cite-1a"></a>[Cardona et al. 2008](https://doi.org/10.1186/1471-2105-9-532) [[1](#ref-1)]. TreeKnit writes `label#H<i>`, with the subtree on the first copy. The `[&segments={...}]` comment is outside the Cardona grammar
- **IcyTree** [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/treeparsing.js#L219-L345)] [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/tree.js#L223-L268)]:
  - The parser reads the label, the `#` id, `[&...]` annotations before or after the length, and the length
  - It reads `segments={0,1}` as a list of strings, which the user can choose as a color trait
  - It merges hybrid copies by id. The copy with children is the source, and copies without children become <a id="gloss-use-7"></a>reticulation edges <sup>[7](#gloss-7)</sup>
  - Unquoted labels must not contain `,():;[]#`. TreeKnit never quotes labels, so strain names with these characters break the file
  - An id can have more than two copies. All copies without children become reticulation edges, so a node with more than two parents loads. An id with one copy is an error: "hybrid nodes must come in groups of 2 or more" [[src](https://github.com/tgvaughan/icytree/blob/af1836d8130e805613470f07054efa3f3720a913/js/tree.js#L255-L260)]
- **TreeKnit-web**: a Next.js prototype with all commits on 2022-12-02, deployed at <https://treeknit-web.vercel.app>. It parses with the IcyTree parser and then pairs nodes whose label strings are equal [[src](https://github.com/neherlab/TreeKnit-web/blob/61a489f7618582959db086838ef31ddf98bb6900/src/components/Tree/nwk/convertIcyTreeToGraph.ts#L54-L71)]. This works for TreeKnit's ARGs because every other node there has a unique label
- **More than two segments**: TreeKnit writes ARGs of two segments only. Two related tools write ARGs of more segments:
  - **ARGTools**, the simulator of TreeKnit's tests:
    - **Syntax**: TreeKnit's, with `hybrid_node_<i>#H<i>` numbered from 1 and `[&segments={0,1,2}]` without spaces. Its test covers three segments [[src](https://github.com/PierreBarrat/ARGTools/blob/824b371cd0a2fd79fe80d4848e3445b3e6718686/tests/test_extnwk.jl#L59-L69)]
    - **Reassortments**: each simulated reassortment splits the segments of a lineage into two groups [[src](https://github.com/PierreBarrat/ARGTools/blob/824b371cd0a2fd79fe80d4848e3445b3e6718686/src/simulate.jl#L383-L384)]
  - **CoalRe** writes its own variant [[src](https://github.com/nicfel/CoalRe/blob/39442ea90fd05ffc2b338affc414e97a7bcee4b9/src/coalre/network/Network.java#L229-L331)]:
    - `#H<i>`, numbered from 0
    - `segments=` as the text of a Java `BitSet`, which has spaces (`{0, 2}`)
    - the additional keys `realCoal` and `segsCarried`
    - exactly two parent edges per reassortment node

## Comparable tools and their interfaces

- **TreeSort** (Python, MIT license; 0.3.1 released 2025-06-26, on PyPI): `treesort -i descriptor.csv -o out.tre`. The descriptor lists segment, alignment, and tree per segment and marks the reference segment. The output is one annotated tree for FigTree or IcyTree [[src](https://github.com/flu-crew/TreeSort/blob/85aad631f0bc5b645ed7eab3d808886165fe844b/treesort/options.py#L29-L67)]. TreeSort needs alignments, and TreeKnit needs only trees
- **CoalRe** (Java, BEAST 2 package; 3.0.3 released 2026-01-29): configured through BEAST XML or BEAUti, with no command line comparable to TreeKnit's [[src](https://github.com/nicfel/CoalRe/tree/39442ea90fd05ffc2b338affc414e97a7bcee4b9)]
- **GiRaF**: reads MrBayes tree samples. The repository has no commit after 2016-10-25 [[src](https://github.com/CSB5/GiRaF/tree/24848f15e2b60aff884c43c78eb8195f0fc8be92)]
- **RF-Net 2**, **VReassort**: research code that accompanies the papers in [`reassortment-methods-literature.md`](reassortment-methods-literature.md)

## Julia libraries behind TreeKnit features

- **Dagger task dependencies**: in Dagger 0.16 (0.16.0 released 2022-07-24), only task results passed as arguments create dependencies. For any other argument, "it'll be passed as-is to the function `f`" [[doc](https://github.com/JuliaParallel/Dagger.jl/blob/66d51472cdb60cf18dcddbc6536a0a4abd97402d/docs/src/index.md?plain=1#L20-L23)]. Ordered access to shared mutable data needs the datadeps API, which arrived in Dagger 0.18.7 (2024-01-29), after TreeKnit's `Dagger = 0.16` bound [[doc](https://github.com/JuliaParallel/Dagger.jl/blob/f910695cf783589b5756922921f5823a232332b4/docs/src/datadeps.md?plain=1#L1-L45)]. TreeKnit's parallel mode passes the trees and the options as plain objects (see [`pipeline.md`](../feat/v0/pipeline.md))
- **ProgressMeter**: version 1.7.2 (2022-03-24) writes to `stderr` by default [[src](https://github.com/timholy/ProgressMeter.jl/blob/83e619faed2780ab5d6929777c8f6151e3b88c5e/src/ProgressMeter.jl#L86)], which is where the annealing progress bar appears

## Gaps

- **Configurations.jl**: whether the empty `[sysimg]` table in `Comonicon.toml` triggers a system image build was not checked
- **Community discussion**: Reddit and Hacker News searches for TreeKnit returned no relevant posts
- **Private use**: only public GitHub code was searched

## Glossary

1. <a id="gloss-1"></a> **MCC (maximally compatible clade).** In TreeKnit, a largest set of leaves whose subtrees have the same topology in two segment trees. [↩](#gloss-use-1)
2. <a id="gloss-2"></a> **Categorical color scale.** A scale that gives each distinct value its own color, without order. An ordinal scale orders the values and colors them along a gradient. [↩](#gloss-use-2)
3. <a id="gloss-3"></a> **ARG (ancestral reassortment graph).** A graph that holds the genealogies of all segments, with one node of two parents for each reassortment. [↩](#gloss-use-3)
4. <a id="gloss-4"></a> **Extended Newick.** A Newick string in which each node with more than one parent appears once per parent <a id="cite-1b"></a>[Cardona et al. 2008](https://doi.org/10.1186/1471-2105-9-532) [[1](#ref-1)]. [↩](#gloss-use-4)
5. <a id="gloss-5"></a> **Hybrid node.** A node of a phylogenetic network with more than one parent. In a reassortment graph, it marks the point where the segments of one virus have different parents. [↩](#gloss-use-5)
6. <a id="gloss-6"></a> **Tanglegram.** A drawing of two rooted trees facing each other, with lines that join the leaves of the same name. [↩](#gloss-use-6)
7. <a id="gloss-7"></a> **Reticulation edge.** An edge into a hybrid node from one of its parents. [↩](#gloss-use-7)

## References

1. <a id="ref-1"></a> Cardona, Gabriel, Francesc Rosselló, and Gabriel Valiente. 2008. "Extended Newick: It is time for a standard representation of phylogenetic networks." _BMC Bioinformatics_ 9:532. https://doi.org/10.1186/1471-2105-9-532 [↩¹](#cite-1a) [↩²](#cite-1b)
