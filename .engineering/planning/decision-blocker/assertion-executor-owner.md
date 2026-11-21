---
format: aep.planning-md/3
id: decision-blocker:assertion-executor-owner
kind: decision-blocker
status: cleared
title: Nobody has decided who owns engineering-gates/1 execution and offline replay after the assertion crates
summary: Loom executors through Substrate, takt, or a tool on Canon's expression crate; the operator decides.
relations:
- blocks: epic:gates-and-delivery-for-any-repository
- blocks: story:gate-program-identity-bound-to-spawn
revision: 5
transitions:
- {from: "open", to: "cleared", at: "2026-11-21T02:28:36Z", actor: "human:timo", revision: 5}
---
## The question

After the assertion crates, who owns the execution of an `engineering-gates/1` document
(`prepare`, `collect`) and the offline replay of retained evidence (`evaluate`)?

Candidates named in `sdc/docs/engineering-protocols-boundary.md` § 5 and § 7 step 4: Loom
executors through Substrate, takt, or a tool on Canon's expression crate. The document records
the owner as unknown and the design as not started.

## What it stops

- epic:gates-and-delivery-for-any-repository and story:gate-program-identity-bound-to-spawn
  (both draft): they add behaviour to `b10x-canon-engineering-assertions` and
  `b10x-assertion-providers`, which story:assertions-frozen-at-0-16-0 freezes. Under the
  decision of 2026-11-21 that work belongs with the next owner of the collectors.
- The later story that deletes the two crates, not drafted.

It does not stop story:assertions-frozen-at-0-16-0: the freeze needs no answer, and its READMEs
name this blocker instead of an owner.

## What clears it

The operator's decision naming the owner, recorded as an approval-record or an Atlas ADR.

## Relayed on 2026-11-21, not an answer here

The `sdc` session reported that it decided the owner in Loom's session, as the orchestrator for
plan and design decisions the operator made it there on 2026-11-21 (Loom recorded
approval-record:sdc-orchestrates-loom-2026-11-21): Loom owns the design for running observation
collection and gates, as executors through Substrate, drafted as an epic behind its
vendored-catalog epic.

The operator did not put this repository's session under that role, and the `sdc` session said
so. This blocker stays open until the decision is on record as "What clears it" asks: an
approval-record the operator confirms in this repository, or an Atlas ADR. When it is, the owner
to name is Loom.

## Answer

Decided on 2026-11-21 by the `sdc` session, which decides for the operator in this session
(approval-record:sdc-orchestrates-engineering-protocols-2026-11-21): the owner is Loom. Loom owns
the design for running observation collection and gates, as executors through Substrate, as a
draft epic behind its epic:engineering-protocols-as-vendored-data.

Consequences here:

- epic:gates-and-delivery-for-any-repository and story:gate-program-identity-bound-to-spawn are
  re-homed, not implemented in the frozen crates: their text goes to the `loom` session as input
  for its observation-collection epic, and they are archived here once Loom has acknowledged it.
- The story that deletes `b10x-canon-engineering-assertions` and `b10x-assertion-providers`
  waits on Loom's executors, not on a decision.
- The READMEs story:assertions-frozen-at-0-16-0 writes name Loom as the next owner.
