---
format: aep.planning-md/3
id: approval-record:red-ci-admits-the-edit
kind: approval-record
status: approved
title: A red CI admits the edit under story.delivery/2
summary: decided when the red-gate rule's open continuous-integration gap was scheduled
relations:
- decides: story:story-delivery-2-edit-after-a-red-ci
revision: 4
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T22:55:51Z", actor: "human:timo", revision: 4}
---
# Approval: a red CI admits the edit under story.delivery/2

## Decision

Decided 2026-10-28 by the orchestrator, under the owner's delegation, when the gap that
approval-record:red-gate-admits-the-edit left open was scheduled: in `story.delivery/2`,
`implementation.edit` is also admissible when `ci.green` is false at the current revision, with
the story's tests written and passing there, as it is after a red gate. The fix lands with a
failing fixture first and ships in the next release of this repository.

## Consequence

The rule is amended in place, as the red-gate rule was in 0.12.1: it admits more and refuses
nothing it admitted before, so a consumer pinned to an earlier release keeps its behaviour and one
that moves gains the edit after a red continuous integration. Tests that passed on their first run
can still reach the edit through a red gate or a red continuous integration; closing that needs an
order form over a later record, story:story-delivery-2-red-gate-needs-red-tests.
