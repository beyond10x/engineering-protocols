# Wave 2026-11-21-w4: AGENTS.md and README.md state the data-only boundary

Wave 4 of 5 of the [data-only catalog plan](data-only-catalog.md). Skill `aep:implementing`
0.26.0, wave mode. Width one.

Integration branch `wave/2026-11-21-w4`, from `main` at `6c9491a`. Opened without a further
question on the `sdc` session's standing decision of 2026-11-21, recorded in
`approval-record:data-only-catalog`; the `sdc` session decides for the operator here
(`approval-record:sdc-orchestrates-engineering-protocols-2026-11-21`).

Commits this wave makes: the opening store commit, the unit's commits and its merge into the
integration branch, adversary-pass commits, the closing store commit, and one bot pull request to
`main` with its merge. No release, no tag, no version number.

Check: `aep plan artifact waves --kind story --status active` (aep 0.80.0) places the one unit
alone. Its scope is cited for `AGENTS.md` and `README.md`.

Pre-flight on 2026-11-21: `/` 67 GB free; the build pool pauses below 50 GB. The unit is
documentation and builds one binary for the archive it measures; the whole gate builds once in
the integration tree.

| Story | Managed tree | Branch | Agent | Stage | Head | Merge |
|---|---|---|---|---|---|---|
| `story:agents-boundary-data-only` | `ep-unit-agents-boundary` | `impl/agents-boundary-data-only` | `aep:implementor` (Sonnet), then `aep:adversary` (Opus) | dispatched | | |

Coordinator-owned files: `CHANGELOG.md`, the store, `docs/plan/`. `AGENTS.md` and `README.md`
are the unit's in this wave: they are the story.

## Decided for the unit before dispatch

In the story, section "Decided before dispatch": how the README's commands are measured while no
published release has a manifest or an archive, which tools a reader needs, what the README says
about the registry crate until `story:protocol-check-tool` lands, and the stale path of the
workspace `docs` skill.

## Result

Not closed yet.
