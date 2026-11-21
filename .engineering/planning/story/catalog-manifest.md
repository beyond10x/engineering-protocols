---
format: aep.planning-md/3
id: story:catalog-manifest
kind: story
status: active
title: A manifest lists every catalog file with its id, revision, sha256 and lowest Canon release
summary: protocols/manifest.yaml, checked against the files in task check.
relations:
- decomposes: epic:data-only-catalog
- serves: vision:O2
scope:
- confidence: cited
  path: AGENTS.md
- confidence: inferred
  path: Cargo.lock
- confidence: inferred
  path: crates/canon-engineering/Cargo.toml
- confidence: cited
  path: crates/canon-engineering/build.rs
- confidence: cited
  path: crates/canon-engineering/tests
- confidence: inferred
  path: protocols/manifest.yaml
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T02:36:45Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-21T02:36:45Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

`protocols/manifest.yaml` lists every file of the catalog: the 10 protocol documents and
`protocols/vocabulary.yaml`. Per protocol it gives the path, the protocol id, the revision, the
sha256 of the file's bytes and the lowest Canon release that reads the file. A consumer checks a
file it took from a tag against the manifest of that tag without compiling anything from this
repository.

## Context

Today a consumer gets a protocol by compiling `b10x-canon-engineering` and calling
`registry::get`; `crates/canon-engineering/build.rs` embeds the files. Loom's `ProtocolCatalog`
already loads protocol text and records a sha256 of its bytes
(`sdc/docs/engineering-protocols-boundary.md` § 3), so a manifest of byte digests is what it can
check against.

A protocol can need a newer Canon than the one a runtime links: `story.delivery/2` uses the
`since` form of Canon 0.8.0 (CHANGELOG.md § 0.14.0). Nothing records that per file today.

## Model

The manifest is a new document format. Settled first, in this story's first commit:

- UNMAPPED: its format name and where its schema lives (an ESS document beside the assertion wire
  model in `ess/`, or a JSON Schema). `protocols/manifest.yaml` is an inferred path.
- "Lowest Canon release" is declared by hand per file and checked in one direction only (below).
  Canon has no form for it and plans none: a protocol carries only `format: protocol/1`, every
  model struct is `deny_unknown_fields`, and a Canon release that predates a field refuses the
  document with a parse error naming the field, not with a version message (the `canon` session,
  2026-11-21, asked for this story; Canon's `story:protocol-floor` is `canon floor`, the
  organization floor of Atlas ADR 0086, and is not this). Asking Canon to carry or report a
  minimum release would be a new request to Canon and needs the operator's word in the `canon`
  session; this story does not make it.

Fields per entry, as requested: `path`, `id`, `revision`, `sha256`, `canon` (lowest release).
`vocabulary.yaml` has a path and a sha256 and no protocol id.

## Protocol first

No protocol YAML and no fixture verdict changes. The check lands first and fails: a test that
reads `protocols/manifest.yaml` fails on the commit that adds the test without the manifest, and
that run is recorded.

## Scope

- `protocols/manifest.yaml` (new) and its schema
- one test in the check crate (today `crates/canon-engineering/tests/`)
- `crates/canon-engineering/Cargo.toml` and `Cargo.lock` (inferred): the test needs a sha256 and
  a reader of `Cargo.lock`, and the crate has neither as a dependency today.
  story:docs-generator-reads-files edits both files and depends on this story for that reason
- `crates/canon-engineering/build.rs`: the manifest is not a protocol and must not become a
  built-in or fail the path rule in `src/builtin_name.rs`
- AGENTS.md § Invariants: one row

## Acceptance

`task check` fails when any of these holds, each shown by a test that mutates a scratch copy:
a file under `protocols/` is missing from the manifest; the manifest names a file that does not
exist; a listed sha256 differs from the file's bytes; a listed id or revision differs from what
Canon parses from the file; the Canon release pinned in `Cargo.lock` is lower than a listed
`canon`. An understated `canon` value is not detected, and the manifest's schema description says
so.
