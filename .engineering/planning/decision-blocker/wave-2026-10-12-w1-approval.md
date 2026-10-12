---
format: aep.planning-md/3
id: decision-blocker:wave-2026-10-12-w1-approval
kind: decision-blocker
status: cleared
title: Wave 2026-10-12-w1 (Canon 0.2.0, negative outcomes, ml.experiment/1) awaits approval
relations:
- blocks: story:canon-0-2-0-pin
- blocks: story:software-change-negative-outcomes
- blocks: story:ml-experiment-protocol
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-12T21:36:06Z", actor: "human:timo", revision: 3}
---
## Question

Approve wave 2026-10-12-w1, and with which units?

## Options

| option | units, in order, one integration branch `wave/2026-10-12-w1`, one pull request | cost |
|---|---|---|
| A | `story:canon-0-2-0-pin`, `story:software-change-negative-outcomes`, `story:ml-experiment-protocol` | three serial units, each an implementor in its own tree branched from the wave branch after the previous merge, three builds one at a time; one release (0.4.0) carrying Canon 0.2.0, the four negative outcomes and `ml.experiment/1` |
| B | `story:canon-0-2-0-pin`, `story:software-change-negative-outcomes` | two serial units and builds; `ml.experiment/1` waits for the next wave and release |

`story:software-change-profiles` is not in either: `blocker:graph-schema-case-inputs` holds it until docs-system extends `b10x-protocol-graph/1`.

The units are serial, not parallel: `aep plan artifact waves` puts each in its own wave because they share `protocols/vocabulary.yaml`, the harness and `generate.rs`, and their `depends_on` edges order them.

Recommendation: A. None of the three is blocked, they ship together as one release, and B only defers the third unit to another wave cycle.

## Clears when

The answer is written into this body and the blocker moves to `cleared`.

## Answer

Option A, approved 2026-10-12: `story:canon-0-2-0-pin`, `story:software-change-negative-outcomes`, `story:ml-experiment-protocol`, serially on `wave/2026-10-12-w1`, one pull request, release 0.4.0 after it. The answer covers the wave's commits, push, pull request, merge once CI is green, tag and release through this repository's process.
