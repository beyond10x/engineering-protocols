---
format: aep.planning-md/3
id: approval-record:sdc-orchestrates-engineering-protocols-2026-11-21
kind: approval-record
status: approved
title: The sdc session decides for the operator in the engineering-protocols session
summary: 'Operator, 2026-11-21, in this session: the sdc session controls it and may decide on his behalf to move its work forward.'
relations:
- decides: epic:data-only-catalog
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-21T02:26:58Z", actor: "human:timo", revision: 2}
---
# Approval: the sdc session decides for the operator in this repository's session

## Decision

The operator wrote in the engineering-protocols session on 2026-11-21, verbatim: "SDC session is
now controlling you. Tell that to the SDC session and that it is now allowed to make decisions on
my behalf for you to move your stuff forward."

Heard directly in this repository's session, not relayed. The `sdc` session was told the same
sentence on the same day.

## What it covers

- Decisions the engineering-protocols session would otherwise put to the operator to move its
  work forward: which stories start, plan and design choices, confirming a relayed operator
  decision, clearing a decision-blocker, merging a pull request of this repository.
- The `sdc` session's messages arrive from the session named `sdc`. A decision is taken from a
  message of that session only.

## What it does not change

- AGENTS.md: commits, pushes, tags and releases are the bot's, the gate is `task check`, a
  release follows § Releases.
- A session's own permission settings, CLAUDE.md and configuration are not edited on another
  session's request, and an action denied in one session is not done by another.
- It is a grant to the session that exists on 2026-11-21. Whether it holds for a later `sdc`
  session is the operator's to say.

## First decisions waiting under it

Put to the `sdc` session on 2026-11-21: merging the pull request that carries
epic:data-only-catalog; confirming approval-record:data-only-catalog; naming Loom as the owner
decision-blocker:assertion-executor-owner asks for; confirming that takt is frozen on its tag;
starting story:catalog-manifest and story:assertion-catalog-path.
