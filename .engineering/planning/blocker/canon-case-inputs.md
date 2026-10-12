---
format: aep.planning-md/3
id: blocker:canon-case-inputs
kind: blocker
status: cleared
title: Canon protocol/1 has no case inputs and no derived obligations
relations:
- blocks: story:software-change-profiles
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-12T21:23:07Z", actor: "human:timo", revision: 3}
---
## What is missing

Canon's `protocol/1` has no case-input section and no obligation derived by rule. The `Protocol` struct declares `artifacts`, `evidence_kinds`, `claims`, `obligations`, `actions`, `outcomes` and `invalidation` with `deny_unknown_fields`, and `Obligation` carries only a description and `discharged_when` (canon `crates/canon/src/model/mod.rs:55-75` and `:122-125`, canon checkout `main` at `761239f`, read 2026-10-07).

No artifact in the canon store mentions case inputs, and canon's one open issue, https://github.com/beyond10x/canon/issues/5, covers imports, the floor construct, `canon diff` and child case outcomes, not case inputs (read 2026-10-07).

## What it stops

`story:software-change-profiles`: a profile is the value of the case input `risk` or `change_kind`, and the risk profiles derive obligations from it. `story:ess-conformance-evidence` and `story:security-independence-rules` use case inputs too and depend on that story.

## Clears when

A Canon release declares case inputs and obligations derived from them in `protocol/1`, and this repository's Canon pin moves to it.

## Cleared by Canon 0.2.0

Canon release 0.2.0 (https://github.com/beyond10x/canon/releases/tag/0.2.0, tag `0.2.0` peeling to `dbb04ea95e9a53bbd0e9c1a24d73545f150e1dfb`, read from `Cargo.lock` after `cargo update` on 2026-10-12) ships what this blocker names:

- `case: inputs:` in `protocol/1`, a closed list of values per input (canon `crates/canon/src/model/mod.rs:94-112`, `CaseSection` and `CaseInput`);
- the `input` predicate leaf, `{input: <id>, in: [<values>]}`, usable in any predicate: claims, action preconditions, outcomes and obligations (canon `crates/canon/src/model/predicate.rs:4-7`, `:66-67`, `:91-95`);
- obligations derived from case inputs: `Obligation.applicable_when`, under which an obligation is `not_applicable` when the predicate is FALSE and applies otherwise (canon `crates/canon/src/model/mod.rs:157-164`);
- `canon-case/1` `inputs`; a case missing a declared input, naming an undeclared one or giving an undeclared value is refused (canon `CHANGELOG.md` § 0.2.0);
- conformance scenario `CANON-INPUT-001` (canon `conformance/scenarios/case-inputs.yaml`), where `peer.review` is `not_applicable` at severity `low` and open at `high`, and `publish_finding` is admissible on the input alone.

This repository's pin moves to 0.2.0 in `story:canon-0-2-0-pin`: all three Canon dependency lines name tag `0.2.0`. The pin alone does not compile, because Canon 0.2.0 adds `Predicate::Input` and eight exhaustive matches here do not cover it; that story handles them.
