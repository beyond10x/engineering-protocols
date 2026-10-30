---
format: aep.planning-md/3
id: approval-record:story-delivery-1-forge-neutral
kind: approval-record
status: approved
title: story.delivery/1 opens its pull request through the run's forge
summary: 'D257, 2026-10-30: takt agrees to the change in story.delivery/1'
relations:
- decides: story:forge-neutral-delivery
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-30T04:24:43Z", actor: "human:timo", revision: 2}
---
# Approval: story.delivery/1 opens its pull request through the run's forge

## Decision

Decided 2026-10-30 as item D257 by the orchestrator, which directs takt and agrees on takt's side
to changing `story.delivery/1`: "D257: take option (a). I direct takt now (handoff D254), and on
takt's side I agree to changing story.delivery/1." The owner's words, relayed by takt: "get rid of
--forge bot - nothing like that must even exist".

`story.delivery/1`'s `pr.open` opens its pull request through the run's forge, as `/2` does since
0.15.1 (approval-record:forge-neutral-delivery).

## Consequence

`/1` stays takt 0.12.0's file byte for byte but for this one agreed change, listed in `AGREED` in
`crates/canon-engineering/tests/takt_protocols_builtin.rs`: the file has the blob id named there,
and undoing the change gives back takt's blob. AGENTS.md § Boundary states the rule. No takt or
loom test checks `/1`'s bytes (takt `main` and loom `main` grepped for the blob id on
2026-10-30). Ships as 0.15.2; takt moves its pin in its next wave.
