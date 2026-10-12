---
format: aep.planning-md/3
id: story:ml-experiment-protocol
kind: story
status: active
title: Ship the ml.experiment/1 protocol as data
relations:
- decomposes: epic:els-first-domain
- depends_on: story:ml-protocol-shape-decision
- depends_on: story:software-change-negative-outcomes
- serves: vision:O2
- informed_by: architecture-decision-record:ml-protocol-shape
scope:
- confidence: inferred
  path: AGENTS.md
- confidence: cited
  path: README.md
- confidence: cited
  path: crates/canon-engineering-docs/src/generate.rs
- confidence: cited
  path: crates/canon-engineering/tests/ml_experiment_protocol.rs
- confidence: cited
  path: fixtures/ml-experiment/
- confidence: cited
  path: protocols/ml-experiment/1.yaml
- confidence: cited
  path: protocols/vocabulary.yaml
- confidence: cited
  path: website/data/protocol-graphs/ml-experiment-1.json
- confidence: cited
  path: website/data/status.json
- confidence: cited
  path: website/docs/protocols/index.md
- confidence: cited
  path: website/docs/protocols/ml-experiment/
- confidence: cited
  path: website/docs/status.mdx
- confidence: cited
  path: website/docs/vocabulary.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-10-12T21:33:46Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-12T21:36:06Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome

This repository ships `ml.experiment/1` as Canon `protocol/1` data at `protocols/ml-experiment/1.yaml`, a fourth built-in, with the artifacts, claims and terminal outcomes `architecture-decision-record:ml-protocol-shape` records, and fixtures under `fixtures/ml-experiment/`. Its terms go into `protocols/vocabulary.yaml`, marked `core`: none of them assumes Git, pull requests or code (`crates/canon-engineering/src/vocabulary.rs:77-84`).

/1 declares:

- artifacts `experiment_design`, `dataset_snapshot`, `baseline`, `candidate`, `run_configuration`, `results`;
- evidence kinds `design_review`, `integrity_check`, `reproduction`, `comparison`, `seed_sweep`;
- claims `experiment.valid` (all of `design_review` `passed` on `experiment_design` and `integrity_check` `clean` on `results`), `result.reproducible` (`reproduction` `matched` on `results`), `candidate.beats_baseline` (`comparison` `candidate_better` on `candidate`), `effect.robust` (`seed_sweep` `stable` on `candidate`), `hypothesis.supported` (all four);
- outcomes `supported` (`hypothesis.supported`), `refuted` (`experiment.valid` and `result.reproducible` TRUE, `candidate.beats_baseline` `is: false`), `invalid_experiment` (`experiment.valid` `is: false`), `inconclusive` (decision `explicitly_inconclusive`) and `superseded` (decision `explicitly_superseded`);
- invalidation rules: a new `dataset_snapshot` revision invalidates `result.reproducible`, `candidate.beats_baseline` and `effect.robust`; a new `baseline` revision invalidates `candidate.beats_baseline`.

The ADR's names are respelled where the vocabulary's rules refuse them (`crates/canon-engineering/src/vocabulary.rs:304`, `:311-313`): the decisions are `explicitly_inconclusive` and `explicitly_superseded`, so that `superseded` is declared once, as an outcome; `superseded` and `explicitly_superseded` are the entries `story:software-change-negative-outcomes` adds, reused, not repeated.

Pressure test: `refuted` exists only because UNKNOWN is not FALSE. A comparison missing, or about another candidate revision, leaves `candidate.beats_baseline` UNKNOWN, and `refuted` stays blocked.

Left out of /1, each a later change in place:

- obligations `preregister_design` and `freeze_dataset_snapshot`, and a `study_kind` case input whose `applicable_when` applies preregistration to confirmatory studies only (Canon 0.2.0 can express it);
- actions (`experiment.run`, `run.reproduce`, `baseline.compare`, `conclusion.record`);
- artifacts `research_question`, `hypothesis`, `analysis`, `conclusion`, to which no evidence binds;
- independent reproduction, the successor case on `superseded`, `reproducibility/1` composition (Canon gaps the ADR names: `EvidenceRecord` has no producer field, canon `crates/canon/src/model/evidence.rs:16-43`; an explicit decision names no other case, `model/explicit.rs:22-36`; `protocol/1` has no imports, `model/mod.rs:56-82`);
- `latency.acceptable`, and an invalidation rule for `run_configuration`.

## Why this story

