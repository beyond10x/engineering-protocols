---
format: aep.planning-md/3
id: story:software-change-negative-outcomes
kind: story
status: active
title: Add declined, rolled-back, superseded and abandoned outcomes to software.change/1
summary: Terminal outcomes other than accepted, each earned by its own decision or evidence.
refs:
- provider: canon
  reference: taskboard:C-007
- provider: taskboard
  reference: E-006
relations:
- decomposes: epic:els-first-domain
- serves: vision:O2
- serves: vision:governed-autonomy
- depends_on: story:software-change-protocol
- depends_on: story:canon-0-2-0-pin
scope:
- confidence: cited
  path: crates/canon-engineering-docs/src/generate.rs
- confidence: cited
  path: crates/canon-engineering/src/vocabulary.rs
- confidence: cited
  path: crates/canon-engineering/tests/adversary_software_change_protocol.rs
- confidence: inferred
  path: crates/canon-engineering/tests/fixture_harness.rs
- confidence: cited
  path: crates/canon-engineering/tests/software_change_negative_outcomes.rs
- confidence: cited
  path: crates/canon-engineering/tests/software_change_protocol.rs
- confidence: cited
  path: crates/canon-engineering/tests/support/mod.rs
- confidence: cited
  path: crates/canon-engineering/tests/vocabulary_yaml.rs
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
  path: website/docs/protocols/index.md
- confidence: cited
  path: website/docs/protocols/software-change/1.mdx
- confidence: cited
  path: website/docs/vocabulary.md
