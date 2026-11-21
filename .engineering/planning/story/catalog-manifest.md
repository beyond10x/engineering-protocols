---
format: aep.planning-md/3
id: story:catalog-manifest
kind: story
status: implemented
title: A manifest lists every catalog file with its id, revision, sha256 and lowest Canon release
summary: protocols/manifest.yaml, checked against the files in task check.
relations:
- decomposes: epic:data-only-catalog
- serves: vision:O2
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: crates/canon-engineering/Cargo.toml
- confidence: cited
  path: crates/canon-engineering/tests
- confidence: cited
  path: protocols/manifest.yaml
- confidence: cited
  path: schema/b10x.protocol-catalog.v1.schema.json
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T02:36:45Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-21T02:36:45Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-21T04:36:04Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":14,"verification":1}}}
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

## Scope as delivered (wave 2026-11-21-w1)

From the implementor's confirmation table and the two adversary passes; the lines above are kept
as planned.

- `protocols/manifest.yaml`: as planned, with one field more than planned, `schema`, the
  repository-relative path of its schema (adversary pass 2: a reader of the manifest alone could
  not find the schema).
- Schema: `schema/b10x.protocol-catalog.v1.schema.json`, a new top-level directory. The two
  UNMAPPED points of `## Model` were settled by the coordinator before dispatch: format name
  `b10x-protocol-catalog/1`, JSON Schema, no `$id` because nothing serves the file at a URL.
- `crates/canon-engineering/tests/catalog_manifest.rs`: 26 tests. Besides the five acceptance
  cases the check refuses an unknown field, a wrong format, a wrong or missing schema, a path
  listed twice, a protocol path outside the build's name/major rule, a vocabulary entry for
  another file, a `canon` that is not MAJOR.MINOR.PATCH, a Cargo.lock that pins no Canon, and a
  file Canon does not parse.
- `crates/canon-engineering/Cargo.toml`, `Cargo.lock`: confirmed. Dev-dependencies only: `sha2`
  (already locked) and `toml`, which adds six packages to `Cargo.lock`.
- `crates/canon-engineering/build.rs`: corrected, no change was needed. It skips anything
  directly under `protocols/` that is not a directory, as it does for `vocabulary.yaml`;
  `canon-engineering-docs` does the same in `generate.rs::sources`.
- AGENTS.md: one row in § Invariants and one rule in § Work (a change under `protocols/` updates
  the manifest in the same commit).
- `canon` values were measured: a probe ran Canon's parse, validate and compile from each tag
  0.1.0 to 0.9.0 over every file. 0.8.0 `story.delivery/2`; 0.4.0 `eval.gated_change/1`,
  `incident.response/1`, `loop.improvement/1`, `protocol.adopt/1`, `self.improvement/1`,
  `software.change/1`; 0.1.0 `ml.experiment/1`, `story.delivery/1`, `support.triage/1`.

## Protocol first, as run

Red on commit `7f5240b`: `cargo test -p b10x-canon-engineering --test catalog_manifest` ended
`test result: FAILED. 0 passed; 14 failed`, 13 on the missing manifest and 1 on the missing
schema. Green from `5b11148`.
