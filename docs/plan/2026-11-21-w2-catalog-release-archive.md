# Wave 2026-11-21-w2: the catalog archive

Wave 2 of 5 of the [data-only catalog plan](data-only-catalog.md). Skill `aep:implementing`
0.26.0, wave mode. Width one.

Integration branch `wave/2026-11-21-w2`, from `main` at `1255faf`. Opened without a further
question on the `sdc` session's standing decision of 2026-11-21, recorded in
`approval-record:data-only-catalog`; the `sdc` session decides for the operator here
(`approval-record:sdc-orchestrates-engineering-protocols-2026-11-21`).

Commits this wave makes: the opening store commit, the unit's commits and its merge into the
integration branch, adversary-pass commits, the closing store commit, and one bot pull request to
`main` with its merge. No release, no tag, no version number.

Check: `aep plan artifact waves --kind story --status active` (aep 0.80.0) places the one unit
alone. Its scope is cited for `AGENTS.md`, `crates/canon-engineering-docs` and `Cargo.lock`, and
inferred for `Taskfile.yml`. The plan's next wave holds two stories the verb would place beside
this one; the plan's order stands and none is added.

Pre-flight on 2026-11-21: `/` 77 GB free; one tree's `target/debug` measured 7.7 GB in wave 1.

The unit builds in its own managed tree's `target/` (AGENTS.md § Work); scratch is under the
coordinating session's scratch root, in a directory named after the unit.

| Story | Managed tree | Branch | Agent | Stage | Head | Merge |
|---|---|---|---|---|---|---|
| `story:catalog-release-archive` | `ep-unit-catalog-release-archive` | `impl/catalog-release-archive` | `aep:implementor` (Sonnet), then `aep:adversary` (Opus) | dispatched | | |

Coordinator-owned files: `AGENTS.md`, `README.md`, `CHANGELOG.md`, `Taskfile.yml`, the store,
`docs/plan/`.

## Decided for the unit before dispatch

In the story, section "Decided before dispatch": the archive's name and layout, the Rust command
that builds it, and what reproducible means.

## Result

Not closed yet.
