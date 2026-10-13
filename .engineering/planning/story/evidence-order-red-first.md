---
format: aep.planning-md/3
id: story:evidence-order-red-first
kind: story
status: active
title: software.change/1 reads the bugfix red-first rule from evidence order (Canon 0.3.0)
summary: Pin Canon 0.3.0 and use first and before so a green-then-red bugfix on one revision may edit, and its red run counts after the edit.
relations:
- decomposes: epic:els-first-domain
- depends_on: story:software-change-profiles
- serves: vision:O2
scope:
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/canon-engineering-assertions/Cargo.toml
- confidence: cited
  path: crates/canon-engineering-docs/Cargo.toml
- confidence: cited
  path: crates/canon-engineering-docs/src/graph.rs
- confidence: cited
  path: crates/canon-engineering-docs/src/protocol.rs
- confidence: cited
  path: crates/canon-engineering-docs/tests/review_graph_schema.rs
- confidence: cited
  path: crates/canon-engineering/Cargo.toml
- confidence: cited
  path: crates/canon-engineering/tests/adversary_software_change_profiles.rs
- confidence: cited
  path: crates/canon-engineering/tests/adversary_software_change_protocol.rs
- confidence: cited
  path: crates/canon-engineering/tests/incident_response_protocol.rs
- confidence: inferred
  path: crates/canon-engineering/tests/software_change_profiles.rs
- confidence: cited
  path: crates/canon-engineering/tests/software_change_protocol.rs
- confidence: cited
  path: crates/canon-engineering/tests/support_triage_protocol.rs
- confidence: inferred
  path: fixtures/software-change
- confidence: cited
  path: protocols/software-change/1.yaml
- confidence: cited
  path: protocols/vocabulary.yaml
- confidence: inferred
  path: website/data/protocol-graphs
- confidence: cited
  path: website/docs/concepts/step-order.mdx
- confidence: inferred
  path: website/docs/protocols
- confidence: inferred
  path: website/docs/vocabulary.md
- confidence: inferred
  path: website/package.json
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-13T02:45:31Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-10-13T02:45:31Z", actor: "human:timo", revision: 5}
---
## Outcome

`software.change/1` reads a bugfix's red-first rule from evidence order, on Canon 0.3.0's order
forms, so the stated limitation of 0.5.0 is lifted: a bugfix that runs the suite green on R1,
writes its regression test and runs that test red on R1 may edit the implementation, and the red
run keeps counting after the edit moves the implementation to R2.

## Why

Adversary finding F1 on `story:software-change-profiles`
(`review-result:adversary-w20261013-software-change-profiles-pass-1`): Canon 0.2.0 reads a pass and
a fail about one revision as contradicting, so `regression.reproduced` is UNKNOWN and
`implementation.edit` stays blocked on R1 with no way forward. 0.5.0 kept it as a stated limitation
(`crates/canon-engineering/tests/adversary_software_change_profiles.rs:124`,
`website/docs/concepts/step-order.mdx`). Revision binding adds a second stop: once the first edit
moves the implementation to R2, the red record about R1 no longer matches, so a fix that needs a
second edit needs a new red on R2 (`fixtures/software-change/change-kinds.fixture.yaml`, header).

Canon 0.3.0 (tag peels to 855191a) adds `{first: {kind, result, subject?}}` and
`{before: {first, then, subject?}}`, ordered on `observed_at`, reading the whole evidence history
including records excluded as `revision_mismatch`. Its own acceptance 5 makes `first` FALSE for
green then red of one kind on one revision, so `first` over `test_result` alone does not lift F1:
the suite's green run and the regression test's red run must be told apart by kind. That is the
choice below; it keeps Atlas ADR 0084 Decision 1's path (`regression.reproduced` TRUE on a failing
`test_result` about the current revision) and adds the order path of Decision 2 beside it.

## Design

- Evidence kind `regression_test_result`: the result of running a bugfix's regression test against
  an implementation revision; pass or fail. `tests.run` may produce it.
- Evidence kind `implementation_change`: the record that the implementation was changed, moving it
  to a new revision. `implementation.edit` may produce it.
- Claim `regression.reproduced` becomes `any` of: the existing evidence match (a `test_result` fail
  about the current implementation revision), and
  `{first: {kind: regression_test_result, result: fail, subject: implementation}}`.
