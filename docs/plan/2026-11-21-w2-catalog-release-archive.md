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
| `story:catalog-release-archive` | `ep-unit-catalog-release-archive` | `impl/catalog-release-archive` | `aep:implementor` (Sonnet), then `aep:adversary` (Opus) | merged | `d0bf3cb` | `7e0e583` |

Coordinator-owned files: `AGENTS.md`, `README.md`, `CHANGELOG.md`, the store,
`docs/plan/`. `Taskfile.yml` went to the unit: its one new target runs the unit's command.

## Decided for the unit before dispatch

In the story, section "Decided before dispatch": the archive's name and layout, the Rust command
that builds it, and what reproducible means.

## Result

Delivered on 2026-11-21. No release, no tag, no version number: `CHANGELOG.md` carries the change
under `[Unreleased]`, and no release has the two assets yet.

- `story:catalog-release-archive`: `canon-engineering-docs catalog-archive --out <dir>
  [--root <dir>]` and `task catalog-archive` (into `target/catalog/`) write
  `engineering-protocols-catalog-<version>.tar.gz` and its `.sha256`. At 0.16.0 the archive has
  13 entries and sha256 `73bacc79762696ea1e384a2054d57ea3dcf26e48c73a3fa880c98d1e379ef004`. Red
  run before the command existed: 0 passed, 5 failed.
- Coordinator's check of the green claim on `1edaaf0`: two runs into two directories gave
  identical bytes, `sha256sum -c` accepted the digest file, and the base `8be96b3` has no such
  command.
- Adversary pass 1 on `1edaaf0`: 4 findings, 2 of them red cases. The command packaged whatever
  file `schema` named and accepted `protocols/./x`, both only on a manifest `task check` refuses;
  three guards had no test; the version in the file name came from the binary. Correction
  `8745e6f`: the command refuses what the manifest check's path rules refuse and an unlisted file
  under `protocols/`, and reads the version from the root's `Cargo.toml`.
- Adversary pass 2 on `8745e6f`: carried 0, new 1, resolved 4 (`aep plan artifact findings`).
  The new one: five of the correction's refusals had no test that fails without them. Its seven
  cases went in as `d0bf3cb`; the coordinator ran the three archive test files on it (11, 6 and 7
  passed, 0 failed). No third pass.
- Coordinator's calls: the command mirrors the manifest check's path rules and refuses an
  unlisted file under `protocols/`, so the archive is everything under `protocols/` plus the
  schema; id, revision and `canon` stay with `task check`; the version comes from the root. In
  `AGENTS.md`: § Releases step 5, the two assets as § Release completion's required artifacts,
  the archive's row in § Invariants, and in § Work that a release asset goes through
  `b10x-gates gh -- release upload`, since `b10x-gates api` sends JSON bodies only.
- Not measured: `b10x-gates gh -- release upload` has not been run for this repository (the
  gates source admits `release` as a command word), and whether GitHub keeps a `+` in an asset
  name.
- Whole gate on the wave branch at `7e0e583` with the coordinator's `AGENTS.md` and
  `CHANGELOG.md` edits, one exit status per step: `cargo fmt --all --check` 0, clippy
  `-D warnings` 0, `cargo test --workspace --locked --no-fail-fast` 0 (472 passed, 0 failed, 0
  ignored, 104 result lines; `--list` 472), `generate --check` 0 (35 files fresh),
  `aep plan artifact validate` 0, then `catalog-archive --out target/catalog` 0 and
  `sha256sum -c` 0. No step skipped itself. Run with `ESS_TOOLCHAIN=0.77.0`.
- Backlog drafts filed from findings: none.
- Cost, sub-agent tokens as the harness reported them at each stop: implementor 60,688 then
  111,701 (two rounds), adversary 95,670 then 145,983 (two passes).
