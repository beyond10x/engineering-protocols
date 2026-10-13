---
format: aep.planning-md/3
id: story:security-independence-rules
kind: story
status: active
title: Add the independent security-review rule to software.change/1
summary: security.reviewed requires an approving security_review from a principal independent of the implementer when the change affects a security boundary.
refs:
- provider: canon
  reference: taskboard:C-004
- provider: canon
  reference: taskboard:C-010
- provider: taskboard
  reference: E-008
relations:
- decomposes: epic:els-first-domain
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:canon-0-4-0-pin
scope:
- confidence: inferred
  path: crates/canon-engineering/src/vocabulary.rs
- confidence: cited
  path: crates/canon-engineering/tests/security_independence_rules.rs
- confidence: cited
  path: fixtures/software-change/
- confidence: cited
  path: protocols/software-change/1.yaml
- confidence: cited
  path: protocols/vocabulary.yaml
revision: 20
transitions:
- {from: "draft", to: "proposed", at: "2026-10-13T13:41:46Z", actor: "human:timo", revision: 19, decided_on: {"recorded":{"review_outcome":5}}}
- {from: "proposed", to: "active", at: "2026-10-13T13:41:47Z", actor: "human:timo", revision: 20, decided_on: {"recorded":{"review_outcome":5}}}
---
## Outcome

`software.change/1` gains an independent security-review rule in `protocols/software-change/1.yaml`:

- The claim `security.reviewed` holds on an approving `security_review` that is bound to the current implementation revision and comes from a producer independent of the implementing principal (`different_principal`).
- A change with the case input `affects_security_boundary` derives a security-review obligation (design § 10, § 12).

`security.reviewed` is a conjunct that extends `implementation.verified`, which `story:software-change-protocol` defines (design § 9). This story edits that definition so that `implementation.verified` also requires `security.reviewed` when `affects_security_boundary` holds. It is not a free-standing claim.

ELS declares which independence dimensions a requirement needs; it does not resolve identities. The fixture supplies the independence decision as trusted input (design § 12, § 45.3; Canon design § 39.4). Canon owns the requirement form and the rule that evidence failing it does not satisfy (CANON-INDEPENDENCE-001).

The story adds the terms it introduces to `protocols/vocabulary.yaml`: the evidence kind `security_review`, the claim `security.reviewed`, the case input `affects_security_boundary`, and the independence dimensions `different_principal` and `different_agent_run`. The typed reader `crates/canon-engineering/src/vocabulary.rs` has no category for an independence dimension, so this story adds that category and adds no term to the Rust (Atlas ADR 0077 point 3, draft).

What stays out: the rollback-verification rule in `incident.response/1`. `story:rollback-verification-rule` declares it, using the dimension `different_principal` added here.

## Scope

