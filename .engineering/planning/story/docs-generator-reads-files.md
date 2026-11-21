---
format: aep.planning-md/3
id: story:docs-generator-reads-files
kind: story
status: draft
title: The docs generator reads files and links Canon, without the registry crate
summary: canon-engineering-docs drops its dependency on b10x-canon-engineering; the assertion reference page is kept or dropped.
relations:
- decomposes: epic:data-only-catalog
- depends_on: story:catalog-manifest
- serves: vision:O2
scope:
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: crates/canon-engineering-docs
- confidence: cited
  path: crates/canon-engineering/Cargo.toml
- confidence: cited
  path: crates/canon-engineering/src/lib.rs
- confidence: cited
  path: crates/canon-engineering/src/vocabulary.rs
- confidence: inferred
  path: crates/engineering-vocabulary
revision: 6
---
## Outcome

`canon-engineering-docs` reads `protocols/` and `protocols/vocabulary.yaml` as files and links
Canon; it does not depend on `b10x-canon-engineering`. The vocabulary reader lives in one small
`publish = false` crate of its own. The site the generator writes is unchanged.

## Context

The generator already compiles protocols itself with `b10x_canon::{ir, model}`
(`crates/canon-engineering-docs/src/generate.rs:10-11`). It depends on the registry crate for one
thing, the vocabulary reader: `canon_engineering::vocabulary::{Category, Marking, Vocabulary}`
(`src/generate.rs:12`, `src/status.rs:7`; `Cargo.toml:13`). It depends on the assertions crate for
two: the assertion reference page (`canon_engineering_assertions::documentation::reference()`,
`src/generate.rs:238`) and the expression model it vendors (`documentation::expression_model()`,
`src/main.rs:55`; `Cargo.toml:10`).

story:protocol-check-tool deletes the registry crate and depends on this story, so the reader
cannot move into the crate that story creates.

## Decided here

- The vocabulary reader (`crates/canon-engineering/src/vocabulary.rs`, 586 lines) moves into a
  new crate created by this story (name inferred: `crates/engineering-vocabulary`,
  `publish = false`). The generator depends on it. `b10x-canon-engineering` depends on it and
  re-exports it as `canon_engineering::vocabulary`, so the registry crate's public items are
  unchanged for as long as it exists.
- The assertion reference page stays generated, and the generator keeps its dependency on
  `b10x-canon-engineering-assertions`, for as long as the frozen assertion crates are in the tree
  (story:assertions-frozen-at-0-16-0). The page goes with the later story that deletes them.

## Order

After story:catalog-manifest: both edit `crates/canon-engineering/Cargo.toml` and `Cargo.lock`.
Before story:protocol-check-tool, which deletes the crate this story takes the reader out of.

## Protocol first

Exempt: no protocol YAML and no fixture verdict changes.

## Scope

- `crates/engineering-vocabulary/` (new), `Cargo.toml` (workspace members), `Cargo.lock`
- `crates/canon-engineering-docs/Cargo.toml`, `src/generate.rs`, `src/status.rs`
- `crates/canon-engineering/src/vocabulary.rs` (moved), `src/lib.rs` (the re-export),
  `Cargo.toml`

## Acceptance

`cargo tree -p canon-engineering-docs` lists no `b10x-canon-engineering`;
`canon-engineering-docs generate --check` reports every generated file fresh with no generated
file changed against the commit before; `git grep -n "pub struct Vocabulary" -- crates` prints one
line, in the new crate; `tests/vocabulary_yaml.rs::vocabulary_yaml_is_the_source` passes.
