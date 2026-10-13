---
format: aep.planning-md/3
id: story:protocol-pages-independence-freshness
kind: story
status: implemented
title: Protocol pages show independence demands and evidence horizons
summary: Generated protocol pages and graph qualifiers render independence_dimensions, independent_on and max_age.
relations:
- decomposes: epic:els-first-domain
- serves: vision:O2
- depends_on: story:stale-evidence-fixtures
scope:
- confidence: cited
  path: crates/canon-engineering-docs/src/graph.rs
- confidence: cited
  path: crates/canon-engineering-docs/src/protocol.rs
- confidence: cited
  path: website/data/protocol-graphs
- confidence: cited
  path: website/docs/protocols
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-13T13:41:49Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-10-13T13:41:50Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-10-13T15:42:48Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":2}}}
---
## Outcome

The generated protocol pages and graphs show what this wave's rules demand, so a reader of the site sees them:

- a protocol page lists the protocol's `independence_dimensions` with their descriptions;
- an evidence match with `independent_on` is rendered with its dimensions on the protocol page (claim, obligation, action and outcome text) and in the graph edge's qualifier (`approved, independent on different_principal`);
- an evidence kind with `max_age` is rendered with its horizon on the protocol page.

`b10x-protocol-graph/1` gains no field: the demand travels in the existing free-text edge qualifier, so no Docs System release is needed.

## Scope

- `crates/canon-engineering-docs/src/protocol.rs`
- `crates/canon-engineering-docs/src/graph.rs`
- `crates/canon-engineering-docs/tests/` (a new test file)
- `website/docs/protocols/`, `website/data/protocol-graphs/` (generated)

## Protocol first

No protocol behaviour change: exempt. The red test is the docs crate's new test, failing on the first commit.

## Acceptance

1. A unit test in the docs crate renders a protocol with `independence_dimensions`, an `independent_on` match and an evidence kind with `max_age: 30m`, and finds each dimension, the demand and the horizon on the page.
2. The graph edge for an `independent_on` match carries the dimensions in its qualifier.
3. `canon-engineering-docs generate --check` passes on the wave's protocols.
