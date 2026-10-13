---
format: aep.planning-md/3
id: blocker:graph-schema-evidence-order
kind: blocker
status: cleared
title: The protocol graph schema has no evidence-order predicate leaves
relations:
- blocks: story:evidence-order-red-first
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-13T03:33:31Z", actor: "human:timo", revision: 3}
---
## What is missing

The dependency graph on every protocol page is a `b10x-protocol-graph/1` document, the docs-system
schema `schema/b10x.protocol-graph.v1.schema.json`, pinned at docs-system commit
`de3e7d84b463d1b6b26e30c28efc07426e8cf012` (`website/package.json:18`). Its predicate leaves are
closed: `all`, `any`, `not`, `claim`, `evidence`, `input`. It has no leaf for Canon 0.3.0's order
forms `{first: {kind, result, subject?}}` and `{before: {first, then, subject?}}`; neither the pin
nor docs-system `main` mentions either key (read through `gh api` on 2026-10-13).

## What it stops

`story:evidence-order-red-first` puts both forms into `software.change/1`. The generator refuses
to write a graph the schema rejects rather than drop the condition, so `task check` cannot pass on
that story until the schema carries both leaves.

## Clears when

A docs-system commit adds the leaves `{"first": {"kind", "result", "subject"?}}` and
`{"before": {"first", "then", "subject"?}}` to `b10x-protocol-graph/1`, its `ProtocolGraph`
renders them, and `website/package.json` here pins that commit.

## Cleared

Cleared 2026-10-13 by docs-system 0.10.0, commit `48afe6a1d55434f393baaeb769ad7f9bdb284b02` (https://github.com/beyond10x/docs-system/releases/tag/0.10.0). Its `schema/b10x.protocol-graph.v1.schema.json` adds the predicate leaves `{"first": {"kind", "result", "subject"?}}` and `{"before": {"first", "then", "subject"?}}`. `website/package.json` pins that commit from `build: pin docs-system 0.10.0` (031ada6) on, which is this blocker's clearing condition; the generator draws both leaves from `feat(docs): the protocol graph draws the order forms` (4ae8e59).
