---
format: aep.planning-md/3
id: story:a-gate-sees-the-change-and-may-not-apply
kind: story
status: implemented
title: A gate can test paths changed since the base and declare when it applies
summary: git.changed, applies_when, not_applicable
tags:
- priority:high
relations:
- serves: vision:governed-autonomy
- decomposes: epic:gates-and-delivery-for-any-repository
scope:
- confidence: inferred
  path: catalog/engineering.yaml
- confidence: inferred
  path: crates/assertion-providers/src/git.rs
- confidence: inferred
  path: crates/assertion-providers/src/lib.rs
- confidence: inferred
  path: crates/canon-engineering-assertions/src/lib.rs
- confidence: inferred
  path: crates/canon-engineering-assertions/tests/gates_result_values.rs
- confidence: inferred
  path: crates/canon-engineering/tests/assertion_replay.rs
- confidence: inferred
  path: ess/domains/gates.yaml
- confidence: inferred
  path: fixtures/assertions/applies-when.fixture.yaml
- confidence: inferred
  path: fixtures/assertions/git-changed.fixture.yaml
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-27T12:14:00Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-27T12:14:00Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-10-27T12:56:35Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
# Story: a-gate-sees-the-change-and-may-not-apply

## Outcome

An `engineering-gates/1` gate can test the change a runtime asks about, and can declare when it applies:
- A catalog operation `git.changed(glob)` is true when a path matching `glob` differs between the provider context's
  `base` and `subject` revisions.
- The context carries `story.id` and `story.tags` when the runtime gives them.
- An optional per-gate `applies_when` (canon-expr) decides applicability. A gate whose `applies_when` is false reports
  `not_applicable`, distinct from `pass`, `fail` and `unknown`.

## Context

- A downstream trial (2026-10-27): every candidate lacked the release-note and documentation changes the repository
  requires for a consumer-visible change. The configured gate passed, because no gate could express "this change must
  touch the changelog".
- The catalog has no change-aware operation (`catalog/engineering.yaml`). The process provider runs a command with a
  cleared environment and no knowledge of a base (`crates/assertion-providers/src/process.rs:163-172`).
- A runtime that reads the gates document from the case's base (takt does) then gets an obligation a proposed change
  cannot weaken by editing it.

## Decision

- Classification ("consumer-visible") is the repository's: a story tag in `story.tags`, or a changed-path glob in
  `applies_when`. Nothing classifies from prose.
- `not_applicable` is written only when `applies_when` evaluated to false. An unevaluated or errored `applies_when` is
  `unknown`.

## Acceptance

Each is a named fixture or test run by `task check`. The ESS gates domain (`ess/domains/gates.yaml`) declares the
operation, the context fields and the result value first.

1. `fixtures/assertions/git-changed.fixture.yaml`: over a two-commit repository, `git.changed("CHANGELOG.md")` is true
   when the subject changed it, false when it did not, and `unknown` when `base` is absent from the context.
2. `fixtures/assertions/applies-when.fixture.yaml`: a gate with `applies_when: 'story.has_tag("consumer-visible")'` is
   `not_applicable` for a story without the tag and evaluates its assertions for one with it.
3. `crates/canon-engineering-assertions/tests/gates_result_values.rs::not_applicable_is_never_pass`: a gates run
   whose only gate is `not_applicable` reports that value per gate, and the run's aggregate is not `pass`.
4. Every existing gates fixture keeps its result.

Fixtures 1 and 2 are loaded by explicit path from `crates/canon-engineering/tests/assertion_replay.rs`, as
`fixtures/assertions/gates.yaml` is today.

## Scope

- `catalog/engineering.yaml`
- `crates/assertion-providers/src/git.rs` (new: the git provider)
- `crates/assertion-providers/src/lib.rs` (registration)
- `crates/canon-engineering-assertions/src/lib.rs` (context fields, `applies_when`, result values)
- `crates/canon-engineering-assertions/tests/gates_result_values.rs`
- `crates/canon-engineering/tests/assertion_replay.rs`
- `ess/domains/gates.yaml`
- `fixtures/assertions/git-changed.fixture.yaml`, `fixtures/assertions/applies-when.fixture.yaml`

## Delivery

The consumer is takt's `story:the-run-shows-each-repository-obligation`. It is delivered in one workspace with an
uncommitted `[patch]` path override in takt's `Cargo.toml` to this working tree. The end-to-end check is takt's
obligations scenario on the patched tree. Then this repository releases and takt takes it as a pin-only hop.

## Status

Implemented on `unit/tr-gates` (e3650f8, review fixes 89ccd56 and 6bd3072), joined into `unit/tr-sd2`.

- `git.changed` measures from the merge base of `base` and `subject`, so edits the base gained after the fork are not
  credited to the change; a subject that is an ancestor of the base is compared tree to tree. Paths are relative to
  the gates root.
- After the review: git runs with lazy fetch and remote protocols off, so repository configuration cannot start a
  program; a non-UTF-8 path no longer aborts the run; globs that can never match are refused; `evaluate` never counts
  a `not_applicable` gate as a failure; the context identity holds the resolved commits.
- Open: a `not_applicable` gate's commands are still collected (skipping them needs a two-phase collection).
- Tests: `gates_result_values` 8 and `_adv1` 5, `git_changed` 5, `_adv1` 6 and `_sec1` 1, `assertion_replay` 8 passed.