- Claim `regression.shown_first`:
  `{before: {first: regression_test_result, then: implementation_change, subject: implementation}}`:
  the regression test ran before the implementation was first changed (parity with AEP
  `test-driven.yaml:35`, `first_seq.test_result < first_seq.diff`).
- Obligation `red_first.required`: applicable when `change_kind` is `bugfix`, at every risk;
  discharged when `regression.shown_first` is TRUE. Each risk's open obligations still strictly
  contain the level below's, for each change kind.
- Both new kinds and the new claim and obligation go into `protocols/vocabulary.yaml`.
- The docs generator renders both forms on the protocol page; the graph writes them as the
  predicate leaves `{"first": {...}}` and `{"before": {...}}` once the pinned docs-system schema
  carries them (`blocker:graph-schema-evidence-order`), and refuses them until then, as it did the
  `input` leaf.

Known limit, from Canon (`story:revision-ordering` there): a red record about any earlier revision
counts, whether or not the current revision descends from it. It goes into `step-order.mdx`, the
CHANGELOG entry and the release notes; no issue is filed for it.

## Scope

- `Cargo.toml` files of the three crates and `Cargo.lock`: Canon tag 0.3.0 (cited: tag lines)
- `crates/canon-engineering-docs/src/graph.rs`, `src/protocol.rs` (cited: E0004 at graph.rs:49,177,266 and protocol.rs:62 under 0.3.0)
- `crates/canon-engineering-docs/tests/review_graph_schema.rs` (cited: schema transcription), `website/package.json` (docs-system pin, once released)
- `crates/canon-engineering/tests/incident_response_protocol.rs:53`, `software_change_protocol.rs:53`, `support_triage_protocol.rs:108`, `adversary_software_change_protocol.rs:127` (cited: E0004)
- `protocols/software-change/1.yaml`, `protocols/vocabulary.yaml` (cited)
- `fixtures/software-change/` new `red-first.fixture.yaml`; `change-kinds.fixture.yaml`, `profiles*.fixture.yaml` expectations (inferred: claim and obligation lists)
- `crates/canon-engineering/tests/adversary_software_change_profiles.rs:124` (cited: the pinned limitation turns into the lifted case), `tests/software_change_profiles.rs` (inferred)
- `website/docs/concepts/step-order.mdx` (cited), generated `website/docs/protocols/software-change/1.mdx`, `website/data/protocol-graphs/software-change-1.json`, `website/docs/vocabulary.md` (generated)
- `CHANGELOG.md` `[Unreleased]`

## Protocol first

First commit after the pin: the protocol YAML, the vocabulary entries and the fixture
`red-first.fixture.yaml` with its expectations; the fixture test and the turned
`adversary_software_change_profiles.rs` case fail on that commit, and the failing run is recorded
under `.engineering/evidence/story/evidence-order-red-first/`. No ESS document covers the protocols
(the ESS specification here is the assertion contracts), so the protocol YAML is the specification.

## Acceptance

`task check` passes with the fixture `red-first` holding, for a `standard` bugfix at R1:

1. suite `test_result` pass at t1, `regression_test_result` fail at t2, both about R1:
   `regression.reproduced` TRUE, `implementation.edit` admissible (F1 lifted);
2. then the implementation at R2 with `implementation_change` at t3: `regression.reproduced` still
   TRUE, `implementation.edit` admissible, `regression.shown_first` TRUE, `red_first.required`
   discharged;
3. `implementation_change` and no `regression_test_result`: `regression.shown_first` FALSE,
   `red_first.required` open, `implementation.edit` blocked;
4. `regression_test_result` pass first, then fail: `first` FALSE, `implementation.edit` blocked
   unless a `test_result` fail about the current revision exists;
5. a feature or refactor: `red_first.required` not applicable.

The generated protocol page names both order forms; the graph carries them as leaves the pinned
docs-system schema validates.

## Source

Adversary finding F1 (review record above); Canon 0.3.0 release notes and its
`story:evidence-order-predicate` (acceptance 1 to 7); Atlas ADR 0084 Decisions 1 and 2; AEP
`principles/development/test-driven.yaml:26,35`.
