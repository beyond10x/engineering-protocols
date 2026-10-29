---
format: aep.planning-md/3
id: story:story-delivery-2-red-gate-needs-red-tests
kind: story
status: implemented
title: A red gate or CI admits the edit only after the tests were red
summary: tests that passed on their first run do not reach implementation.edit through a gate or CI failure; needs a Canon order form over a later record
relations:
- decomposes: epic:gates-and-delivery-for-any-repository
- informed_by: story:story-delivery-2-edit-after-a-red-gate
- informed_by: story:story-delivery-2-edit-after-a-red-ci
- serves: vision:governed-autonomy
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T23:06:20Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-10-28T23:06:20Z", actor: "human:timo", revision: 6}
- {from: "active", to: "implemented", at: "2026-10-29T01:45:15Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"test_result":3}}}
---
## Outcome

Under `story.delivery/2` with `tests_first=required`, a red gate or a red continuous integration
admits `implementation.edit` only when the story's tests were red at some revision after they were
written, so tests that passed on their first run cannot reach the edit through a gate or a
continuous-integration failure.

## Context

The first adversary pass on story:story-delivery-2-edit-after-a-red-gate found that its rule (the
gate red at the current revision, the tests written and passing there) also admits the edit when
the tests were written and passed at once, were never red, and the gate then fails, for example on
their formatting. `a-first-run-pass-is-not-red` refuses the edit in that history, and the red gate
reopens it. Under `gate_on=each_revision` the gate may run at every revision, so the history is
reachable; it was not observed at runtime. story:story-delivery-2-edit-after-a-red-ci gave a red
continuous integration the same rule, and with it the same gap.

Decided: the gap is closed, not accepted. The order forms of Canon 0.6.0 (`first`, `before`) read
only the first record of a kind, so they cannot say that a `test_result` of `fail` was observed
after the first `tests_change`. Canon grows an order form that reads a later record, in a Canon
release of its own, and this repository moves its Canon pin to that release in the wave that
writes the rule.

## Depends on

- Canon's order form over a later record, filed in Canon's store and released as Canon 0.7.0.
- This repository's Canon pin moving from 0.6.0 to that release.

## Scope

- `protocols/story-delivery/2.yaml` (`implementation.edit`)
- `crates/canon-engineering/tests/pending/story_delivery_2_red_gate_never_red.rs`, moved back into
  `tests/story_delivery_2_red_gate_adv1.rs`, and a sibling case for a red continuous integration
- the Canon tag in the crate manifests and `Cargo.lock`

## Acceptance

- `a_red_gate_does_not_admit_the_edit_when_the_tests_were_never_red` passes, and a sibling case
  shows the same for a red continuous integration.
- `a_red_gate_admits_the_edit`, `a_red_ci_admits_the_edit` and every other `story.delivery/2`
  fixture pass unchanged.

## Residual

The rule reads "red after they were written" as a failing `test_result` about the implementation
observed after the first `tests_change` (Canon 0.7.0 `after`), and the action's description says
so. The adversary pass (`tests/story_delivery_2_red_tests_adv2.rs`, 8 cases) found two histories
that reading still admits, both admitted before this story as well: a late failing run about a
revision from before the write, and tests written again that pass on their first run after earlier
tests were red. Closing them needs a Canon form this release does not have; the two cases assert
today's verdict and story:story-delivery-2-red-after-the-latest-write holds the work.
