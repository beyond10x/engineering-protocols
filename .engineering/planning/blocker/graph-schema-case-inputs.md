---
format: aep.planning-md/3
id: blocker:graph-schema-case-inputs
kind: blocker
status: open
title: The protocol graph schema has no case-input predicate and no obligation applicability
relations:
- blocks: story:software-change-profiles
revision: 1
---
## What is missing

The dependency graph on every protocol page is a `b10x-protocol-graph/1` document, and that format is the docs-system schema `schema/b10x.protocol-graph.v1.schema.json`, which this repository consumes through the pin in `website/package.json:18` and transcribes in `crates/canon-engineering-docs/tests/review_graph_schema.rs:53-120`. Its predicate shapes are closed: `all`, `any`, `not`, `claim` and `evidence`. It has no shape for Canon 0.2.0's `input` leaf (`{input: <id>, in: [<values>]}`) and no key for an obligation's `applicable_when`. Neither the pinned commit nor docs-system `main` at `d1542ce` mentions either (read through `gh api` on 2026-10-12).

## What it stops

`story:software-change-profiles`: `software.change/1` gains obligations with `applicable_when` over `risk` and an `implementation.edit` precondition that tests `change_kind`. From `story:canon-0-2-0-pin` on, the generator refuses to write a graph for such a protocol rather than write one the schema rejects or drop the condition, so `task check` cannot pass on that story until the schema carries both.

## Clears when

A docs-system commit extends `b10x-protocol-graph/1` (or a successor format) with an `input` predicate leaf and an obligation's applicability, and its `ProtocolGraph` component renders them, and `website/package.json` here pins that commit.
