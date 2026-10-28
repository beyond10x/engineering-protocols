---
format: aep.planning-md/3
id: story:story-delivery-2-edit-after-a-red-ci
kind: story
status: implemented
title: A red CI admits the edit under story.delivery/2 with tests first
summary: implementation.edit is admissible while ci.green is false at the current revision and the tests are written and pass
relations:
- decomposes: epic:gates-and-delivery-for-any-repository
- informed_by: story:story-delivery-2-edit-after-a-red-gate
- serves: vision:governed-autonomy
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T22:40:25Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-28T22:40:25Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-10-28T23:05:53Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":3}}}
---
## Outcome

Under `story.delivery/2` with `tests_first=required`, continuous integration that fails at the
current revision admits `implementation.edit` there while the story's tests are written and pass,
so a failure only continuous integration sees (another platform, a check the local gate does not
run) can be fixed; once the edit moves the revision, the edit is refused again until something is
red there.

## Context

story:story-delivery-2-edit-after-a-red-gate let a red gate admit the edit and left a failed
continuous-integration result out (approval-record:red-gate-admits-the-edit). After `ci.read`
records a `ci_result` of `fail`, the tests pass and the gate is green at that revision, so
`tests.shown_red` is false and the gate branch does not hold: `implementation.edit` stays blocked,
and a case whose failure no test reads can only decline. A red continuous-integration result
follows the same rule as a red gate.

## Protocol first

The failing fixture `a-red-ci-admits-the-edit` and the test `a_red_ci_admits_the_edit` land first
and fail on the protocol as released in 0.12.1; the rule lands after them.

## Scope

- `protocols/story-delivery/2.yaml` (`implementation.edit`, the `tests_first` input)
- `fixtures/story-delivery-2/a-red-ci-admits-the-edit.fixture.yaml`
- `crates/canon-engineering/tests/story_delivery_2_protocol.rs`
- the generated protocol page and graph

## Acceptance

- `a_red_ci_admits_the_edit` passes: with the tests written and passing, the gate green and the
  pull request open at one revision, a `ci_result` of `fail` there makes `implementation.edit`
  admissible, and at the next revision it is blocked until something is red.
- Every other `story.delivery/2` fixture passes unchanged.
- Tests that passed on their first run can still reach the edit through a red continuous
  integration, as through a red gate; story:story-delivery-2-red-gate-needs-red-tests closes that
  gap for both.
