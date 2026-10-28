---
format: aep.planning-md/3
id: story:story-delivery-2-red-gate-needs-red-tests
kind: story
status: draft
title: A red gate admits the edit only after the tests were red
summary: tests that passed on their first run do not reach implementation.edit through a gate failure
relations:
- decomposes: epic:gates-and-delivery-for-any-repository
- informed_by: story:story-delivery-2-edit-after-a-red-gate
revision: 2
---
## Outcome

Under `story.delivery/2` with `tests_first=required`, a red gate admits `implementation.edit` only
when the story's tests were red at some revision after they were written, so tests that passed on
their first run cannot reach the edit through a gate failure.

## Context

The first adversary pass on story:story-delivery-2-edit-after-a-red-gate found that its rule (the
gate red at the current revision, the tests written and passing there) also admits the edit when
the tests were written and passed at once, were never red, and the gate then fails, for example on
their formatting. `a-first-run-pass-is-not-red` refuses the edit in that history, and the red gate
reopens it. Under `gate_on=each_revision` the gate may run at every revision, so the history is
reachable; it was not observed at runtime. The order forms (`first`, `before`) cannot say that a
`test_result` of `fail` was observed after the first `tests_change`, so the rule cannot be written
yet. A decision is needed on whether this gap is accepted or the protocol language grows the form.

## Scope

- `protocols/story-delivery/2.yaml` (`implementation.edit`)
- `crates/canon-engineering/tests/pending/story_delivery_2_red_gate_never_red.rs`, moved back into
  `tests/story_delivery_2_red_gate_adv1.rs`
- Canon's order forms, if the rule needs a new one

## Acceptance

- `a_red_gate_does_not_admit_the_edit_when_the_tests_were_never_red` passes, with
  `a_red_gate_admits_the_edit` and every other `story.delivery/2` fixture unchanged.
