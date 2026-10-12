---
format: aep.planning-md/3
id: review-result:w20261012-parallel-safety-r1
kind: review-result
status: active
title: Wave 2026-10-12-w1 refinement — parallel-safety critic, round 1
relations:
- reviews: story:canon-0-2-0-pin
- reviews: story:software-change-negative-outcomes
- reviews: story:ml-experiment-protocol
- reviews: story:software-change-profiles
revision: 1
---
approve

I found no pair in the four-story set that can run concurrently and collide. Every shared surface sits on a path that `depends_on` already orders in the intended sequence, `story:canon-0-2-0-pin`, then `story:software-change-negative-outcomes`, then `story:ml-experiment-protocol`, then `story:software-change-profiles`.

**Collision pairs.** The `waves` output reports these overlaps, and each is covered:

| Pair | Shared surface | Covered by |
|---|---|---|
| pin and negative-outcomes | `crates/canon-engineering/tests/software_change_protocol.rs`, `adversary_software_change_protocol.rs` (cited, both bodies) | edge `story:software-change-negative-outcomes` → `story:canon-0-2-0-pin` (`depends_on`) |
| pin and profiles | the same two test files | edge `story:software-change-profiles` → `story:canon-0-2-0-pin` |
| negative-outcomes and ml | `protocols/vocabulary.yaml`, `generate.rs:564`, the generated site pages | edge `story:ml-experiment-protocol` → `story:software-change-negative-outcomes`. The ml body names the shared terms (`superseded`, `explicitly_superseded`), the harness keys and the decision category. |
| ml and profiles | `AGENTS.md`, `generate.rs`, `protocols/vocabulary.yaml`, the generated pages | edge `story:software-change-profiles` → `story:ml-experiment-protocol`. The profiles body says it shares these with ml and runs after it. |
| negative-outcomes and profiles | `protocols/software-change/1.yaml`, `fixtures/software-change/`, `support/mod.rs`, the reader, `vocabulary_yaml.rs`, the site | direct edge. Both bodies have a "Shared surface" section naming the chain. |
| `CHANGELOG.md` `[Unreleased]` (inferred), all four | all four | the transitive chain of edges |

**Reader beside producer.** I traced six reads to a producer in the set, and a `depends_on` edge records all six:
- profiles reads the Canon 0.2.0 pin and case-input rendering from the pin story.
- profiles reads the `declined`, `superseded` and `abandoned` terms and the decision category from negative-outcomes.
- ml reads `add_decisions`, `expect.outcomes`, the decision category and the two shared terms from negative-outcomes.
- negative-outcomes reads the Canon 0.2.0 pin.

**Hidden surfaces I checked in the tree.**
- `vocabulary_yaml.rs` pins only the first 35 terms and says later stories append. ml's append-only edits therefore touch no file that other stories' bodies leave unnamed.
- The profiles story removes `repository.edit`, and it names the three test files that list it (`vocabulary.rs:43`, `vocabulary_yaml.rs:184`, `adversary_vocabulary.rs:81`).
- `generate.rs:564` (term count 36) is edited by all three of negative-outcomes, ml and profiles. The edges order them, and each body names it.
- `adversary2_software_change_render.rs` regenerates the shipped `software-change/1.yaml` through the docs generator. It reads the shipped file, so no story that edits the protocol needs to edit this test.

**What I read.** Four artifacts, with `aep plan artifact show` on each, plus `aep plan artifact waves --kind story --status draft`, `graph` and `relations`. In the wave tree I used `ls`, `git grep` and file reads of `vocabulary_yaml.rs`, `status.rs`, the docs tests and `generate.rs`. Surfaces: 4 items placed from cited surfaces (each also lists some inferred lines, none of which carries a collision), 0 placed only by inference, 0 unplaceable. None of the four appears in the `unassessed:` list. Six acceptance reads traced to a producer in the set, six recorded by `depends_on`.

**What I could not establish.**
- The E0004 site list in the pin body (eight matches) was not re-run. That would need `cargo clippy`, which writes `target/`, and I am read-only. I rely on its cited `file:line` references.
- Out of my lane, and not setting the verdict:
  - The pin and negative-outcomes edge, and the pin and profiles edge, record the order, but neither body states the shared test files as the reason.
  - `story:ml-experiment-protocol` lists `website/docs/status.mdx` as generated. `status.rs` suggests that page is a static import of `status.json`. I did not verify this and left it for the scope or acceptance critics.

```findings
[]
```
