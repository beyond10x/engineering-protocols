---
format: aep.planning-md/3
id: story:software-change-profiles
kind: story
status: proposed
title: Define risk profiles and bugfix/refactor change-kind profiles for software.change/1
summary: Risk profiles change which obligations apply; the bugfix profile adds the red-first rule, refactor and feature omit it; repository.edit splits into tests.write and implementation.edit.
refs:
- provider: canon
  reference: taskboard:C-001
- provider: canon
  reference: taskboard:C-005
- provider: taskboard
  reference: E-003
relations:
- decomposes: epic:els-first-domain
- depends_on: story:software-change-protocol
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:vocabulary-yaml-source
- depends_on: story:software-change-negative-outcomes
- depends_on: story:canon-0-2-0-pin
- depends_on: story:ml-experiment-protocol
scope:
- confidence: cited
  path: AGENTS.md
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: crates/canon-engineering-docs/src/generate.rs
- confidence: cited
  path: crates/canon-engineering-docs/src/graph.rs
- confidence: cited
  path: crates/canon-engineering-docs/tests/review_graph_schema.rs
- confidence: cited
  path: crates/canon-engineering/src/vocabulary.rs
- confidence: cited
  path: crates/canon-engineering/tests/adversary_software_change_protocol.rs
- confidence: cited
  path: crates/canon-engineering/tests/adversary_vocabulary.rs
- confidence: cited
  path: crates/canon-engineering/tests/software_change_profiles.rs
- confidence: cited
  path: crates/canon-engineering/tests/software_change_protocol.rs
- confidence: cited
  path: crates/canon-engineering/tests/support/mod.rs
- confidence: cited
  path: crates/canon-engineering/tests/vocabulary.rs
- confidence: cited
  path: crates/canon-engineering/tests/vocabulary_yaml.rs
- confidence: cited
  path: docs/examples/software-change.md
- confidence: cited
  path: fixtures/software-change/
- confidence: cited
  path: protocols/software-change/1.yaml
- confidence: cited
  path: protocols/vocabulary.yaml
- confidence: cited
  path: website/data/protocol-graphs/software-change-1.json
- confidence: cited
  path: website/data/status.json
- confidence: cited
  path: website/docs/concepts/profiles-and-generated-protocols.mdx
- confidence: cited
  path: website/docs/concepts/step-order.mdx
- confidence: cited
  path: website/docs/protocols/index.md
- confidence: cited
  path: website/docs/protocols/software-change/1.mdx
- confidence: cited
  path: website/docs/vocabulary.md
- confidence: inferred
  path: website/examples/protocols/software-change-bugfix/1.yaml
- confidence: cited
  path: website/package-lock.json
- confidence: cited
  path: website/package.json
revision: 33
transitions:
- {from: "draft", to: "proposed", at: "2026-10-12T21:33:46Z", actor: "human:timo", revision: 32, decided_on: {"recorded":{"review_outcome":4}}}
---
## Outcome

Four risk profiles (trivial, standard, elevated, critical) change the proof burden of `software.change/1` without creating a second lifecycle. Each profile keeps the same claims, actions and outcomes and changes only which obligations apply. Authority does not vary by profile: Canon 0.2.0 cannot make a capability requirement depend on an input (canon `crates/canon/src/model/mod.rs:176`, `:186-190`), and that is a later change.

This story edits `protocols/software-change/1.yaml` to add the `risk` case input and four obligations whose `applicable_when` tests it, each discharged by a claim the protocol already has:

| obligation | discharged when | applies at risk |
|---|---|---|
| `tests.required` | `tests.pass` | trivial, standard, elevated, critical |
| `review.required` | `implementation.reviewed` | standard, elevated, critical |
| `provenance.required` | `release.proven` | elevated, critical |
| `health.required` | `deployment.healthy` | critical |

An obligation a profile does not apply is reported `not_applicable`. The obligations report the proof burden; they do not gate `accepted`, whose `requires` stays as it is (`protocols/software-change/1.yaml:116-120`). Making `accepted` wait on the applicable obligations is a later change.

It adds the terms it introduces to `protocols/vocabulary.yaml`: the case input `risk`, the profiles `trivial`, `standard`, `elevated` and `critical`, and the four obligations. The typed reader `crates/canon-engineering/src/vocabulary.rs` has no category for a case input or an input value, so this story adds them to the reader and adds no term to the Rust (Atlas ADR 0077 point 3, draft: the vocabulary is data).