`epic:els-first-domain` promises the ML protocol's shape decided (TASKBOARD E-009), and `story:ml-protocol-shape-decision` delivered that as `architecture-decision-record:ml-protocol-shape`, which says: "This record builds nothing. The protocol gets its own story with its own § Protocol first." This is that story. The build is asked for by the record's § Decision ("It will live at `protocols/ml-experiment/1.yaml` in this repository"), by its reason 4 (Canon's phase 1 validates the kernel against `software.change/1`, `incident.response/1` and `ml.experiment/1`), and by https://github.com/beyond10x/engineering-protocols/issues/7, which asked for it to be ordered among the next three stories. It decomposes the epic as the E-009 follow-on, not as a second claim on the decision.

## Scope

Read 2026-10-12 against Canon 0.2.0 (`dbb04ea`). Each line is cited (read in the tree) or inferred.

- `protocols/ml-experiment/1.yaml` (new) (cited)
- `fixtures/ml-experiment/` (new) (cited)
- `protocols/vocabulary.yaml` (cited)
- `crates/canon-engineering/tests/ml_experiment_protocol.rs` (new) (cited)
- `crates/canon-engineering-docs/src/generate.rs:564`, the hard-coded vocabulary term count (cited)
- `README.md:28-33` (protocol table), `:68` and `:75-78` (`protocols list` output gains `ml-experiment@1`) (cited)
- generated: `website/docs/protocols/ml-experiment/` (new), `website/data/protocol-graphs/ml-experiment-1.json` (new), `website/docs/protocols/index.md`, `website/data/status.json`, `website/docs/status.mdx`, `website/docs/vocabulary.md` (cited: `generate.rs:226-268`, `status.rs:138-161`)
- `AGENTS.md:19` (ML experiment is no longer "later") and a row in § Invariants for the refutation guarantee (inferred)
- `CHANGELOG.md` `[Unreleased]` (inferred)
- No edit: `crates/canon-engineering/build.rs` embeds any `protocols/<name>/<major>.yaml` and `tests/protocol_registry.rs:76-103` compares with the files on disk (cited)

It uses the harness keys `add_decisions` and `expect.outcomes`, which `story:software-change-negative-outcomes` adds to `crates/canon-engineering/tests/support/mod.rs`; this story does not edit the harness.

## Order

Second of the three stories https://github.com/beyond10x/engineering-protocols/issues/7 asks to order: `story:software-change-negative-outcomes`, then this story, then `story:software-change-profiles`. It depends on `story:software-change-negative-outcomes` for the shared terms `superseded` and `explicitly_superseded`, the reader's decision category and the harness keys.

## Protocol first

Atlas ADR 0080: the first commit adds `protocols/ml-experiment/1.yaml`, the fixtures with the expectations in § Acceptance, and `crates/canon-engineering/tests/ml_experiment_protocol.rs`. On it `ml_experiment_outcomes_are_earned` fails at item 2: the new terms do not resolve in the vocabulary. The implementation commit adds the terms, raises the count in `generate.rs`, regenerates the site and updates `README.md`.

## Acceptance

The test `ml_experiment_outcomes_are_earned` in `crates/canon-engineering/tests/ml_experiment_protocol.rs` passes under `task check`. It loads `protocols/ml-experiment/1.yaml` through Canon by way of the harness, with fixtures under `fixtures/ml-experiment/`; each case gives `revision: c1` and the current revision of every artifact: `experiment_design` x1, `dataset_snapshot` d1, `baseline` b1, `candidate` m1, `run_configuration` k1, `results` s1. It expects:

1. Starting from the protocol file, it validates and compiles in Canon, and the registry lists `ml-experiment@1` beside the other built-ins.
2. Starting from `protocols/vocabulary.yaml` as this story leaves it, every artifact, evidence kind, claim and outcome id of /1 and the decision `explicitly_inconclusive` resolves to exactly one vocabulary entry marked `core`, and `superseded` and `explicitly_superseded` still resolve to exactly one each.
3. Starting from no evidence and no decision, every claim is `unknown` and every outcome is `blocked`.
4. Starting from design review `passed`, integrity `clean`, reproduction `matched`, comparison `candidate_better` on m1 and seed sweep `stable`, all five claims are `true`, `supported` is `legitimate`, and `refuted` and `invalid_experiment` are `blocked`.
5. Starting from the evidence of 4 with the comparison `baseline_better` instead, `candidate.beats_baseline` is `false`, `refuted` is `legitimate` and `supported` is `blocked`.
6. Starting from the evidence of 5 with `candidate` moved to m2, the comparison about m1 no longer counts: `candidate.beats_baseline` is `unknown`, not `false`, and `refuted` and `supported` are `blocked`.
7. Starting from the evidence of 4, each record carrying `upstream_revisions: {dataset_snapshot: d1}`, with `dataset_snapshot` moved to d2, `result.reproducible`, `candidate.beats_baseline`, `effect.robust` and `hypothesis.supported` are `unknown`, `experiment.valid` stays `true`, and `supported` is `blocked`.
8. Starting from the evidence of 4, the `comparison` record carrying `upstream_revisions: {baseline: b1}`, with `baseline` moved to b2, `candidate.beats_baseline` and `hypothesis.supported` are `unknown`, `result.reproducible`, `effect.robust` and `experiment.valid` stay `true`, and `supported` is `blocked`.
9. Starting from design review `passed`, integrity `violated` and a `baseline_better` comparison, `experiment.valid` is `false`, `invalid_experiment` is `legitimate` and `refuted` is `blocked`.
10. Starting from 3, after an `explicitly_inconclusive` decision for case revision `c1` is added, `inconclusive` is `legitimate`; the same decision for `c0` leaves it `blocked`.
11. Starting from 3, after an `explicitly_superseded` decision for `c1` is added, `superseded` is `legitimate`.
12. In no state of the fixtures are two of `supported`, `refuted` and `invalid_experiment` `legitimate` together.

## Source

`architecture-decision-record:ml-protocol-shape` (wave 2026-10-04-w3); TASKBOARD E-009.
