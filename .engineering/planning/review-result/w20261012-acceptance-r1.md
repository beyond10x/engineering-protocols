---
format: aep.planning-md/3
id: review-result:w20261012-acceptance-r1
kind: review-result
status: active
title: Wave 2026-10-12-w1 refinement — acceptance critic, round 1
relations:
- reviews: story:canon-0-2-0-pin
- reviews: story:software-change-negative-outcomes
- reviews: story:ml-experiment-protocol
- reviews: story:software-change-profiles
revision: 1
---
needs-revision

story:canon-0-2-0-pin — acceptance item 2 ("validates against the graph schema test") names no test that renders the graph from the `risk` source. The only schema test is `every_committed_protocol_graph_conforms_to_the_pinned_schema_and_its_protocol`, which reads only committed `website/data/protocol-graphs/*.json`. None of those holds an `input` leaf, because no built-in declares a case input, so the check can pass without ever seeing the generated document. The item should name the test that generates the graph from the case-input source and runs the schema check on it, as item 1 should for its "named test" — `.engineering/planning/story/canon-0-2-0-pin.md:63`

What I read: 4 of 4 ids (`story:canon-0-2-0-pin`, `story:software-change-negative-outcomes`, `story:ml-experiment-protocol`, `story:software-change-profiles`), each whole body via `aep plan artifact show <id>` in `~/.local/state/worktree/trees/b10x/engineering-protocols/ep-wave-20261012-w1`, plus `aep plan artifact kinds` and `aep plan artifact lifecycle story`. I also checked the symbols and commands the acceptances name against the tree and Canon `dbb04ea`:
- The pin in `Cargo.lock` and the three manifests.
- The `task check` contents (it includes `generate --check`).
- The existing graph schema test.
- The harness API (`evaluate` returns a `Decision` with reasons, and `record` passes through to Canon, so `upstream_revisions` is accepted).
- Canon's status spellings: `approval-required`, `not_applicable` and `missing-input`.

The other three stories each state a before-and-after, one checkable test with numbered expectations, and observable outputs, and I found nothing to change in them.

What I could not establish:
- Whether the pinned docs-system graph schema permits an `input` leaf. Reading it at `~/beyond10x/docs-system` was denied by the guard, as it is another repository. This bears on item 2 being satisfiable, and the transcription in `review_graph_schema.rs:53` refuses unknown predicate keys today.
- Out of my lane: the ordering of the four stories and the two-way shared surfaces are for `plan-critic-parallel-safety` and `plan-critic-design`. Whether `ml-experiment-protocol`'s Outcome ("marked `core`", README and AGENTS edits) is covered is for `plan-critic-scope`.
- Mild unease, not a finding: `canon-0-2-0-pin` bundles four independent checks (pin, page, graph, no drift) under one `task check` lead-in. It is clear that all must hold, so I left it.

```findings
- file: .engineering/planning/story/canon-0-2-0-pin.md
  line: 63
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "acceptance item 2 'validates against the graph schema test' names no test that renders the graph from the case-input source; the existing schema test (crates/canon-engineering-docs/tests/review_graph_schema.rs:331) reads only committed graphs, none of which holds an input leaf, so it can pass without seeing the generated document, and item 1's 'a named test' likewise names none"
```
