---
format: aep.planning-md/3
id: review-result:data-only-catalog-parallel-safety-r2
kind: review-result
status: active
title: data-only-catalog decomposition, parallel-safety critic, round 2
relations:
- reviews: story:catalog-manifest
- reviews: story:catalog-release-archive
- reviews: story:assertion-catalog-path
- reviews: story:assertions-frozen-at-0-16-0
- reviews: story:docs-generator-reads-files
- reviews: story:agents-boundary-data-only
- reviews: story:protocol-check-tool
revision: 1
---
needs-revision
story:catalog-manifest — its acceptance needs a sha256 digest and a read of the `Cargo.lock` Canon pin from a test in `crates/canon-engineering/tests`, so it must edit `crates/canon-engineering/Cargo.toml` (dev-dependencies) and `Cargo.lock`. Its Scope lists neither, and story:docs-generator-reads-files, which lists both, is in the same wave 1 with no edge or note between them (surface inferred) — `aep plan artifact show story:catalog-manifest` (Scope and Acceptance), `crates/canon-engineering/Cargo.toml:16-25` (no `sha2` or TOML reader today), `aep plan artifact waves --kind story --status draft` (wave 1 holds both)

**What you read.** All 7 stories, with `aep plan artifact show` on each, `aep plan artifact waves --kind story --status draft` and `aep plan artifact relations`. I checked `git grep` for `catalog/engineering.yaml` and `sha2`, and read `crates/canon-engineering/Cargo.toml`. Surface counts:
- Cited: 7.
- Inferred: 0 whole items. Some individual scope entries are inferred, including `protocols/manifest.yaml`, `Taskfile.yml`, `.github/workflows/check.yml` and `crates/engineering-vocabulary`.
- Unplaceable: 0.

Acceptance reads traced to a producer in the set: 9.
- `catalog-release-archive` reads the manifest.
- `agents-boundary-data-only` reads the manifest, the archive and the freeze.
- `assertions-frozen-at-0-16-0` reads the moved catalog path.
- `protocol-check-tool` reads the vocabulary crate, the freeze and the manifest test and row.
- `catalog-release-archive` reads the manifest again, through its unpack test.
- `protocol-check-tool` also reads the archive's Taskfile target, transitively.

A `depends_on` edge records all 9. The remaining shared-file collisions are named by edges: AGENTS.md, README.md, `build.rs`, `crates/canon-engineering/tests`, `Taskfile.yml`, `Cargo.toml`, `Cargo.lock`, `src/lib.rs` and `src/vocabulary.rs`. Some are ordered transitively, for example `protocol-check-tool` after `catalog-release-archive` through `agents-boundary-data-only`. The pre-existing stories in the 25 reported collisions are outside this set.

**What I could not establish.**
- Where the archive's unpack test (`story:catalog-release-archive`) lives. Its Scope names only `Taskfile.yml` and AGENTS.md. It is ordered after the manifest story either way, so it is not a collision.
- Where the manifest schema lives. The body marks it UNMAPPED. If it lands in `ess/`, that is a new surface, but no other story in the set touches `ess/`.
- Out of my lane, not counted toward the verdict: `story:protocol-check-tool` names `crates/canon-engineering-assertions` in its body but not in its typed scope. That is a scope-field accuracy matter for the scope or design critic.

```findings
- file: "aep plan artifact show story:catalog-manifest"
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "story:catalog-manifest's acceptance needs a sha256 digest and a Cargo.lock Canon-pin read from a test in crates/canon-engineering/tests, so it must edit crates/canon-engineering/Cargo.toml and Cargo.lock (inferred surface: neither is in its Scope, and the crate has no sha2 or TOML reader today), and story:docs-generator-reads-files lists both files and sits in the same wave 1 with no depends_on edge or body note between them; add an ordering edge recording the shared files, or split the surface so the manifest check does not touch them"
```