Design basis: `docs/design/engineering-lifecycle-specification-design.md` § 9 declares risk as a case input, and § 10 derives obligations from it. § 27 creates a distinct lifecycle only when the semantic graph differs. A profile is therefore the value of that case input, not an imported protocol. The TASKBOARD says `standard` and design § 9 says `normal`; use `standard`.

Pressure test: the mechanism (case input → applicable obligations) names no Git or pull-request concept, so `incident.response/1` could later key severity on it. Adding incident severity is not part of this story.

## ADR 0084 addition: change-kind profiles

Atlas ADR 0084 (accepted 2026-10-04, option C) § Also needed: AEP's `applies_when: task.kind any_of [feature, bugfix]` (aep `principles/development/test-driven.yaml:18-20`) becomes a `software.change/1` profile. Same mechanism as the risk profiles: a second case input, `change_kind`, whose value selects the rule; no second lifecycle.

- **`repository.edit` splits** into `tests.write` (no precondition) and `implementation.edit`, in `protocols/vocabulary.yaml` and in `protocols/software-change/1.yaml` (ADR 0084 § Decision 1). `repository.edit` leaves the vocabulary, and `AGENTS.md` § Serves names `implementation.edit` and `tests.write` where it names `repository.edit` today.
- **`bugfix`** adds the red-first rule: `implementation.edit` has the precondition `any: [{input: change_kind, in: [feature, refactor]}, {claim: regression.reproduced}]`, and `regression.reproduced` is TRUE on a `test_result` whose result is `fail`, with no subject, so red recorded on one implementation revision still counts after the edit produces the next. Once Canon has the evidence-order predicate (canon `story:evidence-order-predicate`), the rule also requires the first `test_result` to precede the first implementation change; that tightening is a later change.
- **`refactor`** omits the rule.
- **`feature`** is a value because Canon refuses a case missing a declared input (canon `crates/canon/src/eval/case.rs:124-130`), so without it a feature change could not open a case. ADR 0084 names only `bugfix` and `refactor`, so in /1 `feature` carries no red-first rule; giving it AEP's rule is a later tightening.
- Telling a test edit from an implementation edit needs a path scope on capabilities (ADR 0084 § Decision 1, ADR 0083 § Open; canon `story:capability-scope`). Until then the split is by action id only.

## Scope

Read 2026-10-12 against Canon 0.2.0 (`dbb04ea`). Each line is cited (read in the tree) or inferred.

- `protocols/software-change/1.yaml`: case inputs, obligations, the split of `repository.edit` (`:92-94`), `regression.reproduced` (cited)
- `fixtures/software-change/`: new `profiles` and `change-kinds`; `chg-1842.fixture.yaml:6-16` gains `inputs:` (`risk: standard`, `change_kind: refactor`) and its seven action maps replace `repository.edit` (cited)
- `protocols/vocabulary.yaml:124-127` and the new terms (cited)
- `crates/canon-engineering/src/vocabulary.rs:36-65`: the case-input and input-value categories (cited)
- `crates/canon-engineering/tests/software_change_profiles.rs` (new) (cited)
- `crates/canon-engineering/tests/support/mod.rs`: a `set_inputs:` state key beside `set_revisions` (`:486-509`), and `not_applicable` in the format's obligation statuses (`:61-62`) (cited)
- `crates/canon-engineering/tests/software_change_protocol.rs:113`, `:180-198`, `:191` (`ir.obligations.is_empty()`), `:247` (cited)
- `crates/canon-engineering/tests/adversary_software_change_protocol.rs:54-61` (a case with no `inputs`), `:140` (cited)
- `crates/canon-engineering/tests/vocabulary.rs:43`, `vocabulary_yaml.rs:184`, `adversary_vocabulary.rs:81` (each lists `repository.edit`) (cited)
- `crates/canon-engineering-docs/src/generate.rs:118-126` (`category_title`) and the term count (cited)
- generated: `website/docs/protocols/software-change/1.mdx`, `website/data/protocol-graphs/software-change-1.json`, `website/docs/vocabulary.md`, `website/data/status.json`, `website/docs/protocols/index.md` (cited)
- hand-written: `docs/examples/software-change.md:17`, `AGENTS.md:12-14`, `website/docs/concepts/profiles-and-generated-protocols.mdx:17,26-31`, `website/docs/concepts/step-order.mdx:54-58,76-80` (both say Canon has no profile construct) (cited)
- `website/examples/protocols/software-change-bugfix/1.yaml` and `website/data/example-graphs/software-change-bugfix-1.json`: the preview the built-in now supersedes, kept or retired with the concept pages (inferred)
- `CHANGELOG.md` `[Unreleased]` (inferred)
- `crates/canon-engineering-docs/src/graph.rs`: `graph::document` emits the `input` leaf and an obligation's `applicable_when` in the shape docs-system gives `b10x-protocol-graph/1`, the `canon-ir/1` form `{"input": {"id": <id>, "in": [<values>]}}` (agreed 2026-10-12, not the flat source form), replacing the refusal `story:canon-0-2-0-pin` adds and its test `the_graph_refuses_case_inputs_the_schema_cannot_carry`; `crates/canon-engineering-docs/tests/review_graph_schema.rs:53-120`, the schema transcription, follows the extended schema; `website/package.json:18` and `website/package-lock.json` pin the docs-system commit that extends it (cited: pin story, `blocker:graph-schema-case-inputs`)

