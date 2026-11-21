---
format: aep.planning-md/3
id: decision-blocker:assertion-executor-owner
kind: decision-blocker
status: open
title: Nobody has decided who owns engineering-gates/1 execution and offline replay after the assertion crates
summary: Loom executors through Substrate, takt, or a tool on Canon's expression crate; the operator decides.
relations:
- blocks: epic:gates-and-delivery-for-any-repository
- blocks: story:gate-program-identity-bound-to-spawn
revision: 2
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
