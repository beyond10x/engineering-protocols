---
format: aep.planning-md/3
id: story:builtin-state-bound
kind: story
status: draft
title: Every built-in checks under Canon's state bound (ADR 0094 rule 5)
summary: self.improvement/1, ml.experiment/1 and software.change/1 are over STATE_BOUND and pinned in OVER_BOUND.
relations:
- decomposes: epic:els-first-domain
- serves: vision:O2
revision: 1
---
## Outcome

Atlas ADR 0094 rule 5 holds for every built-in: Canon's `check` covers each one under `STATE_BOUND` (65,536) with no findings.

## Today

Measured by `crates/canon-engineering/tests` on wave 2026-10-26-w1 (implementor report, Canon 0.6.0), three built-ins are over the bound and are pinned in the test's `OVER_BOUND` list at their current measurement:

| built-in | states |
|---|---|
| `self.improvement/1` | 5,971,968 |
| `ml.experiment/1` | 4,398,046,511,104 |
| `software.change/1` | 10,192,158,720 |

`self.improvement/1` came from takt byte for byte (ADR 0094 rule 3), so a split is a new major and a takt change. `ml.experiment/1` and `software.change/1` were already over the bound before the ADR. The fix is a split of each protocol or a Canon change that reduces independent producers (ADR 0094 § Consequences); which one is not decided here.

## Acceptance

1. `OVER_BOUND` is empty and the canon-check test passes for every built-in.