## Shared surface

This story is the third link of the `software.change/1` chain: `story:software-change-protocol` → `story:software-change-negative-outcomes` → `story:software-change-profiles` → `story:ess-conformance-evidence` → `story:security-independence-rules` → `story:stale-evidence-fixtures`. Every story on it edits `protocols/software-change/1.yaml` and `fixtures/software-change/`, and Canon has no composition that would let a profile live in its own file. It also shares `protocols/vocabulary.yaml`, the reader, `generate.rs`'s term count and the generated vocabulary page with `story:ml-experiment-protocol`, and runs after it.

## Protocol first

Atlas ADR 0080 (draft). The first commit changes the following, and nothing else:

- `protocols/software-change/1.yaml`, adding the `risk` and `change_kind` case inputs, the four obligations, the split of `repository.edit` into `tests.write` and `implementation.edit`, the claim `regression.reproduced` and the precondition on `implementation.edit`;
- the fixtures `profiles` and `change-kinds`, and `chg-1842`'s `inputs:` and action maps, with the expectations in § Acceptance;
- the `set_inputs:` key in `crates/canon-engineering/tests/support/mod.rs`, and `inputs` in the case `adversary_software_change_protocol.rs:54-61` builds;
- `crates/canon-engineering/tests/software_change_profiles.rs`.

On that commit `profiles_nest_open_obligations` fails at items 4 and 8: the new terms do not yet resolve in the vocabulary, `repository.edit` still does, and the reader has no category for case inputs or input values. The implementation commit adds and removes the terms, adds the categories and their arms in the reader, `generate.rs` and the vocabulary tests, updates the expected shape lists in `software_change_protocol.rs`, the example, `AGENTS.md` and the concept pages, and regenerates the site, without changing the protocol YAML or the fixtures.

## Canon capability

Canon 0.2.0 (tag `0.2.0`, `dbb04ea`; `blocker:canon-case-inputs`, cleared) ships what this story needs:

- `case: inputs:` with closed value lists (canon `crates/canon/src/model/mod.rs:78-110`), and a case refused when it leaves out a declared input, names an undeclared one or gives an undeclared value (`eval/case.rs:87-133`);
- the `input` predicate leaf `{input: <id>, in: [...]}`, usable in claims, preconditions, outcomes and obligation applicability (`model/predicate.rs:67`, `:91-98`); a precondition blocked partly by an input names it as `{"input": <id>, "value": <value>}` (`eval/actions.rs:217-221`);
- `Obligation.applicable_when`: FALSE makes the obligation `not_applicable`; applicable, it is `discharged` when `discharged_when` is TRUE and `open` otherwise (`model/mod.rs:157-164`, `eval/obligations.rs:6-18`, `:63-80`);
- one `canon-ir/1` serves every profile: inputs are case data (`ir/mod.rs:71`, `:111-117`).

Still missing in 0.2.0: a capability requirement that depends on an input, and obligations that gate an outcome without the outcome testing them itself.

## Domain relations

