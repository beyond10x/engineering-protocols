# Wave 2026-10-26-w1: ESS 0.69.2 and takt's protocols (Atlas ADR 0094)

Integration branch `wave/2026-10-26-w1`, from `main` at `eb31336`. Run without a stage-1 stop:
`approval-record:wave-2026-10-26-w1`. Width two; `aep plan artifact waves` (aep 0.77.1) puts the
two units in disjoint sets once `AGENTS.md` and `README.md` are the coordinator's.

Commits this wave makes: the opening store commit, each unit's commits and their merges into the
integration branch, adversary-pass commits, the closing store commit, the release commit, one bot
pull request to `main`, the tag and the GitHub Release.

Each unit builds in its own managed tree's `target/` (AGENTS.md § Work); scratch is under the
session's scratch root, one directory per unit.

| Story | Managed tree | Branch | Stage | Head | Merge |
|---|---|---|---|---|---|
| `story:ess-0-69-2-pin` | `ep-unit-ess-pin` | `impl/ess-0-69-2-pin` | merged; adversary pass 1 green (`3f44d07`) | `3f44d07` | `35cdcbe` |
| `story:takt-protocols-builtin` | `ep-unit-takt-protocols` | `impl/takt-protocols-builtin` | adversary pass 1 (`aep:adversary`, opus) | `7d960a2` | |

Coordinator-owned files: `AGENTS.md`, `README.md`, `CHANGELOG.md`, `Cargo.toml`, `Cargo.lock`, the
store.
