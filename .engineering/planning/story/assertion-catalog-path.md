---
format: aep.planning-md/3
id: story:assertion-catalog-path
kind: story
status: implemented
title: The assertion function catalog leaves catalog/ for a path that names assertions
summary: catalog/engineering.yaml is canon-catalog/1 for assertions, not the protocol catalog; it moves, bytes unchanged.
relations:
- decomposes: epic:data-only-catalog
- serves: vision:O2
scope:
- confidence: cited
  path: catalog
- confidence: cited
  path: crates/canon-engineering-assertions/catalog.yaml
- confidence: cited
  path: crates/canon-engineering-assertions/src/lib.rs
- confidence: cited
  path: crates/canon-engineering/tests/adversary2_protocol_registry.rs
- confidence: cited
  path: crates/canon-engineering/tests/adversary3_catalog_path.rs
- confidence: cited
  path: website/docs/guides/assertions.md
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T02:36:51Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-21T02:36:51Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-21T04:36:04Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1,"review_outcome":3,"verification":1}}}
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

## Scope as delivered (wave 2026-11-21-w1)

From the implementor's confirmation table and the two adversary passes; the lines above are kept
as planned.

- `catalog/engineering.yaml` moved to `crates/canon-engineering-assertions/catalog.yaml`, sha256
  `d86dca5d0673a0f464433b31b41523e0c2517c394aba5a433d156136d4ec2856` at tag 0.16.0, at the
  wave's base and after the move; `catalog/` removed: confirmed.
- `crates/canon-engineering-assertions/src/lib.rs:26`, `website/docs/guides/assertions.md:6`:
  confirmed, one line each.
- Missed by this scope and by the acceptance grep, found by adversary pass 1:
  `crates/canon-engineering/tests/adversary2_protocol_registry.rs:161` copied the directory
  `catalog/` into a scratch checkout, spelled `"catalog"`, and panicked once the directory was
  gone. The line is removed; the copy of `crates/` carries the catalog.
- Added by adversary pass 2: `crates/canon-engineering/tests/adversary3_catalog_path.rs`, which
  shows that a checkout without the moved file does not build.
- Planning artifacts that cite the old path keep it: they are the record of implemented work
  (epic:gates-and-delivery-for-any-repository, story:a-gate-sees-the-change-and-may-not-apply).
