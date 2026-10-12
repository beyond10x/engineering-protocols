---
format: aep.planning-md/3
id: review-result:w20261012-parallel-safety-r2
kind: review-result
status: active
title: Wave 2026-10-12-w1 refinement — parallel-safety critic, round 2
relations:
- reviews: story:canon-0-2-0-pin
- reviews: story:software-change-negative-outcomes
- reviews: story:ml-experiment-protocol
- reviews: story:software-change-profiles
revision: 1
---
needs-revision

story:software-change-profiles — it must lift the `b10x-protocol-graph/1` refusal that `story:canon-0-2-0-pin` writes into `crates/canon-engineering-docs/src/graph.rs` (cited, pin body) once `blocker:graph-schema-case-inputs` clears, but its Scope names no `graph.rs`, no `review_graph_schema.rs` transcription and no `website/package.json` and `website/package-lock.json` bump, so the pair's real collision on `graph.rs` is unnamed. Either add those files to its Scope with a sentence saying it replaces the pin story's `the_graph_refuses_case_inputs_the_schema_cannot_carry` test, or split the graph change into its own item that sits between the blocker and this story. — .engineering/planning/story/software-change-profiles.md:104-120 (Scope lists `generate.rs` and the generated graph JSON but no graph renderer or schema pin), .engineering/planning/story/canon-0-2-0-pin.md:42 and :67 (the refusal and its test), .engineering/planning/blocker/graph-schema-case-inputs.md:21 ("`website/package.json` here pins that commit")

The collision has a surface behind it and is not only a wording gap:

- **Why `graph.rs` must change:** `protocols/software-change/1.yaml` gains `applicable_when` and `input` predicates in this story. `graph::document` (`crates/canon-engineering-docs/src/graph.rs:163`) then has to emit them for `generate --check` and `task check` to pass, and that is the profiles acceptance.
- **The stale transcription:** `crates/canon-engineering-docs/tests/review_graph_schema.rs:53-120` transcribes the closed schema and must change with it. That header already cites `929f965`, while `website/package.json:18` pins `9d35e63`.
- **No owner for the bump:** the blocker clears only when `website/package.json` pins the new docs-system commit. No story in the set owns that bump.

The `depends_on` edge from profiles to the pin story does order the two. The profiles body does not admit the file, so `aep plan artifact waves` cannot show this overlap.

**What I read:** five artifacts. I ran `aep plan artifact show` on each of the four stories and on the blocker, plus `waves --kind story --status draft`, `graph`, `relations` and `show review-result:w20261012-parallel-safety-r1`. In the wave tree I read the planning bodies, `graph.rs`, `review_graph_schema.rs` and `website/package.json`, and used `git grep`. Surfaces: 4 of 4 stories placed from cited paths (each also has some inferred lines, none carrying a collision claim), 0 inferred only, 0 unplaceable. The blocker names its surfaces but is not workable. I traced seven acceptance reads to a producer in the set, and a `depends_on` edge records all seven:

- negative-outcomes reads the pin.
- profiles reads the pin's rendering.
- profiles reads the decision category and terms from negative-outcomes.
- ml reads `add_decisions`, `expect.outcomes` and the decision category from negative-outcomes.
- ml reads the shared terms `superseded` and `explicitly_superseded` from negative-outcomes.
- profiles' `set_inputs:` sits beside the negative-outcomes edits in `support/mod.rs`.
- profiles' term count at `generate.rs:564` follows ml's.

An eighth read, the schema extension, comes from the blocker, and a `blocks` edge records it. The other collision pairs from `waves` (the shared `CHANGELOG.md`, `generate.rs`, `vocabulary.yaml`, the test files and generated pages) are all ordered by the edges pin, then negative-outcomes, then ml, then profiles. The round-1 finding set was empty, and this finding is new in round 2.

**What I could not establish:**
- I did not re-run `cargo clippy` for the E0004 site list, because it writes `target/` and I am read-only.
- Whether `website/docs/status.mdx` is generated or a static import of `status.json` is unverified and belongs to the scope or acceptance critics.
- Out of my lane, not setting the verdict:
  - `story:software-change-negative-outcomes` line 3 of its Order section still says profiles waits on `blocker:canon-case-inputs`, which is cleared. It omits the new graph-schema blocker and the pin edge.
  - The `review_graph_schema.rs` header cites `929f965` while `website/package.json:18` pins `9d35e63`. This is stale text in a file I could not rule on.

```findings
- file: .engineering/planning/story/software-change-profiles.md
  line: 104
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "story:software-change-profiles must lift the graph refusal story:canon-0-2-0-pin writes into crates/canon-engineering-docs/src/graph.rs (cited, pin body) once blocker:graph-schema-case-inputs clears, yet its Scope names no graph.rs, no review_graph_schema.rs transcription and no website/package.json or package-lock.json pin bump, so the collision on graph.rs is unnamed; either add those files to its Scope and say it replaces the pin story's the_graph_refuses_case_inputs_the_schema_cannot_carry test, or split the graph change into its own item ordered between the blocker and this story"
```
