---
format: aep.planning-md/3
id: story:canon-0-4-0-pin
kind: story
status: active
title: Pin Canon 0.4.0
summary: Move every Canon dependency to tag 0.4.0, which adds independence dimensions and independent_on.
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
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-13T13:41:44Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-13T13:41:45Z", actor: "human:timo", revision: 4}
---
## Outcome

Every Canon dependency moves from tag 0.3.0 to tag 0.4.0 (`b10x-canon` in `crates/canon-engineering/Cargo.toml` and `crates/canon-engineering-docs/Cargo.toml`, `b10x-canon-expr` in `crates/canon-engineering-assertions/Cargo.toml`), and `Cargo.lock` records commit 7c2cec13. Canon 0.4.0 adds independence dimensions, `independent_on`, `producer` and the case `independence` map; protocols without them evaluate as before.

## Scope

- `crates/canon-engineering/Cargo.toml`
- `crates/canon-engineering-docs/Cargo.toml`
- `crates/canon-engineering-assertions/Cargo.toml`
- `Cargo.lock`

## Protocol first

No behaviour change: exempt. The existing suite is the check.

## Acceptance

1. `cargo tree -p b10x-canon-engineering` resolves `b10x-canon` from tag 0.4.0, and `Cargo.lock` names 7c2cec13 for `b10x-canon` and `b10x-canon-expr`.
2. `cargo test --workspace --locked` passes with every test it passed on 0.3.0.
