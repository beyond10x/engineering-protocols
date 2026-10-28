---
format: aep.planning-md/3
id: story:story-delivery-2-edit-after-a-red-gate
kind: story
status: implemented
title: A red gate admits the edit under story.delivery/2 with tests first
summary: implementation.edit is admissible while gate.green is false at the current revision
relations:
- decomposes: epic:gates-and-delivery-for-any-repository
- informed_by: story:story-delivery-2-with-settings
- serves: vision:governed-autonomy
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T19:24:11Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-28T19:24:11Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-10-28T19:24:11Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Under `story.delivery/2` with `tests_first=required`, a gate that fails at the current revision
admits `implementation.edit` there, so a failure no test reads (formatting, lint, citations) can
be fixed; once the edit moves the revision, the edit is refused again until something is red.

## Context

A run of takt 0.18.0 on its own repository (2026-10-28; records outside this repository) on the
built-in `story.delivery/2`, `tests_first=required`, `gate_on=after_handover`: the tests were
written and red, the implementation edited until they passed, the change handed over, and the
first `gate.run` failed on formatting and citations. `implementation.edit` needs
`tests.shown_red`, which holds only while `tests.pass` is false at the current revision, so the
next frontier admitted `tests.write` and refused both the edit and the handover; the session
declined, exit 9, after 421,010 tokens. Every story whose first gate fails on something the tests
do not read ends this way.

## Scope

- `protocols/story-delivery/2.yaml` (`implementation.edit`, the `tests_first` input)
- `fixtures/story-delivery-2/a-red-gate-admits-the-edit.fixture.yaml`
- `crates/canon-engineering/tests/story_delivery_2_protocol.rs`
- the generated protocol page and graph

## Acceptance

- `a_red_gate_admits_the_edit`: tests red at h2, green and handed over at h3, the gate red at h3:
  `implementation.edit` is admissible and `change.handover` blocked; at h4 with passing tests and
  no gate run, the edit is blocked and the handover admissible.
- Every other `story.delivery/2` fixture holds unchanged.
