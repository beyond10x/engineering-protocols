---
format: aep.planning-md/3
id: approval-record:data-only-catalog
kind: approval-record
status: draft
title: engineering-protocols becomes a data-only catalog (operator, 2026-11-21, option A)
summary: Relayed by the sdc session on 2026-11-21; not heard in this repository's session directly.
tags:
- relayed
relations:
- decides: epic:data-only-catalog
revision: 2
---
# Approval: engineering-protocols becomes a data-only catalog

## Decision

Operator decision of 2026-11-21, answer "A" to the question the `sdc` session put to him, relayed
to this repository by the `sdc` session in a cross-session message on 2026-11-21. This session did
not hear it from the operator. His words as relayed: "engineering protocols was just that, a
collection of potential flows, and then there is an example implementation for the binding, so we
could demonstrate it, but maybe this is now superseded anyway."

Option A as `sdc/docs/engineering-protocols-boundary.md` § Decision states it: engineering-protocols
becomes a data-only catalog; Loom loads protocols from files; observation collection moves into
Loom executors through Substrate; the crates shrink to a check run in CI; a Canon bump then needs
no engineering-protocols release.

## Consequence

- epic:data-only-catalog and its stories are drafted. Nothing is scheduled by this record.
- It replaces Atlas ADR 0094 decision 2 ("a runtime pins a release of `b10x-canon-engineering` and
  reads protocols through its registry") once Atlas records that; AGENTS.md § Boundary still
  states ADR 0094's rule until story:agents-boundary-data-only lands.
- The assertion crates are not deleted by this decision: the owner of their replacement is not
  settled (decision-blocker:assertion-executor-owner).

## Second relay

Later on 2026-11-21 the `sdc` session relayed a second statement the operator made in the `sdc`
session, after its report on this epic: "I'm approving all the drafts you have created according
to my requirements." By the `sdc` session's own reading it does not cover
decision-blocker:assertion-executor-owner or the two older drafts that plan work in the frozen
crates. It was made before the critic round changed five of the seven story bodies. It is a relay
too: this record and every story stay draft.

## What the operator would be asked

Whether the relayed wording is his decision as recorded here, before any story of the epic is
moved out of draft.
