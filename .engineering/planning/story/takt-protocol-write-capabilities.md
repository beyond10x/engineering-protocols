---
format: aep.planning-md/3
id: story:takt-protocol-write-capabilities
kind: story
status: draft
title: Name a capability on every write action of the takt protocols
summary: Six write actions of story.delivery/1, self.improvement/1 and loop.improvement/1 name no capability.
relations:
- decomposes: epic:els-first-domain
- serves: vision:O2
revision: 1
---
## Outcome

Every write action of `story.delivery/1`, `self.improvement/1` and `loop.improvement/1` names the capability that authorises it (vision O1, `AGENTS.md` § Serves), or the exception is decided and written down.

## Today

Adversary pass 1 of wave 2026-10-26-w1 (`review-result:adversary-w20261026-takt-protocols-builtin-pass-1`, finding F4) found six write actions with no `requires: capability`: `story.delivery/1` `tests.write`, `implementation.edit`, `pr.open`; `self.improvement/1` `finding.file`; `loop.improvement/1` `loop.change.deliver`, `loop.finding.file`. The files are takt's byte for byte (Atlas ADR 0094 rule 3), so they cannot change here without a new major that takt adopts. `adversary_every_builtin_write_action_names_its_capability` lists the six and fails when one changes.

`tests.write` and `implementation.edit` are the same exception `software.change/1` already carries (capabilities have no path scope in Canon).

## Acceptance

1. The test's exception list holds only the entries `AGENTS.md` § Serves names, and each new major is adopted by takt.
