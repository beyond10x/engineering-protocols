# Wave 2026-11-21-w3: the frozen assertion crates, and the generator reads files

Wave 3 of 5 of the [data-only catalog plan](data-only-catalog.md). Skill `aep:implementing`
0.26.0, wave mode. Width two.

Integration branch `wave/2026-11-21-w3`, from `main` at `9fb737c`. Opened without a further
question on the `sdc` session's standing decision of 2026-11-21, recorded in
`approval-record:data-only-catalog`; the `sdc` session decides for the operator here
(`approval-record:sdc-orchestrates-engineering-protocols-2026-11-21`).

Commits this wave makes: the opening store commit, each unit's commits and its merge into the
integration branch, adversary-pass commits, the closing store commit, and one bot pull request to
`main` with its merge. No release, no tag, no version number.

Check: `aep plan artifact waves --kind story --status draft` (aep 0.80.0) places the two units in
one wave and reports no collision between them. `story:assertions-frozen-at-0-16-0` is cited for
`README.md` and for `Cargo.toml` and `README.md` of the two assertion crates;
`story:docs-generator-reads-files` is cited for the root `Cargo.toml`, `Cargo.lock`,
`crates/canon-engineering-docs`, three files of `crates/canon-engineering`, and inferred for the
new `crates/engineering-vocabulary`. The verb also places `story:ess-conformance-evidence` in the
same wave; it is not of this plan and is not taken.

Pre-flight on 2026-11-21: `/` 94 GB free; one tree's `target/debug` measured 7.7 GB in wave 1.

Each unit builds in its own managed tree's `target/` (AGENTS.md § Work); scratch is under the
coordinating session's scratch root, in a directory named after the unit.

| Story | Managed tree | Branch | Agent | Stage | Head | Merge |
|---|---|---|---|---|---|---|
| `story:assertions-frozen-at-0-16-0` | `ep-unit-assertions-frozen` | `impl/assertions-frozen-at-0-16-0` | `aep:implementor` (Sonnet), then `aep:adversary` (Opus) | dispatched | | |
| `story:docs-generator-reads-files` | `ep-unit-docs-generator-reads-files` | `impl/docs-generator-reads-files` | `aep:implementor` (Sonnet), then `aep:adversary` (Opus) | dispatched | | |

Coordinator-owned files: `AGENTS.md`, `CHANGELOG.md`, the store, `docs/plan/`. `README.md` is the
first unit's for § Assertion gates and the coordinator's elsewhere.

## Decided for the units before dispatch

In each story, section "Decided before dispatch": for the frozen crates, that the READMEs name
Loom as the owner of the replacement (the decision-blocker cleared after the story was written)
and how the public items are compared; for the generator, the new crate's name and what stays in
the registry crate.

## Result

Not closed yet.
