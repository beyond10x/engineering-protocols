---
format: aep.planning-md/3
id: architecture-decision-record:eval-gated-change-shape
kind: architecture-decision-record
status: proposed
title: An eval-gated change is its own protocol, eval.gated_change/1
relations:
- decides: story:eval-gated-change-protocol
revision: 1
---
## Decision

Proposed, not accepted: the operator accepts or rejects it.

An eval-gated change (an automated loop that tunes an agent's text or configuration against an
evaluation set and keeps a change only on independent evidence about that exact change) is its own
protocol, `eval.gated_change/1`, not a profile of `ml.experiment/1` and not a profile of
`software.change/1`. It would live at `protocols/eval-gated-change/1.yaml`, header
`id: eval.gated_change`, `revision: 1`, with fixtures under `fixtures/eval-gated-change/` and its
new terms in `protocols/vocabulary.yaml`. It reuses the idioms of both neighbours as shared
vocabulary terms, not by hosting.

This record builds nothing. `story:eval-gated-change-protocol` builds the protocol and carries its
own § Protocol first.

Sources: this repository at `dc231c7`; Canon at tag 0.6.0 (`efcfd04`), paths prefixed `canon:`;
a consumer's neutral proposal for `eval.gated_change/1` (not in this repository), cited as "the
proposal". The repository pins Canon 0.6.0 (`crates/canon-engineering/Cargo.toml:18`), the
release the proposal's model was validated, compiled and checked with.

## Why its own protocol

The test is the one `architecture-decision-record:ml-protocol-shape` applied: a distinct protocol
when the semantic graph or outcomes differ materially.

1. **Its outcomes and graph differ from `ml.experiment/1`.** That protocol closes a hypothesis
   case `supported`, `refuted`, `invalid_experiment`, `inconclusive` or `superseded`
   (`protocols/ml-experiment/1.yaml:93-121`) and declares no actions and no case inputs
   (`:1-136`). An eval-gated change decides a write on one candidate change: actions
   `change.keep`, `change.reject` and `change.propose`, `change.keep` gated on a capability, and
   outcomes `kept`, `rejected`, `proposed` and `inconclusive`. Its graph adds what the proposal
   says `ml.experiment/1` lacks: a held-out split, a content and leakage check, and a surface the
   loop may change. A profile would have to add actions, capabilities and three outcomes to a
   shipped protocol for every one of its consumers.
2. **Its subject and outcomes differ from `software.change/1`.** That protocol's artifacts are an
   intent, a specification, a plan, an implementation, a release and a deployment
   (`protocols/software-change/1.yaml:42-54`), and `accepted` needs a proven release, a healthy
   deployment and a realized objective (`:276-285`). A tuned prompt or tool cap is kept, not
   released or deployed, and none of those claims applies to it.
3. **A profile is a case input of one compiled protocol, and Canon checks every profile at once.**
   Profiles here are case inputs (`protocols/software-change/1.yaml:17-34`; AGENTS.md § Invariants,
   "A profile is a case input, not a protocol"). `canon check` takes every value of every case
   input in one state space (`canon:crates/canon/src/check/mod.rs:33-35`), and refuses a space of
   more than 65,536 states (`:147-152`, `STATE_BOUND` at `:234`). The proposal's own protocol
   checks at 55,296 states, 84% of the bound; its evidence and capability dimensions multiply
   into either host's space rather than adding to it (`:10-13`, `:24-31`), so as a profile it
   could not stay under the bound and the host could no longer be checked.
4. **Composition is still not available.** Canon 0.6.0's `protocol/1` declares artifacts,
   evidence kinds, claims, obligations, actions, outcomes, invalidation rules, case inputs and
   independence dimensions, with `deny_unknown_fields` and no import section
   (`canon:crates/canon/src/model/mod.rs:68-97`). Extending `ml.experiment/1` instead of copying
   it is the proposal's Canon request 2; it is a reason to revisit, not a way to build /1.

## What it reuses

| idiom | from | how |
|---|---|---|
| `experiment.valid` on a passed `design_review` | `protocols/ml-experiment/1.yaml:47-60`; vocabulary `protocols/vocabulary.yaml:200`, `:220` | same terms, one vocabulary entry each |
| `result.reproducible` | `protocols/ml-experiment/1.yaml:61-67`; `protocols/vocabulary.yaml:224` | same term |
| candidate against baseline, `candidate` artifact | `protocols/ml-experiment/1.yaml:18-21`; `protocols/vocabulary.yaml:188` | same term; its vocabulary description must hold for a candidate whose revision hashes its basis |
| `independent_on` an independence dimension | `protocols/software-change/1.yaml:36-40`, `:103-112` (`security.reviewed`) | every claim on the candidate demands `different_agent_run` (`:39-40`) |
| a write action with a precondition and a capability | `protocols/software-change/1.yaml:256-265` (`repository.merge`) | `change.keep`: precondition `change.supported` TRUE, capability `loop_change.keep`. `software.change/1` has no action named keep; `repository.merge` is the idiom reused |
| decision outcome `inconclusive` on `explicitly_inconclusive` | `protocols/ml-experiment/1.yaml:114-117`; `protocols/vocabulary.yaml:252`, `:256` | same terms |

## How it stays under `canon check`'s bound

Evidence binds to one subject revision. The proposal makes the `candidate` revision hash its
content together with its basis (baseline, question set, data revision, design), so a moved basis
is a new candidate revision and older records are excluded as `revision_mismatch`. That replaces
four upstream artifacts with invalidation rules. With them, each evidence dimension takes
`n^(c * 2^m)` values for `m` upstream artifacts (`canon:crates/canon/src/check/mod.rs:46-48`,
`:67-69`), and the proposal reports more than 2^128 states. With revision binding alone, `m = 0`
and the protocol checks at 55,296 states with no findings. The cost: an explanation says the
evidence is about another candidate revision, not which basis moved, and every claim goes
`UNKNOWN` when any part of the basis moves.

## Rejected

| option | reason |
|---|---|
| profile of `ml.experiment/1` | outcomes and graph differ (reason 1); a profile shares the host's state space (reason 3); no imports (reason 4) |
| profile of `software.change/1` | subject and outcomes differ (reason 2); state space (reason 3) |
| four upstream artifacts with invalidation rules | more than 2^128 states, refused by `canon check` (`canon:crates/canon/src/check/mod.rs:147-152`) |

## Recommendation

Accept: ship `eval.gated_change/1` as its own protocol, bound to a candidate revision that hashes
its basis, reusing `ml.experiment/1`'s claim terms and `software.change/1`'s capability-gated
write action. Revisit as a profile or extension only when Canon gains imports.
