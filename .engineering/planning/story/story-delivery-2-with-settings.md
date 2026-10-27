---
format: aep.planning-md/3
id: story:story-delivery-2-with-settings
kind: story
status: implemented
title: story.delivery/2 declares tests_first and gate_on as case inputs
summary: red before implementation, gate after handover, as settings
tags:
- priority:high
relations:
- serves: vision:governed-autonomy
- decomposes: epic:gates-and-delivery-for-any-repository
scope:
- confidence: inferred
  path: crates/canon-engineering/tests/story_delivery_2_protocol.rs
- confidence: inferred
  path: fixtures/story-delivery-2
- confidence: inferred
  path: protocols/story-delivery/2.yaml
- confidence: inferred
  path: protocols/vocabulary.yaml
- confidence: inferred
  path: website/docs/protocols/story-delivery
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-27T12:13:59Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-10-27T12:14:00Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-10-27T12:32:24Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
# Story: story-delivery-2-with-settings

## Outcome

The built-in `story.delivery/2` declares its variants as `case.inputs`. A runtime sets them from the repository's
configuration when the case opens:
- `tests_first: [required, optional]`. Under `required`, `implementation.edit` is admissible only after a `tests.run`
  recorded `fail` on a revision `tests.write` produced.
- `gate_on: [after_handover, each_revision]`. Under `after_handover`, `gate.run` is admissible only when `tests.pass`
  and `change.handed_over` hold, so the gate runs once, on the revision that can be handed over.

It also declares the claim `change.complete`: test paths and implementation paths both changed since the base, with
tests and gate green at the same revision. `pr.open` requires it in addition to its `story.delivery/1` preconditions, so the pull request
opens once, on the complete change. This is the protocol half of takt's `story:story-delivery-tests-first`, which
consumes it.

`story.delivery/1` stays byte for byte.

## Context

- A downstream trial of takt (2026-10-27). Red-before-green happened in three of three runs, but nothing required it:
  `implementation.edit` has no precondition (`protocols/story-delivery/1.yaml:132-134`).
- The gate ran three times per case (43.6/61.1/35.0 s, 20.5/52.1/61.7 s, 19.0/23.4/24.4 s), once on the known-red
  test-only revision, because `gate.run` has no precondition (`:160-164`).
- Canon case inputs are the protocol's settings: declared with their values, "fixed when the case opens" (canon
  `crates/canon/src/model/case.rs:41-44`), as `software.change/1` declares `risk` (`protocols/software-change/1.yaml:17-24`).
  Canon has no default for an input, so the runtime supplies every value.
- The order forms (`first`, `before`) exist in protocol/1 (`crates/canon-engineering-docs/src/graph.rs:676`).
- takt's owner decision of 2026-10-19 is that a story case writes its tests first (takt `story:story-delivery-tests-first`).

## Acceptance

Each is a named fixture under `fixtures/story-delivery-2/`, run by `task check`. Canon's `check` finds nothing under
`STATE_BOUND`, or the protocol joins `OVER_BOUND` at its measured count.

1. `red-first-refuses-implementation-first`: under `tests_first: required`, `implementation.edit` is not admissible
   before a recorded `fail`.
2. `a-first-run-pass-is-not-red`: a `tests.write` whose first `tests.run` passed leaves `implementation.edit` not
   admissible.
3. `tests-optional-keeps-the-old-order`: under `tests_first: optional`, both writes are admissible at the base.
4. `gate-after-handover`: under `gate_on: after_handover`, `gate.run` is not admissible at a red revision or before
   handover, and is admissible after both.
5. `gate-each-revision`: under `gate_on: each_revision`, `gate.run` is admissible at every revision, as in `story.delivery/1`.
6. `an-old-gate-pass-does-not-hold`: a `gate_result` at an earlier implementation revision leaves `gate.green` unknown.
7. Every `fixtures/story-delivery/` fixture still gives its result on `story.delivery/1`.
8. `pr-open-needs-change-complete`: with only test paths changed, or only implementation paths, `pr.open` is not
   admissible, even with tests and gate green. With both changed and both green at one revision, it is admissible.

The fixtures are run from `crates/canon-engineering/tests/story_delivery_2_protocol.rs`, beside today's
`story_delivery_protocol.rs`. A new protocol file is embedded by `build.rs` with no Rust edit
(`crates/canon-engineering/src/registry.rs:1-6`).

## Scope

- `protocols/story-delivery/2.yaml` (new)
- `crates/canon-engineering/tests/story_delivery_2_protocol.rs` (new)
- `fixtures/story-delivery-2/`
- `protocols/vocabulary.yaml`
- `website/docs/protocols/story-delivery/` (generated page)

## Delivery

The consumer is takt (`story:story-delivery-tests-first` and `story:takt-ships-default-protocol-settings`). It is
delivered in one workspace with an uncommitted `[patch]` path override in takt's `Cargo.toml` to this working tree.
The end-to-end check is a takt story run with a fake harness on the patched tree: the test is red before the edit, and
the gate runs once. Then this repository releases and takt takes it as a pin-only hop.

## Status

Implemented on `unit/tr-sd2` (c33e64a, review fix 481a7c3).

- `tests.shown_red` holds when a `tests_change` is recorded and `tests.pass` is false at the current revision. A
  baseline run at the seed no longer locks the edit out, and a failing run about another revision is excluded as
  `revision_mismatch` (fixtures `baseline-pass-then-red`, `baseline-fail-then-red`,
  `a-red-run-about-another-revision-does-not-admit-the-edit`).
- Known limit: after `implementation.edit` moves the revision, the next edit waits for a test run at the new revision.
  Canon 0.6.0 has no revision-bound or latest-record order form, so a red run bound to the `tests_change` revision cannot
  persist across edits; `story:story-delivery-2-red-run-binds-revision` (draft) holds it.
- Tests: `story_delivery_2_protocol` 10 passed, `story_delivery_2_protocol_adv1` 5 passed; `fixture_harness` 7,
  `protocol_registry` 1, `vocabulary_yaml` 2, `story_delivery_protocol` 1 passed; docs drift check fresh.
