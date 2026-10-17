---
format: aep.planning-md/3
id: story:eval-gated-change-protocol
kind: story
status: implemented
title: Ship the eval.gated_change/1 protocol as data
relations:
- decomposes: epic:els-first-domain
- serves: vision:O2
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: README.md
- confidence: cited
  path: crates/canon-engineering-docs
- confidence: cited
  path: crates/canon-engineering/tests/eval_gated_change_protocol.rs
- confidence: cited
  path: fixtures/eval-gated-change
- confidence: cited
  path: protocols/eval-gated-change
- confidence: cited
  path: protocols/vocabulary.yaml
- confidence: cited
  path: website
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-10-17T05:05:32Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-17T05:05:32Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-17T07:33:22Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"test_result":2}}}
---
## Outcome

This repository ships `eval.gated_change/1` as Canon `protocol/1` data at
`protocols/eval-gated-change/1.yaml`, a fifth built-in, with fixtures under
`fixtures/eval-gated-change/`, in the shape `architecture-decision-record:eval-gated-change-shape`
records (accepted by the operator on 2026-10-17). It governs an automated loop that
tunes an agent's text or configuration (guidance, a system prompt, tool caps) against an
evaluation set: a change is kept only on evidence about that exact change, from a producer other
than the run that wrote it, so the loop's own say-so never satisfies the gate.

The source is a consumer's neutral proposal for `eval.gated_change/1` (not in this repository).
Its model validates and compiles with Canon 0.6.0 and checks at 55,296 states with no findings.
/1 declares what the proposal declares:

- artifacts `eval_design` and `candidate`; the `candidate` revision hashes its content together
  with its basis (baseline, question set, data revision, design), so a moved basis is a new
  candidate revision;
- evidence kinds `design_review` (passed or failed), `guard_check` (clean, leaked or outside: the
  surface, expected answers inside the candidate, new forbidden content in answers),
  `tuning_comparison` (reproduced, not_reproduced or not_better: a first and a second run) and
  `heldout_comparison` (held or regressed);
- claims `experiment.valid`, `change.within_surface`, `content.clean`, `tuning.improved`,
  `result.reproducible`, `heldout.not_regressed` and `change.supported` (all of the keep claims);
  every claim on the candidate demands `independent_on: [different_agent_run]`, as
  `security.reviewed` demands `different_principal` (`protocols/software-change/1.yaml:103-112`);
- actions `design.review` (capability `eval_design.review`), `change.keep` (precondition
  `change.supported` TRUE, capability `loop_change.keep`, the idiom of `repository.merge` at
  `protocols/software-change/1.yaml:256-265`), `change.reject` (some keep claim FALSE),
  `change.propose` (outside the surface) and `change.apply_outside_surface` (a capability never
  granted to a loop);
- outcomes `kept`, `rejected`, `proposed`, and `inconclusive` on the decision
  `explicitly_inconclusive` (`protocols/ml-experiment/1.yaml:114-117`).

UNKNOWN blocks both keep and reject: missing, stale or non-independent evidence never becomes a
decision either way.

Reused, not repeated, in `protocols/vocabulary.yaml`: `candidate` (`:188`), `design_review`
(`:200`), `experiment.valid` (`:220`), `result.reproducible` (`:224`), `inconclusive` (`:252`),
`explicitly_inconclusive` (`:256`), `different_agent_run` (`:368`). The new terms are marked
`core` only where none assumes Git, pull requests or code.

### How `canon check` is met

`canon check` refuses a protocol of more than 65,536 states (`STATE_BOUND`,
`canon:crates/canon/src/check/mod.rs:234`, the same at Canon tags 0.4.0, 0.5.0 and 0.6.0), and
the bound is a constant, not an option. The natural model, with baseline, question set, data
revision and design as four upstream artifacts carrying invalidation rules, has more than 2^128
states: each such evidence dimension takes `n^(c * 2^m)` values (`check/mod.rs:46-48`, `:67-69`).

/1 meets the bound by binding every evidence record to the one `candidate` revision that hashes
the basis, and declaring **no invalidation rule**: a record about an earlier basis is excluded as
`revision_mismatch` by revision binding, so `m = 0` in every dimension. Splitting the claim graph
into several protocols is not taken: `change.keep` needs every keep claim in one precondition and
`protocol/1` has no imports (`canon:crates/canon/src/model/mod.rs:68-97`). Checking per-action
sub-models is not taken: it checks documents that are not the shipped one. Bounding case inputs
does not help: /1 declares none, and an input adds one factor per value (`check/mod.rs:33-35`).

The headroom is small: 55,296 is 84% of the bound, and evidence, capability and decision
dimensions multiply (`check/mod.rs:10-13`, `:24-32`). One more 3-valued capability or one more
decision would pass it. Any later revision of /1 recounts before it lands.

## Scope

Files it would add or change (cited: read in the tree at `08b6d42`; inferred: not read):

- `protocols/eval-gated-change/1.yaml` (new) (cited: `build.rs` embeds any
  `protocols/<name>/<major>.yaml`, AGENTS.md § Invariants)
- `fixtures/eval-gated-change/` (new) (cited: `fixtures/` holds one directory per protocol)
- `crates/canon-engineering/tests/eval_gated_change_protocol.rs` (new), with a `canon check` test
  modelled on `crates/canon-engineering/tests/support_triage_protocol.rs:279-286` (cited)
- `protocols/vocabulary.yaml`: the new terms (cited)
- `README.md:28-33` protocol table and `:69-79` `protocols list` output (cited)
- generated, by `task docs`: `website/docs/protocols/eval-gated-change/` (new),
  `website/data/protocol-graphs/eval-gated-change-1.json` (new), `website/docs/protocols/index.md`,
  `website/data/status.json`, `website/docs/status.mdx`, `website/docs/vocabulary.md` (cited:
  AGENTS.md § Generated files)
