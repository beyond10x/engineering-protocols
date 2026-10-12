---
format: aep.planning-md/3
id: story:canon-0-2-0-pin
kind: story
status: active
title: Depend on Canon 0.2.0 and render case inputs
relations:
- decomposes: epic:els-first-domain
- serves: vision:O2
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/canon-engineering-assertions/Cargo.toml
- confidence: cited
  path: crates/canon-engineering-docs/Cargo.toml
- confidence: inferred
  path: crates/canon-engineering-docs/src/generate.rs
- confidence: cited
  path: crates/canon-engineering-docs/src/graph.rs
- confidence: cited
  path: crates/canon-engineering-docs/src/protocol.rs
- confidence: inferred
  path: crates/canon-engineering-docs/tests/
- confidence: cited
  path: crates/canon-engineering/Cargo.toml
- confidence: cited
  path: crates/canon-engineering/tests/adversary_software_change_protocol.rs
- confidence: cited
  path: crates/canon-engineering/tests/incident_response_protocol.rs
- confidence: cited
  path: crates/canon-engineering/tests/software_change_protocol.rs
- confidence: cited
  path: crates/canon-engineering/tests/support_triage_protocol.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-12T21:33:49Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-10-12T21:36:06Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

The workspace depends on Canon 0.2.0 (`b10x-canon` and `b10x-canon-expr` at tag `0.2.0`, which peels to `dbb04ea`; `Cargo.lock` records it), and everything in it that matches on Canon's `Predicate` handles the `input` leaf Canon 0.2.0 adds. The documentation generator renders what Canon 0.2.0 adds to `protocol/1`: a protocol page lists the case inputs a `case: inputs:` section declares with their values, says when an obligation with `applicable_when` applies, and writes an `input` predicate as "input `x` is one of `a`, `b`".

The dependency graph cannot carry case inputs yet. `b10x-protocol-graph/1` is the docs-system schema `schema/b10x.protocol-graph.v1.schema.json` (pinned through `website/package.json:18`); its predicate shapes are closed (`all`, `any`, `not`, `claim`, `evidence`, transcribed in `crates/canon-engineering-docs/tests/review_graph_schema.rs:53-120`), and neither the pinned commit nor docs-system `main` at `d1542ce` mentions an input or `applicable_when` (`gh api` read of the schema, 2026-10-12). So `graph::document` refuses a protocol whose predicates hold an `input` leaf or whose obligations carry `applicable_when`, naming the schema gap, instead of writing a document the schema rejects or silently dropping the condition. No built-in declares either, so nothing generated changes. Extending the schema is docs-system's change; `story:software-change-profiles` waits on it (`blocker:graph-schema-case-inputs`).

No built-in protocol changes: none declares a case input yet, so every generated page and graph under `website/` stays byte-identical.

## Why

Moving the pin from 0.1.0 to 0.2.0 alone does not compile: Canon 0.2.0 adds `Predicate::Input`, and eight exhaustive matches fail with E0004 (`cargo clippy --workspace --all-targets --locked` on the wave tree, 2026-10-12): `crates/canon-engineering-docs/src/graph.rs:37`, `:136`, `:221`, `crates/canon-engineering-docs/src/protocol.rs:53`, and the `reached` helpers in `crates/canon-engineering/tests/software_change_protocol.rs:53`, `incident_response_protocol.rs:53`, `support_triage_protocol.rs:108`, `adversary_software_change_protocol.rs:125`. Every later story that declares a case input (`story:software-change-profiles` first) needs the pin and the rendering.

## Scope

- `crates/canon-engineering/Cargo.toml`, `crates/canon-engineering-docs/Cargo.toml`, `crates/canon-engineering-assertions/Cargo.toml`, `Cargo.lock` (cited: the three Canon tag lines)
- `crates/canon-engineering-docs/src/graph.rs`, `crates/canon-engineering-docs/src/protocol.rs` (cited: E0004 sites), and `crates/canon-engineering-docs/src/generate.rs` where it calls `graph::document` (inferred)
- `crates/canon-engineering-docs/tests/` (inferred: graph schema and page tests for the `input` leaf and case inputs)
- `crates/canon-engineering/tests/software_change_protocol.rs`, `incident_response_protocol.rs`, `support_triage_protocol.rs`, `adversary_software_change_protocol.rs` (cited: E0004 sites)
- `CHANGELOG.md` `[Unreleased]` (inferred)

## Protocol first

Exempt: no protocol changes behaviour. The generator's new rendering lands test first: the first commit adds the docs tests below and fails to compile or fails on them; the implementation commit follows.

## Acceptance

`task check` passes on the unit's head with Canon 0.2.0 in `Cargo.lock`, and:

1. Starting from a protocol source with `case: {inputs: {risk: {values: [low, high]}}}` and an obligation whose `applicable_when` is `{input: risk, in: [high]}`, the page the generator renders lists the input `risk` with values `low` and `high`, and says the obligation applies when input `risk` is one of `high`: the unit test `the_page_lists_case_inputs_and_when_an_obligation_applies` in `crates/canon-engineering-docs/src/protocol.rs` renders that source and asserts both lines.
2. Starting from the same source, `graph::document` returns an error naming `b10x-protocol-graph/1` and the case input, and writes no document: the unit test `the_graph_refuses_case_inputs_the_schema_cannot_carry` in `crates/canon-engineering-docs/src/graph.rs`. The same source without the obligation's `applicable_when` but with an action precondition `{input: risk, in: [low]}` is refused the same way.
3. Starting from the built-ins as they are, `canon-engineering-docs generate --check` reports no drift.
4. Starting from the workspace manifests, every Canon dependency names tag `0.2.0` and `Cargo.lock` records `dbb04ea95e9a53bbd0e9c1a24d73545f150e1dfb` for `b10x-canon` and `b10x-canon-expr`.

## Source

`blocker:canon-case-inputs`; Canon 0.2.0 CHANGELOG "Add case inputs to `protocol/1`".
