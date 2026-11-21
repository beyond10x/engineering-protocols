---
format: aep.planning-md/3
id: story:assertion-catalog-path
kind: story
status: draft
title: The assertion function catalog leaves catalog/ for a path that names assertions
summary: catalog/engineering.yaml is canon-catalog/1 for assertions, not the protocol catalog; it moves, bytes unchanged.
relations:
- decomposes: epic:data-only-catalog
scope:
- confidence: cited
  path: catalog
- confidence: cited
  path: crates/canon-engineering-assertions/catalog.yaml
- confidence: cited
  path: crates/canon-engineering-assertions/src/lib.rs
- confidence: cited
  path: website/docs/guides/assertions.md
revision: 4
---
## Outcome

No file outside `protocols/` is called a catalog of this repository. `catalog/engineering.yaml`,
the assertion function catalog (`canon-catalog/1`, 26 functions), moves to a path that names
assertions, so "the catalog" means the protocol files and their manifest.

## Context

`catalog/` holds one file, `engineering.yaml`. It is read at build time by
`crates/canon-engineering-assertions/src/lib.rs:26`
(`BUILTIN_CATALOG = include_str!("../../../catalog/engineering.yaml")`) and named in
`website/docs/guides/assertions.md:6`. Two planning artifacts cite the path:
epic:gates-and-delivery-for-any-repository and story:a-gate-sees-the-change-and-may-not-apply.

## Decided here

- The new path is `crates/canon-engineering-assertions/catalog.yaml`: the file sits in the one
  crate that reads it, and that crate's name says what the catalog is of.
- story:assertions-frozen-at-0-16-0 freezes that crate and depends on this story. This move
  changes one `include_str!` path and no behaviour, and lands before the freeze.

## Protocol first

Exempt: no protocol YAML, no fixture verdict and no catalog content changes.

## Scope

- `catalog/engineering.yaml` (moved), `catalog/` (removed)
- `crates/canon-engineering-assertions/catalog.yaml` (new path),
  `crates/canon-engineering-assertions/src/lib.rs:26`
- `website/docs/guides/assertions.md:6`

## Acceptance

The directory `catalog/` does not exist; `crates/canon-engineering-assertions/catalog.yaml`
exists and has the sha256 `catalog/engineering.yaml` has at tag 0.16.0; `task check` is green;
`git grep -n "catalog/engineering.yaml"` outside `.engineering/` and CHANGELOG.md history prints
nothing.
