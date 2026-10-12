---
format: aep.planning-md/3
id: review-result:w20261012-acceptance-r2
kind: review-result
status: active
title: Wave 2026-10-12-w1 refinement — acceptance critic, round 2
relations:
- reviews: story:canon-0-2-0-pin
- reviews: story:software-change-negative-outcomes
- reviews: story:ml-experiment-protocol
- reviews: story:software-change-profiles
revision: 1
---
needs-revision

story:ml-experiment-protocol — the acceptance never moves `baseline`, so the Outcome's second invalidation rule ("a new `baseline` revision invalidates `candidate.beats_baseline`") can be left out of `protocols/ml-experiment/1.yaml` and `ml_experiment_outcomes_are_earned` still passes; item 7 exercises only the `dataset_snapshot` rule, so an item should start from the evidence of 4 with records carrying `upstream_revisions: {baseline: b1}`, move `baseline` to b2, and expect `candidate.beats_baseline` `unknown` — `.engineering/planning/story/ml-experiment-protocol.md:107` (rule declared at `:54`)

story:ml-experiment-protocol — the Outcome says the new terms are "marked `core`", but item 2 only asks that each id "resolves to exactly one vocabulary entry", which reads the same whether the entries are marked `core` or `software.change`, and no existing test pins the marking of new terms — `.engineering/planning/story/ml-experiment-protocol.md:102` (promise at `:46`)

What I read: 5 of 5 ids (`story:canon-0-2-0-pin`, `story:software-change-negative-outcomes`, `story:ml-experiment-protocol`, `story:software-change-profiles`, `blocker:graph-schema-case-inputs`), each whole body via `aep plan artifact show <id>` in `~/.local/state/worktree/trees/b10x/engineering-protocols/ep-wave-20261012-w1`. I also read `review-result:w20261012-acceptance-r1`, `aep plan artifact kinds` and `aep plan artifact lifecycle story|blocker`. I checked the tree and Canon `dbb04ea`:
- The pin and the full hash in `Cargo.lock` and the three manifests.
- `Taskfile.yml` `check` (clippy, test, `generate --check`).
- `graph::document` (returns `Value` today) and the `protocol.rs` and `graph.rs` test modules.
- `repository.edit` carries no `requires`.
- Canon's reasons for an `any` precondition (`eval/actions.rs`).

The round-1 finding on `story:canon-0-2-0-pin` is fixed. Items 1 and 2 now name `the_page_lists_case_inputs_and_when_an_obligation_applies` and `the_graph_refuses_case_inputs_the_schema_cannot_carry`. Item 2 asks for an error from `graph::document` rather than a schema check on committed graphs, so it can no longer pass without seeing the generated document. I found nothing to change in `story:canon-0-2-0-pin`, `story:software-change-negative-outcomes`, `story:software-change-profiles` or the blocker's "Clears when".

What I could not establish:
- Whether `upstream_revisions` on a `baseline` upstream and the invalidation semantics behave as the finding expects. I read only Canon's reason logic, not the invalidation evaluator. The finding is that the check is absent, not that it would fail.
- Out of my lane, for `plan-critic-scope`: the `blocker:graph-schema-case-inputs` "Clears when" has docs-system extend the schema and this repository pin it. `story:software-change-profiles` then needs `task check` to pass, but its scope lists none of `crates/canon-engineering-docs/src/graph.rs`, `crates/canon-engineering-docs/tests/review_graph_schema.rs:53-120` or `website/package.json`. Those hold the generator's refusal, the schema transcription and the pin that have to change before the graph can carry an input.
- Mild unease, not a finding: `story:canon-0-2-0-pin` and `story:software-change-profiles` put several checks under one `task check` lead-in. Each item is separately named, so I left them.

```findings
- file: .engineering/planning/story/ml-experiment-protocol.md
  line: 107
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance never moves `baseline`, so the Outcome's second invalidation rule (a new `baseline` revision invalidates `candidate.beats_baseline`, declared at line 54) can be omitted from protocols/ml-experiment/1.yaml with ml_experiment_outcomes_are_earned still passing; item 7 exercises only the `dataset_snapshot` rule"
- file: .engineering/planning/story/ml-experiment-protocol.md
  line: 102
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the Outcome promises the new terms are marked `core` (line 46), but item 2 only checks that each id resolves to exactly one vocabulary entry, which reads the same whether the entry is marked `core` or `software.change`"
```
