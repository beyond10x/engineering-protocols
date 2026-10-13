---
format: aep.planning-md/3
id: story:stale-evidence-fixtures
kind: story
status: implemented
title: Add stale-evidence fixtures across both protocols
summary: Revision-stale reviews and freshness-expired observations evaluate UNKNOWN, never FALSE, in delivery and incident fixtures.
refs:
- provider: canon
  reference: taskboard:C-004
- provider: canon
  reference: taskboard:C-008
- provider: taskboard
  reference: E-005
relations:
- decomposes: epic:els-first-domain
- depends_on: story:software-change-protocol
- depends_on: story:incident-response-protocol
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:security-independence-rules
- depends_on: story:rollback-verification-rule
- depends_on: story:incident-investigation-obligation
- depends_on: story:canon-0-4-0-pin
scope:
- confidence: cited
  path: crates/canon-engineering/tests/stale_evidence.rs
- confidence: cited
  path: crates/canon-engineering/tests/support/mod.rs
- confidence: cited
  path: fixtures/incident-response/
- confidence: cited
  path: fixtures/software-change/
- confidence: cited
  path: protocols/incident-response/1.yaml
- confidence: cited
  path: protocols/software-change/1.yaml
revision: 16
transitions:
- {from: "draft", to: "proposed", at: "2026-10-13T13:41:49Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-10-13T13:41:49Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "active", to: "implemented", at: "2026-10-13T15:42:48Z", actor: "human:timo", revision: 16, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
## Outcome

This story owns the staleness rules for review, observation and health evidence in both protocols. It declares each rule in the protocol YAML and lands the fixture that exercises it in the same change, so no staleness rule lands without its fixture (`AGENTS.md` § Rules).

- In `protocols/software-change/1.yaml`:
  - `code_review` evidence is bound to the implementation revision it reviewed, so `implementation.reviewed` holds only on a review of the current revision.
  - `operational_observation` evidence carries a 30-minute freshness horizon, so `deployment.healthy` holds only on a fresh observation (design § 9, § 11.3).
- In `protocols/incident-response/1.yaml`: `operational_observation` health evidence carries a 30-minute freshness horizon, so `service.healthy` holds only on a fresh observation. Design § 11.3 states 30 minutes for operational evidence and gives no separate incident horizon, so the incident horizon takes the same value.

Each stale case evaluates `UNKNOWN`, never `FALSE` (`AGENTS.md` § Rules). The R1/R2 `test_result` case stays in `story:software-change-protocol`, and staleness of conformance evidence after a specification revision stays in `story:ess-conformance-evidence`. The story introduces no term, so it does not edit `protocols/vocabulary.yaml` or `crates/canon-engineering/src/vocabulary.rs`.

The fixtures exercise Canon behaviour through ELS protocols; ELS does not implement applicability or freshness. The evaluation instant is passed explicitly. Rust in this story is test code only.

## Scope

- `protocols/software-change/1.yaml`
- `fixtures/software-change/`
- `protocols/incident-response/1.yaml`
- `fixtures/incident-response/`
- `crates/canon-engineering/tests/support/mod.rs` (the evaluation instant reaches Canon; per-state `at`)
- `crates/canon-engineering/tests/stale_evidence.rs` (new)

## Shared surface

This story edits both protocol documents and both fixture trees, so it is the last link of both chains:

- on `protocols/software-change/1.yaml` and `fixtures/software-change/`: `story:software-change-protocol` → `story:software-change-negative-outcomes` → `story:software-change-profiles` → `story:ess-conformance-evidence` → `story:security-independence-rules` → `story:stale-evidence-fixtures`;
- on `protocols/incident-response/1.yaml` and `fixtures/incident-response/`: `story:incident-response-protocol` → `story:rollback-verification-rule` → `story:stale-evidence-fixtures`.

It depends on `story:software-change-protocol`, `story:incident-response-protocol`, `story:security-independence-rules` and `story:rollback-verification-rule`. `story:protocol-docs-render` depends on it, because it is the last story that changes either built-in protocol.

## Protocol first

Atlas ADR 0080 (draft). The first commit changes the following, and nothing else:

- both protocol documents, adding the revision binding on `code_review` and the freshness horizons;
- the fixtures `stale-review` and `stale-observation` in `fixtures/software-change/` and `stale-health` in `fixtures/incident-response/`, with the expectations in § Acceptance;
- `crates/canon-engineering/tests/stale_evidence.rs`.

The red test is `stale_evidence_is_unknown_and_fresh_evidence_holds`. **This story has no ELS implementation beyond that data.** It adds no term and no Rust outside the test, so the test fails on the first commit only if Canon (revision binding C-004, freshness C-008) does not yet evaluate what the YAML declares. If the test passes on the first commit, the story records that run as its baseline and says no implementation commit follows. ADR 0080 does not yet say how it applies to a story whose whole change is protocol data; that question is open.

## Canon capability

Carried since Canon 0.1.0 and pinned at 0.4.0: revision binding (C-004: a record applies only while its `subject_revision` is the case's current revision of its subject) and freshness (C-008: an evidence kind's `max_age`, here `30m`, excludes a record older than that at the evaluation instant as expired, leaving the claim `UNKNOWN`). The instant reaches Canon as `eval::Supplied.at`.

Harness change, in this story's scope (`crates/canon-engineering/tests/support/mod.rs`): the harness passes the fixture's `at` to Canon, and a state may give its own `at` (an instant not before any evidence so far and not before the previous state's), so one fixture evaluates the same observation at T0 + 29 and T0 + 31 minutes. Existing fixtures whose operational observations would be older than 30 minutes at their instant move their instants, not their expectations, and the change says which.

`code_review` is already bound to the implementation revision (`subject: implementation`), so expectations 1 and 2 may pass on the first commit; that run is recorded as their baseline, and the implementation commit for them is the harness change only.

## Domain relations

- ELS protocol → Canon protocol model: many-to-one. Canon owns the language and an ELS protocol is a document in it; an ELS protocol cannot exist before the Canon capability it uses. Inferable: from `crates/canon-engineering/Cargo.toml:9`, the `b10x-canon` dependency, and `crates/canon-engineering/src/lib.rs:5`, which types protocol ids with `b10x_canon::ProtocolId`. No ess/1 document declares it, because ELS opts out of ESS for protocol semantics (`AGENTS.md` § ESS).

## Acceptance

The test `stale_evidence_is_unknown_and_fresh_evidence_holds` in `crates/canon-engineering/tests/stale_evidence.rs` passes under `task check`. It loads both protocol documents through Canon by way of the harness. It expects:

1. Starting from the fixture `stale-review` in `fixtures/software-change/` (implementation revision R2, a passing `test_result` bound to R2, one approving `code_review` bound to R1), evaluation reports `implementation.reviewed = UNKNOWN`, not `FALSE`.
2. Starting from the same fixture with the approving `code_review` bound to R2 instead, evaluation reports `implementation.reviewed = TRUE`.
3. Starting from the fixture `stale-observation` in `fixtures/software-change/` (a `deployment` with one `operational_observation` reporting healthy at T0), evaluated at T0 + 31 minutes, evaluation reports `deployment.healthy = UNKNOWN`, not `FALSE`.
4. Starting from the same fixture evaluated at T0 + 29 minutes, evaluation reports `deployment.healthy = TRUE`.
5. Starting from the fixture `stale-health` in `fixtures/incident-response/` (the INC-492 case with `release.rollback` approved and one `operational_observation` reporting the service healthy at T0), evaluated at T0 + 31 minutes, evaluation reports `service.healthy = UNKNOWN` (not `FALSE`), `restore_service` open and `emergency.leave` blocked.
6. Starting from the same fixture evaluated at T0 + 29 minutes, evaluation reports `service.healthy = TRUE`, `restore_service` satisfied and `emergency.leave` admissible.

## Source

TASKBOARD E-005; Atlas ADR 0077 point 3 (draft); `docs/design/engineering-lifecycle-specification-design.md` § 9, § 11, § 39.7; `AGENTS.md` § Rules; round-1 reviews `review-result:els-first-domain-acceptance-r1`, `review-result:els-first-domain-design-r1`, `review-result:els-first-domain-parallel-safety-r1`.
