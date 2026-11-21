# Wave 2026-11-21-w1: the catalog manifest and the assertion catalog's path

Wave 1 of 5 of the [data-only catalog plan](data-only-catalog.md). Skill `aep:implementing`
0.26.0, wave mode.

Integration branch `wave/2026-11-21-w1`, from `main` at `479cd6b`. Approved by the `sdc` session
on 2026-11-21 ("wave 1 = story:catalog-manifest and story:assertion-catalog-path, as one wave;
start"), which decides for the operator here
(`approval-record:sdc-orchestrates-engineering-protocols-2026-11-21`).

Commits this wave makes: the opening store commit, each unit's commits and their merges into the
integration branch, adversary-pass commits, the closing store commit, and one bot pull request to
`main` with its merge. No release, no tag, no version number.

Check: `aep plan artifact waves --kind story --status active` (aep 0.80.0) puts both units in
wave 1 with no collision between them. Scope of `story:assertion-catalog-path` is cited; of
`story:catalog-manifest` cited except `protocols/manifest.yaml`,
`crates/canon-engineering/Cargo.toml` and `Cargo.lock`, which are inferred. Scoping was done at
drafting and read twice by the plan-critic panel (`review-result:data-only-catalog-*-r1`, `-r2`);
no `story-scoper` was run.

Pre-flight on 2026-11-21: `/` 112 GB free; one tree's `target/debug` measured 8.5 GB in
`story:canon-0-9-0-pin`'s tree; width two plus the integration tree.

Each unit builds in its own managed tree's `target/` (AGENTS.md § Work); scratch is under the
coordinating session's scratch root, one directory per unit, named after the unit.

| Story | Managed tree | Branch | Agent | Stage | Head | Merge |
|---|---|---|---|---|---|---|
| `story:catalog-manifest` | `ep-unit-catalog-manifest` | `impl/catalog-manifest` | `aep:implementor`, then `aep:adversary` | dispatched | | |
| `story:assertion-catalog-path` | `ep-unit-assertion-catalog-path` | `impl/assertion-catalog-path` | `aep:implementor`, then `aep:adversary` | dispatched | | |

Coordinator-owned files: `AGENTS.md`, `README.md`, `CHANGELOG.md`, the store, `docs/plan/`.

## Decided for the units before dispatch

- The manifest's format name is `b10x-protocol-catalog/1`, its file `protocols/manifest.yaml`,
  its schema `schema/b10x.protocol-catalog.v1.schema.json` (JSON Schema). These were UNMAPPED in
  `story:catalog-manifest`.
- The assertion catalog's new path is `crates/canon-engineering-assertions/catalog.yaml`, as
  `story:assertion-catalog-path` decides.

## Also in the opening commit

`story:gate-program-identity-bound-to-spawn` is archived: Loom took it as its own story of the
same id on 2026-11-21, after `decision-blocker:assertion-executor-owner` named Loom the owner.

## Result

Not closed yet.
