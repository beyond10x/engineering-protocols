---
format: aep.planning-md/3
id: blocker:graph-schema-evidence-after
kind: blocker
status: cleared
title: The protocol graph schema has no after order-form leaf
summary: b10x-protocol-graph/1 has first and before leaves only, so story.delivery/2 with Canon 0.7.0's after cannot be drawn
relations:
- blocks: story:story-delivery-2-red-gate-needs-red-tests
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-29T01:28:29Z", actor: "human:timo", revision: 3}
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

## Cleared

Cleared 2026-10-29 by docs-system 0.13.0, commit `7bbb9c2ce9beb9fc8ee5b6c8bff5770f1f9961bd`
(https://github.com/beyond10x/docs-system/pull/39). Its `schema/b10x.protocol-graph.v1.schema.json`
adds the predicate leaf `{"after": {"first", "then": {"kind", "result"}, "subject"?}}`, and
`ProtocolGraph` validates and reads it. `website/package.json` pins that commit, which is this
blocker's clearing condition.
