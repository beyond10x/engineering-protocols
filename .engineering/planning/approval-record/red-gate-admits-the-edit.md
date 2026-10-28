---
format: aep.planning-md/3
id: approval-record:red-gate-admits-the-edit
kind: approval-record
status: approved
title: A red gate admits the edit under story.delivery/2
summary: decided on a takt run that could not fix a gate failure
relations:
- decides: story:story-delivery-2-edit-after-a-red-gate
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T19:14:27Z", actor: "human:timo", revision: 3}
---
# Approval: a red gate admits the edit under story.delivery/2

## Decision

Decided 2026-10-28 by the orchestrator, under the owner's delegation, on a takt run that could not
fix a gate failure: in `story.delivery/2`, `implementation.edit` is also admissible when
`gate.green` is false at the current revision, that is, after a failed `gate.run`. The fix lands
with a failing fixture first and ships in the next release of this repository.

## Consequence

The rule is amended in place, as the red-run rule of `story.delivery/2` was in 0.11.0: it admits
more and refuses nothing it admitted before, so a consumer pinned to an earlier release keeps its
behaviour and one that moves gains the edit after a red gate. A failed continuous-integration
result is not covered by this decision.
