---
format: aep.planning-md/3
id: story:canon-0-6-0-pin
kind: story
status: implemented
title: Pin Canon 0.6.0
summary: Move every Canon dependency to tag 0.6.0, so a consumer on Canon 0.6.0 resolves one b10x-canon.
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
- confidence: cited
  path: crates/canon-engineering/Cargo.toml
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-10-16T16:44:19Z", actor: "human:timo", revision: 7}
- {from: "proposed", to: "active", at: "2026-10-16T16:44:19Z", actor: "human:timo", revision: 8}
- {from: "active", to: "implemented", at: "2026-10-16T17:15:38Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Every Canon dependency moves from tag 0.4.0 to tag 0.6.0 (`b10x-canon` in `crates/canon-engineering/Cargo.toml` and `crates/canon-engineering-docs/Cargo.toml`, `b10x-canon-expr` in `crates/canon-engineering-assertions/Cargo.toml`), and `Cargo.lock` records commit efcfd040. A consumer that depends on this crate and on Canon 0.6.0 resolves exactly one `b10x-canon`. Canon 0.5.0 adds optional action argument schemas and Canon 0.6.0 a requirements catalogue for its own conformance run; the protocols here declare neither and evaluate as before.

## Scope

- `crates/canon-engineering/Cargo.toml`
- `crates/canon-engineering-docs/Cargo.toml`
- `crates/canon-engineering-assertions/Cargo.toml`
- `Cargo.lock`

## Protocol first

No behaviour change: exempt. The existing suite is the check.

## Acceptance

1. `Cargo.lock` names tag 0.6.0, commit efcfd040, for `b10x-canon` and `b10x-canon-expr`, and no other `b10x-canon` entry.
2. `task check` passes with every test it passed on Canon 0.4.0, and generated documentation shows no drift.
