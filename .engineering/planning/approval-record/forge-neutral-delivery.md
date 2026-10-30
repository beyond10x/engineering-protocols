---
format: aep.planning-md/3
id: approval-record:forge-neutral-delivery
kind: approval-record
status: approved
title: Public protocols name no organisation's delivery route
summary: decided by the owner on 2026-10-30, relayed as item D253
relations:
- decides: story:forge-neutral-delivery
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-30T03:29:25Z", actor: "human:timo", revision: 2}
---
# Approval: public protocols name no organisation's delivery route

## Decision

Decided 2026-10-30 by the owner, relayed by the orchestrator as item D253: "bot refers to the
internal bot of b10x org, why would this be an option on a public coding agent". Every public
protocol and vocabulary entry becomes forge-neutral: `pr.open` opens its pull request through the
run's forge.

## Consequence

The wording is amended in place in `story.delivery/2` and the vocabulary, as `/2`'s rules were in
0.11.0 through 0.14.0, and ships as 0.15.1. Neither Canon nor a consumer pins `/2` by digest, and
no consumer asserts on the description text, so a consumer pinned to an earlier release keeps the
old text and one that moves gains the new one with no other change.

`story.delivery/1` is not edited: AGENTS.md § Boundary keeps it takt 0.12.0's file byte for byte
until a new major is agreed with takt, and `takt_protocols_builtin.rs` checks its blob id. Its
wording changes only after takt agrees, in a later release. Consumers that carry their own copy of
`story.delivery` (a later revision kept outside this repository) change their copy themselves.
