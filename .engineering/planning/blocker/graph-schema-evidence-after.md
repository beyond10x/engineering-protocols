---
format: aep.planning-md/3
id: blocker:graph-schema-evidence-after
kind: blocker
status: open
title: The protocol graph schema has no after order-form leaf
summary: b10x-protocol-graph/1 has first and before leaves only, so story.delivery/2 with Canon 0.7.0's after cannot be drawn
relations:
- blocks: story:story-delivery-2-red-gate-needs-red-tests
revision: 1
---
## What is missing

The dependency graph on every protocol page is a `b10x-protocol-graph/1` document, the docs-system
schema `schema/b10x.protocol-graph.v1.schema.json`, pinned at docs-system commit
`5fd348a9e49a96832f70af7917e6cd2db2c32abe` (`website/package.json:18`). Its order-form leaves are
`first` and `before` (lines 104-113 of the schema at that commit). It has no leaf for Canon
0.7.0's order form `{after: {first, then: {kind, result}, subject?}}`; neither the pin nor
docs-system `main` at `b2729c0` has an `after` key (read with `git show` on 2026-10-29).

## What it stops

story:story-delivery-2-red-gate-needs-red-tests puts `after` into `story.delivery/2`'s
`implementation.edit` precondition. The generated graph `website/data/protocol-graphs/story-delivery-2.json`
then carries an `after` leaf that the schema rejects:
`every_committed_protocol_graph_conforms_to_the_pinned_schema_and_its_protocol` fails with
"story-delivery-2: $.nodes[4].predicate.any[0].all[0]: `after` is not a predicate form". The wave
stops before a pull request
rather than publish a graph its schema rejects.

## Clears when

A docs-system commit adds the leaf `{"after": {"first", "then": {"kind", "result"}, "subject"?}}`
to `b10x-protocol-graph/1`, its `ProtocolGraph` renders it, and `website/package.json` here pins
that commit.