- `protocols/software-change/1.yaml`
- `fixtures/software-change/`
- `protocols/vocabulary.yaml`
- `crates/canon-engineering/src/vocabulary.rs` (inferred: the reader's independence-dimension category)
- `crates/canon-engineering/tests/security_independence_rules.rs` (new)

## Shared surface

`protocols/software-change/1.yaml`, `fixtures/software-change/` and `protocols/vocabulary.yaml` are edited by every story on the `software.change/1` chain. In wave 2026-10-13-w3 this story is the first unit, then `story:rollback-verification-rule`, then `story:stale-evidence-fixtures`, serially on one integration branch. The ordering edge to `story:ess-conformance-evidence` (draft, not in this wave) is dropped: it only kept two edits of `implementation.verified` apart, and that story will extend the definition this one leaves. It edits neither `protocols/incident-response/1.yaml` nor `fixtures/incident-response/`.

`story:rollback-verification-rule` depends on this story: it uses `different_principal`, the independence-dimension category this story adds to the typed reader, and both edit `protocols/vocabulary.yaml`.

## Protocol first

Atlas ADR 0080 (draft). The first commit changes the following, and nothing else:

- `protocols/software-change/1.yaml`, adding `security_review`, `security.reviewed`, the derived obligation and the extended `implementation.verified`;
- the fixture `security-boundary` in `fixtures/software-change/`, with the expectations in § Acceptance;
- `crates/canon-engineering/tests/security_independence_rules.rs`.

On that commit `independent_security_review_gates_implementation_verified` fails at item 5. The five new terms do not yet resolve in the vocabulary, and the reader has no independence-dimension category. The implementation commit adds the terms to `protocols/vocabulary.yaml` and the category to `crates/canon-engineering/src/vocabulary.rs`.

## Canon capability

Carried by Canon 0.4.0 (tag peels to 7c2cec13), pinned by this wave:

- `protocol/1` `independence_dimensions` declares `different_principal` and `different_agent_run`;
- an evidence match takes `independent_on: [different_principal]`;
- each `canon-evidence/1` record names its `producer`, and the fixture's `canon-case/1` `independence` map, written by the trusted context, lists the dimensions each producer is independent on;
- a record that fails the demand is inapplicable, never contrary: the claim is `UNKNOWN` and the explanation lists it as `not_independent` (`CANON-INDEPENDENCE-001`).

Case inputs (`affects_security_boundary`, values `true` and `false`) are carried since Canon 0.2.0. C-010's requirements catalogue (`story:conformance-suite` in Canon) is not needed: this story asserts Canon's decision on fixtures, not a catalogue entry.

Every `canon-case/1` for `software.change/1` must now give `affects_security_boundary`, as Canon refuses a case missing a declared input; every fixture under `fixtures/software-change/` gains it, and the release notes say so for case writers.

## Domain relations

- ELS protocol → Canon protocol model: many-to-one. Canon owns the language and an ELS protocol is a document in it; an ELS protocol cannot exist before the Canon capability it uses. Inferable: from `crates/canon-engineering/Cargo.toml:9`, the `b10x-canon` dependency, and `crates/canon-engineering/src/lib.rs:5`, which types protocol ids with `b10x_canon::ProtocolId`. No ess/1 document declares it, because ELS opts out of ESS for protocol semantics (`AGENTS.md` § ESS).

## Acceptance

The test `independent_security_review_gates_implementation_verified` in `crates/canon-engineering/tests/security_independence_rules.rs` passes under `task check`. It loads `protocols/software-change/1.yaml` through Canon by way of the harness. The fixture `security-boundary` in `fixtures/software-change/` gives its case `independence: {p1: [], p2: [different_principal]}`; P1 is the implementing principal. It expects:

1. Starting from the fixture `security-boundary` (implementation revision R1, `affects_security_boundary: true`, a passing `test_result` bound to R1 produced by P1, no `security_review`), evaluation reports `security.reviewed = UNKNOWN`, `implementation.verified = UNKNOWN`, the obligation `security_review.required` open and `repository.merge` blocked.
2. Starting from the state of 1 and adding an approving `security_review` bound to R1 with `producer: p1`, evaluation reports `security.reviewed = UNKNOWN` (not `FALSE`) and `implementation.verified = UNKNOWN`, and Canon's explanation of `security.reviewed` lists that record as `not_independent`.
3. Starting from the state of 1 and adding an approving `security_review` bound to R1 with `producer: p2`, evaluation reports `security.reviewed = TRUE`, `implementation.verified = TRUE` and `security_review.required` discharged.
4. Starting from the state of 1 with `affects_security_boundary: false`, evaluation reports `implementation.verified = TRUE` with no `security_review`, and `security_review.required` not applicable.
5. Starting from `protocols/vocabulary.yaml` as this story leaves it, read through `crates/canon-engineering/src/vocabulary.rs`, each of `security_review`, `security.reviewed`, `affects_security_boundary`, `different_principal` and `different_agent_run` resolves to exactly one vocabulary entry, the last two in the independence-dimension category.
6. Every existing fixture under `fixtures/software-change/` gives `affects_security_boundary: false` and keeps every expectation it had.

## Source

TASKBOARD E-008; Atlas ADR 0077 point 3 (draft); `docs/design/engineering-lifecycle-specification-design.md` § 9, § 10, § 12, § 39.5; round-1 reviews `review-result:els-first-domain-acceptance-r1`, `review-result:els-first-domain-design-r1`, `review-result:els-first-domain-parallel-safety-r1`; round-2 review `review-result:els-first-domain-design-r2` (split into this story and `story:rollback-verification-rule`).