- ELS protocol → Canon protocol model: many-to-one. Canon owns the language and an ELS protocol is a document in it; an ELS protocol cannot exist before the Canon capability it uses. Inferable: from `crates/canon-engineering/Cargo.toml:9`, the `b10x-canon` dependency, and `crates/canon-engineering/src/lib.rs:5`, which types protocol ids with `b10x_canon::ProtocolId`. No ess/1 document declares it, because ELS opts out of ESS for protocol semantics (`AGENTS.md` § ESS).

## Acceptance

The test `profiles_nest_open_obligations` in `crates/canon-engineering/tests/software_change_profiles.rs` passes under `task check`. Its fixture is `profiles` in `fixtures/software-change/`: implementation revision R1, `change_kind` `refactor`, no evidence, no authority decision, and one fixed evaluation instant. The fixture is evaluated four times, with `risk` set to `trivial`, `standard`, `elevated` and `critical` through `set_inputs:`. It expects:

1. Starting from that fixture, all four evaluations run against one `canon-ir/1` document compiled once from `protocols/software-change/1.yaml`, so no profile has a protocol of its own.
2. Starting from that fixture, the open-obligation sets are exactly {`tests.required`} at trivial, {`tests.required`, `review.required`} at standard, {`tests.required`, `review.required`, `provenance.required`} at elevated and all four at critical, so they are pairwise different.
3. Starting from that fixture, each set strictly contains the set of the profile below it (trivial ⊂ standard ⊂ elevated ⊂ critical), and every obligation outside a profile's set is reported `not_applicable`, not `open`.
4. Starting from `protocols/vocabulary.yaml` as this story leaves it, read through `crates/canon-engineering/src/vocabulary.rs`, each of `risk`, `trivial`, `standard`, `elevated`, `critical`, `tests.required`, `review.required`, `provenance.required` and `health.required` resolves to exactly one vocabulary entry.
5. Starting from the fixture `change-kinds` (implementation revision R1, `risk` `standard`, no authority decision, one fixed evaluation instant) with `change_kind` `bugfix` and no evidence, `tests.write` is admissible and `implementation.edit` is blocked, its reasons naming the claim `regression.reproduced` and the input `change_kind` with value `bugfix`.
6. Starting from `change-kinds` with `change_kind` `bugfix` and one `test_result` whose result is `fail`, `implementation.edit` is admissible, and stays admissible after the implementation moves to R2.
7. Starting from `change-kinds` with `change_kind` `refactor`, and again with `feature`, and no evidence, `implementation.edit` is admissible.
8. Starting from `protocols/vocabulary.yaml` as this story leaves it, each of `change_kind`, `feature`, `bugfix`, `refactor`, `tests.write`, `implementation.edit` and `regression.reproduced` resolves to exactly one vocabulary entry, and `repository.edit` resolves to none.
9. Starting from a `software.change` case that gives no `risk`, Canon refuses it as `missing-input`.
10. Starting from the protocol as this story leaves it, the committed `website/data/protocol-graphs/software-change-1.json` carries the four obligations' applicability and the `input` leaf of `implementation.edit`'s precondition, `generate --check` reports no drift, and `every_committed_protocol_graph_conforms_to_the_pinned_schema_and_its_protocol` passes against the transcription of the extended schema.

## Source

TASKBOARD E-003; Atlas ADR 0084 § Decision 1 and § Also needed; aep `principles/development/test-driven.yaml:18-20,26`; Atlas ADR 0077 point 3 (draft); `docs/design/engineering-lifecycle-specification-design.md` § 6.6, § 9, § 10, § 27; round-1 review `review-result:els-first-domain-parallel-safety-r1`.

## Order

Third of the three stories https://github.com/beyond10x/engineering-protocols/issues/7 asks to order: `story:software-change-negative-outcomes`, then `story:ml-experiment-protocol`, then this story. Canon 0.2.0 cleared `blocker:canon-case-inputs`; this story also depends on `story:canon-0-2-0-pin`, which moves the pin and renders case inputs in the generated pages.

It also waits on `blocker:graph-schema-case-inputs`: the generated dependency graph cannot carry a case input or an obligation's applicability until docs-system extends `b10x-protocol-graph/1`, and from `story:canon-0-2-0-pin` on the generator refuses rather than write a graph the schema rejects.