- any hard-coded vocabulary term count in `crates/canon-engineering-docs/src/generate.rs`
  (inferred)
- `AGENTS.md:20` (the protocol list) and a row in § Invariants for the independence guarantee
  (inferred)
- `CHANGELOG.md` `[Unreleased]` (inferred)

Not here: the Canon pin. The repository pins Canon 0.6.0 (`crates/canon-engineering/Cargo.toml:18`),
the release the proposal's model was checked with; no pin story is needed first.

## Open

Items for Canon that the proposal surfaced. None is work in this repository; /1 ships without them.

1. Numeric comparison inside a claim (tuning score at least baseline plus a gain), which would
   remove the harness-computed `candidate_better` string; results are plain strings today.
2. Protocol imports or extension, so this protocol could extend `ml.experiment/1` instead of
   copying its idioms (`canon:crates/canon/src/model/mod.rs:68-97` has no import section).
3. A larger or configurable `canon check` bound, or symmetry reduction over upstream artifacts,
   which would let a later revision name which part of the basis moved.
4. A `case_outcome` evidence kind (decided upstream, not built), so an iteration's decision could
   feed a run-level case.

Two readings to confirm in the Protocol first commit (the proposer's model is not at hand; the fixtures decide), rewriting the
acceptance line before the test if they differ: that `change.reject` is not admissible while any
keep claim is UNKNOWN, even with another FALSE (acceptance 10); and that the vocabulary entry for
`candidate` still holds for a candidate whose revision hashes its basis.

## Protocol first

AGENTS.md § ESS: the first commit adds `protocols/eval-gated-change/1.yaml`, the fixtures with
the expectations in § Acceptance, and `crates/canon-engineering/tests/eval_gated_change_protocol.rs`.
On it, `eval_gated_change_outcomes_are_earned` fails at item 2: the new terms do not resolve in
the vocabulary. The failing run is recorded under `.engineering/evidence/story/`. Later commits
add the terms, regenerate the site and update `README.md` and `AGENTS.md`.

## Acceptance

The tests `eval_gated_change_outcomes_are_earned` and
`eval_gated_change_check_stays_under_the_bound` in
`crates/canon-engineering/tests/eval_gated_change_protocol.rs` pass under `task check`. Each
fixture case gives the case revision and the current revision of `eval_design` and `candidate`
(`h1`), and lists the producer `run-b` as independent on `different_agent_run`; `run-a` wrote the
candidate and is not listed. They expect:

1. Starting from the protocol file, it validates and compiles in Canon at the tag the repository
   pins, and the registry lists `eval-gated-change@1` beside the other built-ins.
2. Starting from `protocols/vocabulary.yaml` as this story leaves it, every artifact, evidence
   kind, claim, action and outcome id of /1 resolves to exactly one vocabulary entry (capability ids
   are not checked: `vocabulary/1` has no capability category), and the reused terms listed in
   § Outcome still resolve to exactly one each.
3. `canon check` over the compiled /1 runs, its report is clean (no findings), and its state
   count is at most `STATE_BOUND` (65,536); the test asserts both, as
   `support_triage_protocol.rs:283-286` does, and prints the count.
4. Starting from no evidence and no decision, every claim is `unknown`, neither `change.keep` nor
   `change.reject` is admissible, and every outcome is `blocked`.
5. Starting from every keep claim's evidence passing, by `run-b`, about `h1`, with
   `loop_change.keep` granted: `change.supported` is `true`, `change.keep` is admissible,
   `change.reject` is not, and `kept` is `legitimate`.
6. Starting from the evidence of 5 with `loop_change.keep` not granted, `change.keep` is not
   admissible.
7. Starting from the evidence of 5 produced by `run-a` instead, the records are excluded as not
   independent: the claims are `unknown`, not `false`, and neither `change.keep` nor
   `change.reject` is admissible.
8. Starting from the evidence of 5 with `candidate` moved to `h2` (a basis moved), the records are
   excluded as `revision_mismatch`: the claims are `unknown`, and neither `change.keep` nor
   `change.reject` is admissible.
9. Starting from the evidence of 5 with `heldout_comparison` `regressed`, `heldout.not_regressed`
   is `false`, `change.reject` is admissible, `change.keep` is not, and `rejected` is `legitimate`;
   the same holds with `guard_check` `leaked` (`content.clean` `false`) and with
   `tuning_comparison` `not_better`.
10. Starting from no evidence but a `heldout_comparison` `regressed` by `run-b`, `change.reject`
    is not admissible: the other keep claims are `unknown`.
11. Starting from `guard_check` `outside` by `run-b`, `change.within_surface` is `false`,
    `change.propose` is admissible, `change.keep` is not, and `proposed` is `legitimate`;
    `change.apply_outside_surface` is admissible in no fixture state.
12. Starting from 4, an `explicitly_inconclusive` decision for the current case revision makes
    `inconclusive` `legitimate`; the same decision for an earlier case revision leaves it
    `blocked`.
13. In no state of the fixtures are two of `kept`, `rejected` and `proposed` `legitimate`
    together.

## Source

A consumer's neutral proposal for `eval.gated_change/1`;
`architecture-decision-record:eval-gated-change-shape`;
`architecture-decision-record:ml-protocol-shape`; `protocols/ml-experiment/1.yaml`;
`protocols/software-change/1.yaml`; Canon 0.6.0 `crates/canon/src/check/mod.rs`.
