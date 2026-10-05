# Knowledge Base

Shared knowledge base (KB). AI agents and humans collaborate here: documenting progress, tracking issues, recording design decisions, and maintaining project knowledge.

## Motivation

- AI continuity. Sessions start with zero memory. KB persists decisions, defects, and findings so knowledge compounds instead of being rediscovered.
- Human onboarding. Synthesized view of algorithms, design rationale, open problems, and research context.
- Multi-agent coordination. Parallel agents working on different issues share state through issues, decisions, and proposals.
- Scientific rigor. The port must track what is correct, what diverges from the published algorithm, and what diverges from the reference implementation, TreeKnit.jl.
- Preventing rework. Known issues and errata prevent re-investigating solved problems or re-introducing fixed bugs.
- Decision traceability. Design choices are documented with rationale, not implicit in code.

## Directories

Each directory is created with its first entry.

| Directory     | Description                                                                                                                                                                                                                 |
| ------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `_raw/`       | Human-produced source material (specifications, papers, notes). Read-only for AI.                                                                                                                                           |
| `algo/`       | Algorithm documentation: scientific background, implementation status, locations in TreeKnit.jl and in this port                                                                                                            |
| `decisions/`  | Deliberate design choices with rationale (one file per decision)                                                                                                                                                            |
| `feat/`       | Features. `v0/` inventories what TreeKnit.jl, the reference implementation, does, with links to its source. The documents next to `v0/` are the parity checklist of this port; `overview.md` is the index. They mark each feature `[x]` done, `[/]` partial, or `[ ]` not done |
| `issues/`     | Concrete problems. Severity-prefixed (H/M/N). The working list agents consult before domain work. PREFER independent issues, but entangled problems may share a file when splitting would lose clarity                      |
| `proposals/`  | Undecided design documents analyzing a problem space with options and tradeoffs. Every actionable item in a proposal must be extracted into a separate issue so it is not lost when the proposal is no longer actively read |
| `reports/`    | Research reports on algorithms, optimization methods, and implementation analysis                                                                                                                                           |
| `ref-errata/` | Defects in TreeKnit.jl that this port correctly avoids (2+ evidence sources required)                                                                                                                                       |

## Structure

`_raw/` contains source material. All other directories contain AI-maintained derived knowledge. Source code is ground truth. KB entries are guides, not substitutes for code verification.

The knowledge base holds prose and data only -- no executable code. Reference scripts belong in [`ref/`](../ref/), development scripts in [`dev/`](../dev/).

When new material is added to `_raw/`, dependent articles in other directories should be reviewed and updated.

`feat/v0/` describes TreeKnit.jl as it is, including its defects. It is reference material outside the taxonomy below and holds no work items. `ref-errata/` records the TreeKnit.jl defects that this port avoids, with evidence, and `algo/` documents each algorithm in both implementations.

## Taxonomy

Every work item falls into exactly one category:

| Category                            | Directory                                         | Scope                                                                         |
| ----------------------------------- | ------------------------------------------------- | ----------------------------------------------------------------------------- |
| Implemented, same behavior          | `algo/`, `feat/`                                  | Feature with equivalent behavior to TreeKnit.jl or the published algorithm    |
| Implemented, different behavior     | `decisions/`                                      | Feature with deliberate divergence from TreeKnit.jl, or intentionally removed |
| Not yet done                        | `issues/`                                         | Bugs, missing features, stubs, unused flags, behavioral differences           |
| New in this port                    | `proposals/` (pre-impl), `decisions/` (post-impl) | Feature not in TreeKnit.jl                                                    |
| TreeKnit.jl defective, port correct | `ref-errata/`                                     | TreeKnit.jl defect that this port does not reproduce                          |

### Decision rules

- Feature working with same results: `feat/` `[x]`, algorithm in domain file
- Feature working with different results: `decisions/` (one file with rationale)
- Feature intentionally removed: `decisions/` (one file with rationale)
- Feature missing, stubbed, or broken: `issues/` (severity-prefixed file)
- New feature: `proposals/` pre-implementation, then `decisions/` post-implementation
- TreeKnit.jl behavior wrong, port correct: `ref-errata/` (one file with evidence)

### Severity (issues only)

| Prefix | Severity   | Criteria                                                                                         |
| ------ | ---------- | ------------------------------------------------------------------------------------------------ |
| `H-`   | High       | Crashes, data loss, incorrect scientific results, or blocked required behavior                   |
| `M-`   | Medium     | Incorrect behavior under bounded conditions or a specified capability gap                        |
| `N-`   | Negligible | Documentation, test, maintainability, or presentation defect with no demonstrated runtime effect |

Derive severity from specification language, user requirements, external evidence, or demonstrated impact. If the evidence does not distinguish a severity, do not infer one from assumed usage frequency.

### Proposal lifecycle

- Proposal records the problem space, design axes, options, and tradeoffs.
- Extract every actionable item into a separate issue. Keep open design questions in their issues until the required decisions are approved.

### Issue lifecycle

- An issue records the problem, its locations, the fix direction, and the validation plan when one is known. It is the unit of work an agent or developer picks up
- Open design questions stay in the issue until the required decisions are approved. Mark them with a `> [!IMPORTANT]` block that starts with **Decision required.** and states the conflict, the options, and the evidence without choosing an option
- Mark a claim that lacks evidence (an unreproduced defect, an unmeasured cost, an unverified contract) with a `> [!IMPORTANT]` block that starts with **Investigation required.** and names the evidence to collect
- Mark statements that code changes may have made stale with a `> [!WARNING]` block that starts with **Needs review.** and gives the evidence
- Delete the issue when it is fully resolved. On partial resolution, update it to describe the remaining work
