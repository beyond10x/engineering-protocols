---
format: aep.planning-md/3
id: story:takt-protocols-builtin
kind: story
status: active
title: Ship story.delivery/1, self.improvement/1 and loop.improvement/1 as built-ins (Atlas ADR 0094)
summary: takt's three protocols move here byte for byte, with vocabulary, fixtures and a canon check over every built-in.
refs:
- provider: atlas
  reference: architecture/adr/0094-engineering-protocols-holds-every-spec-level-protocol.md
relations:
- decomposes: epic:els-first-domain
- serves: vision:O2
scope:
- confidence: cited
  path: crates/canon-engineering-docs
- confidence: cited
  path: crates/canon-engineering/src/vocabulary.rs
- confidence: cited
  path: crates/canon-engineering/tests
- confidence: cited
  path: fixtures/loop-improvement
- confidence: cited
  path: fixtures/self-improvement
- confidence: cited
  path: fixtures/story-delivery
- confidence: cited
  path: protocols/loop-improvement/1.yaml
- confidence: cited
  path: protocols/self-improvement/1.yaml
- confidence: cited
  path: protocols/story-delivery/1.yaml
- confidence: cited
  path: protocols/vocabulary.yaml
- confidence: cited
  path: website/data
- confidence: cited
  path: website/docs
revision: 19
transitions:
- {from: "draft", to: "proposed", at: "2026-10-26T12:35:26Z", actor: "human:timo", revision: 18}
- {from: "proposed", to: "active", at: "2026-10-26T12:35:26Z", actor: "human:timo", revision: 19}
---
## Outcome

Atlas ADR 0094 (`architecture/adr/0094-engineering-protocols-holds-every-spec-level-protocol.md`, accepted 2026-10-19): every spec-level protocol lives here. takt's three protocols become built-ins of `b10x-canon-engineering`, byte for byte, so a runtime reads them through `registry::get` instead of carrying a copy:

| Built-in | File here | Source |
|---|---|---|
| `story.delivery/1` | `protocols/story-delivery/1.yaml` | takt `protocols/story-delivery/1.yaml` at tag 0.12.0 (blob `e21daf63a1`, same on takt `main` 378159a) |
| `self.improvement/1` | `protocols/self-improvement/1.yaml` | takt blob `4c477b5a34` |
| `loop.improvement/1` | `protocols/loop-improvement/1.yaml` | takt blob `6d848471c4` |

takt's `protocols/loop-improvement/metrics.yaml` (`takt-loop-metrics/1`) is runtime data, not a protocol, and stays in takt (ADR 0094 rule 2). takt is not changed here; its re-pin is separate work in takt.

## Decisions taken (coordinator, inside the orchestrator-approved item)

- Vocabulary (ADR 0094 § Consequences): every term the three protocols name is declared in `protocols/vocabulary.yaml` in the category it is used in. Terms only meaningful for Git, pull requests or code are marked `software.change`; terms only meaningful in an agent runtime's self-improvement turn get a new marking `self.improvement`, in a loop-of-loops turn `loop.improvement`; everything else `core`. A term already declared keeps its one entry; its meaning may be widened only so that it stays true for every protocol using it.
- ADR 0094 rule 5: a test runs Canon's `check` on every built-in and requires zero findings under `STATE_BOUND`. A built-in that does not hold today is listed by name with the test asserting its current measurement, so the test goes red when it changes. ADR 0094 records `self.improvement/1` at 5,971,968 states (Canon 0.6.0, 2026-10-19), over the bound of 65,536; it is moved byte for byte anyway and the gap is filed as its own story.
- Every rule of the three protocols is exercised by a fixture under `fixtures/<name>/` through the existing harness, as for every other built-in.

## Protocol first

The protocols are new built-ins here. The first commit lands the three YAML files, their fixtures and the named tests; the vocabulary-coverage test fails on it (the terms are not yet declared) and that failing run is recorded before the vocabulary commit.

## Acceptance

1. `registry::list` names `story-delivery@1`, `self-improvement@1` and `loop-improvement@1`, and each file is byte-identical to takt tag 0.12.0's (`tests/protocol_registry.rs` plus a byte comparison against the recorded blob ids).
2. A test fails when any term a built-in other than `support.triage/1` names (artifact kinds, evidence kinds, claim, action, obligation and outcome ids, decision names, case inputs, input values, independence dimensions) is missing from the vocabulary or declared in another category; it passes on the merged tree.
3. A test runs `b10x_canon::check::check` on every built-in: zero findings and `states <= STATE_BOUND`, except the named list, each entry asserting its current measurement.
4. Fixtures under `fixtures/story-delivery/`, `fixtures/self-improvement/`, `fixtures/loop-improvement/` reach every outcome of each protocol and show its stale-revision or independence case (`UNKNOWN`, not `FALSE`) where the protocol declares one.
5. `canon-engineering-docs generate --check` passes with the three protocols' generated pages, graphs, status rows and the vocabulary page committed.