revision: 22
transitions:
- {from: "draft", to: "proposed", at: "2026-10-12T21:33:46Z", actor: "human:timo", revision: 20, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-10-12T21:36:06Z", actor: "human:timo", revision: 22, decided_on: {"recorded":{"review_outcome":3}}}
---
## Outcome

`software.change/1` declares terminal outcomes beyond `accepted`:

- `declined`: earned on an `explicitly_declined` decision, and reachable without any implementation artifact (design § 6.7, § 9 `completion.declined`, ELS-OUTCOME-001);
- `rolled_back`: earned only with `rollback_result` evidence whose verdict is complete. Invoking a rollback is not enough (design § 9 `completion.rolled_back`, § 15, ELS-RECOVERY-001);
- `superseded`: earned on an `explicitly_superseded` decision recorded on this case;
- `abandoned`: earned on an `explicitly_abandoned` decision recorded on this case. The change stopped without reaching any planned outcome (design § 8.8). A downstream software-change loop ends in it (https://github.com/beyond10x/engineering-protocols/issues/7, gap 2).

To make `rolled_back` reachable, this story also declares two things in `protocols/software-change/1.yaml`. One is the action `release.rollback` on `software.change/1` with an authority requirement; the term is already in the vocabulary from `story:els-vocabulary`. The other is the evidence kind `rollback_result` with subject `release`. The story adds the terms it introduces to `protocols/vocabulary.yaml`: the outcomes `declined`, `rolled_back`, `superseded` and `abandoned`, the evidence kind `rollback_result`, and the decisions `explicitly_declined`, `explicitly_superseded` and `explicitly_abandoned`. The typed reader `crates/canon-engineering/src/vocabulary.rs` has no category for a decision, so this story adds that category to the reader and adds no term to the Rust (Atlas ADR 0077 point 3, draft). `story:rollback-verification-rule` later uses `rollback_result` in `incident.response/1`.

Not in this story: `inconclusive`. Design § 8.8 defines it as an investigation that finished without a sufficient conclusion. It is an outcome of `ml.experiment/1` (`architecture-decision-record:ml-protocol-shape`), and issue 7 does not ask for it on `software.change/1`.

The relation between a change and the change that supersedes it is not modelled here. Design § 45.5 leaves change nesting open, so the decision is a fact on this case only.

Pressure test: no outcome assumes Git, and a declined change carries no repository artifact.

## Scope

Read 2026-10-12 against Canon 0.2.0 (`dbb04ea`). Each line is cited (read in the tree) or inferred.

- `protocols/software-change/1.yaml`: the four outcomes, `release.rollback`, `rollback_result` (cited)
- `fixtures/software-change/`: new `declined`, `superseded`, `abandoned`, `rolled-back`; `chg-1842.fixture.yaml` gains `release.rollback` in each of its seven `actions:` maps, because the harness compares every declared action (`crates/canon-engineering/tests/support/mod.rs:670-673`) (cited)
- `protocols/vocabulary.yaml`: eight terms appended (cited)
- `crates/canon-engineering/src/vocabulary.rs`: `Category` and `Category::ALL` (`:36-66`) gain the decision category; the spelling rule (`:12-15`) names it as single-token (cited)
- `crates/canon-engineering/tests/support/mod.rs`: the fixture format gains `add_decisions` (`canon-decisions/1` records, added to those of earlier states as `add_authority` is) passed as `Supplied::decisions` to `eval::evaluate_with`, and `expect.outcomes`. Today it passes only `authority` (`:663-667`), `WrittenState` refuses unknown keys (`:268-280`) and `Expectation` has no outcomes (`:238-246`); the harness header asks that a story changing it say so in its scope (`:1-2`) (cited)
- `crates/canon-engineering/tests/software_change_negative_outcomes.rs` (new) (cited)
- `crates/canon-engineering/tests/software_change_protocol.rs:168-202`: exact evidence-kind, action and outcome lists (cited)
- `crates/canon-engineering/tests/adversary_software_change_protocol.rs:303-309`: `ABOUT` needs `("rollback_result", "release")` (cited)
- `crates/canon-engineering/tests/vocabulary_yaml.rs:390-398`: category match (cited)
- `crates/canon-engineering-docs/src/generate.rs:118-127` (`category_title`) and `:564` (term count 36) (cited)
- generated: `website/docs/vocabulary.md`, `website/docs/protocols/software-change/1.mdx`, `website/data/protocol-graphs/software-change-1.json`, `website/docs/protocols/index.md`, `website/data/status.json` (cited)
- `crates/canon-engineering/tests/fixture_harness.rs`, the `adversary*_harness_*.rs` tests (inferred: they exercise the format that gains keys)
- `CHANGELOG.md` `[Unreleased]` (inferred)

## What Canon 0.2.0 makes of it

Read in canon `dbb04ea`:

- An outcome earned on a decision is `requires: {decision: <name>}`, as the whole requirement, never inside `all`/`any`/`not` (`crates/canon/src/model/requirement.rs:4-14`). It is `legitimate` only with an applying `canon-decisions/1` record, else `blocked` with the reason `{decision, present: false}` (`crates/canon/src/eval/outcomes.rs:7-13`).
- A decision applies only when its `case_revision` equals the case's `revision`; a case with no `revision` gets none (`crates/canon/src/eval/decisions.rs:28-33`). So the four new fixtures give the case a `revision`.
- Canon lists every declared outcome as `legitimate` or `blocked` (`crates/canon/src/eval/outcomes.rs:5-14`). "No outcome" below means every declared outcome is `blocked`.
- A case lists every declared artifact (`crates/canon/src/eval/case.rs:73-80`, `missing-artifact`). A declined change therefore names `implementation` at a placeholder revision with no evidence about it, which is what "carries no implementation" means here.
- No Canon input records that an action was performed. "Invoking a rollback is not enough" is shown with a `rollback_result` whose result is not `complete`.

## Protocol first

Atlas ADR 0080 (draft). The first commit changes the following, and nothing else:

- `protocols/software-change/1.yaml`, adding the four outcomes, `release.rollback` and `rollback_result`;
- the fixtures `declined`, `superseded`, `abandoned` and `rolled-back` in `fixtures/software-change/`, with the expectations in § Acceptance, and `release.rollback` in `chg-1842`'s action maps;
- the harness keys `add_decisions` and `expect.outcomes` in `crates/canon-engineering/tests/support/mod.rs`, which only forward an input Canon already checks;
- `crates/canon-engineering/tests/software_change_negative_outcomes.rs`.

On that commit `negative_outcomes_are_earned` fails at item 8: the eight new terms do not yet resolve in the vocabulary, and the reader has no decision category. The implementation commit adds the terms, the category and its arms in the reader, the docs generator and the vocabulary tests, the expected lists in `software_change_protocol.rs` and `adversary_software_change_protocol.rs`, and regenerates the site.

## Shared surface

`protocols/software-change/1.yaml`, `fixtures/software-change/` and `protocols/vocabulary.yaml` are edited by every story on the `software.change/1` chain, and this story is its second link: `story:software-change-protocol` → `story:software-change-negative-outcomes` → `story:software-change-profiles` → `story:ess-conformance-evidence` → `story:security-independence-rules` → `story:stale-evidence-fixtures`. Canon has no composition that would let the outcomes live in their own file. This story depends on `story:software-change-protocol`. `story:software-change-profiles`, `story:ml-experiment-protocol` and `story:rollback-verification-rule` depend on it; `story:rollback-verification-rule` for `rollback_result`.

## Order

The three stories https://github.com/beyond10x/engineering-protocols/issues/7 asks to order, held by `depends_on` edges:

1. `story:software-change-negative-outcomes`, this story. Every Canon capability it uses is `implemented` in the canon store: outcomes (C-007, canon `story:outcomes`), the decision half (canon `story:decision-outcomes`) and action admissibility with authority (C-006, canon `story:action-admissibility`).
2. `story:ml-experiment-protocol`. It adds outcome terms to `protocols/vocabulary.yaml`, `superseded` among them, so it lands after this story.
3. `story:software-change-profiles`. Canon 0.2.0 cleared `blocker:canon-case-inputs`; it also waits on `story:canon-0-2-0-pin` and on docs-system for the graph format (`blocker:graph-schema-case-inputs`).

## Canon capability

Outcomes and completion (C-007), and action admissibility with authority (C-006) for `release.rollback`.

## Domain relations

- ELS protocol → Canon protocol model: many-to-one. Canon owns the language and an ELS protocol is a document in it; an ELS protocol cannot exist before the Canon capability it uses. Inferable: from `crates/canon-engineering/Cargo.toml:9`, the `b10x-canon` dependency, and `crates/canon-engineering/src/lib.rs:5`, which types protocol ids with `b10x_canon::ProtocolId`. No ess/1 document declares it, because ELS opts out of ESS for protocol semantics (`AGENTS.md` § ESS).

## Acceptance

The test `negative_outcomes_are_earned` in `crates/canon-engineering/tests/software_change_negative_outcomes.rs` passes under `task check`. It loads `protocols/software-change/1.yaml` through Canon by way of the harness. It expects:

1. Starting from the fixture `declined` in `fixtures/software-change/` (case revision `c1`, `implementation` at a placeholder revision with no evidence about it, no decision), every declared outcome is `blocked`, and `declined` names the absent decision `explicitly_declined`. After an `explicitly_declined` decision for case revision `c1` is added, `declined` is `legitimate` and the evidence holds no record about `implementation`.
2. Starting from the fixture `superseded` (case revision `c1`, implementation revision R1, no decision), every declared outcome is `blocked`. After an `explicitly_superseded` decision for `c1` is added, `superseded` is `legitimate`.
3. Starting from the fixture `abandoned` (case revision `c1`, implementation revision R1, no evidence, no decision), every declared outcome is `blocked`. After an `explicitly_abandoned` decision for `c1` is added, `abandoned` is `legitimate` and `accepted` is `blocked`.
4. Starting from the fixture `abandoned` with an `explicitly_abandoned` decision whose `case_revision` is `c0`, not the case's `c1`, `abandoned` is `blocked`: a decision on another revision of the case does not earn it.
5. Starting from the fixture `rolled-back` (release revision v2, a `deployment`, no authority decision, no `rollback_result`), `release.rollback` is `approval-required` and `rolled_back` is `blocked`.
6. Starting from the state of 5, with authority granted for `release.rollback` and a `rollback_result` about release v2 whose result is `partial`, `release.rollback` is `admissible` and `rolled_back` is `blocked`.
7. Starting from the state of 6 and adding a `rollback_result` about release v1 whose result is `complete`, `rolled_back` is still `blocked` (the record is about another release revision, so it is UNKNOWN, not TRUE); adding one about release v2 whose result is `complete` makes `rolled_back` `legitimate`.
8. Starting from `protocols/vocabulary.yaml` as this story leaves it, read through `crates/canon-engineering/src/vocabulary.rs`, each of `declined`, `rolled_back`, `superseded`, `abandoned`, `rollback_result`, `explicitly_declined`, `explicitly_superseded` and `explicitly_abandoned` resolves to exactly one vocabulary entry.

## Source

TASKBOARD E-006; https://github.com/beyond10x/engineering-protocols/issues/7 (gap 2: `abandoned`); Atlas ADR 0077 point 3 (draft); `docs/design/engineering-lifecycle-specification-design.md` § 6.7, § 8.8, § 9, § 14, § 15, § 39.8–§ 39.10; round-1 reviews `review-result:els-first-domain-acceptance-r1`, `review-result:els-first-domain-design-r1`.
