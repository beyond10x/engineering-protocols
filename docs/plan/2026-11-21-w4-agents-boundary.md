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
| `story:agents-boundary-data-only` | `ep-unit-agents-boundary` | `impl/agents-boundary-data-only` | `aep:implementor` (Sonnet), then `aep:adversary` (Opus) | merged | `43027cb` | `65ab3ff` |

Coordinator-owned files: `CHANGELOG.md`, the store, `docs/plan/`. `AGENTS.md` and `README.md`
are the unit's in this wave: they are the story.

## Decided for the unit before dispatch

In the story, section "Decided before dispatch": how the README's commands are measured while no
published release has a manifest or an archive, which tools a reader needs, what the README says
about the registry crate until `story:protocol-check-tool` lands, and the stale path of the
workspace `docs` skill.

## Result

Delivered on 2026-11-21. No release, no tag, no version number: `CHANGELOG.md` carries the change
under `[Unreleased]`.

- `story:agents-boundary-data-only`: README.md § Take the protocols and check them replaces
  § Install and § Smallest example; AGENTS.md § Boundary, the Loom row, § ESS, § Releases and
  § Public documentation state the boundary of `approval-record:data-only-catalog`. The
  acceptance grep prints 9 lines on the base and none on the wave branch.
- The README's commands were extracted from its fenced blocks and run as written with
  `<version>` = 0.16.0: tag route on a copy of the tracked `protocols/` and `schema/`, exit 0;
  archive route on a locally built archive, exit 0; with one file changed and one deleted, exit
  1 naming both; a changed archive fails `sha256sum -c`. Not run: the `curl` download, since no
  release has the two files.
- Adversary pass 1 on `361cdcb`: 5 findings, every acceptance clause met. The README's `awk`
  check read no pair for a manifest entry written in another YAML form (an entry starting
  `- sha256:`, a flow-style vocabulary, a quoted digest) and then exited 0 with a changed file
  unchecked; a clone with Git for Windows' default `core.autocrlf=true` failed the check without
  naming a file; three sentences were wrong or too strong (the binary's future, what `canon`
  guarantees, ADR 0094 decision 4).
- Coordinator's call, beyond the story's scope: the gate holds the manifest to the line shape
  the README's check reads (`LineShape` in `tests/catalog_manifest.rs`, 26 → 35 tests), and
  `.gitattributes` marks `protocols/**` and `schema/**` `-text`. Correction `2cafe04`. The
  coordinator cloned it with `core.autocrlf=true`: README.md checks out with CRLF, the manifest
  with LF, the check prints 11 OK, exit 0.
- Adversary pass 2 on `2cafe04`: carried 0, new 2, resolved 5. 16 manifest variants found no text
  the gate accepts on which the README's check misses a changed file. The two new findings were
  test strength: the order of the pairs had no case, and the `.gitattributes` test only looked
  for its two lines. Correction `43027cb`; no third pass, the coordinator ran the manifest tests
  on it (35 passed).
- Not measured: the README's check with uutils or busybox `sha256sum` and with `mawk`; it was
  run with GNU coreutils 9.12, `gawk` (also `--posix`) and `shasum -a 256`.
- `upstream-blocker:consumers-compile-against-canon-engineering` gained a note: Loom's four
  crates pin tag 0.16.0 now, not 0.15.2.
- Whole gate on the wave branch at `65ab3ff` with the coordinator's `CHANGELOG.md` edit, one
  exit status per step: `cargo fmt --all --check` 0, clippy `-D warnings` 0,
  `cargo test --workspace --locked --no-fail-fast` 0 (493 passed, 0 failed, 0 ignored, 110
  result lines; `--list` 493), `generate --check` 0 (35 files fresh),
  `aep plan artifact validate` 0, `catalog-archive` 0 (sha256 unchanged, `73bacc79…f004`),
  `sha256sum -c` 0, the acceptance grep 0 lines, the README's check 11 OK. No step skipped
  itself. Run with `ESS_TOOLCHAIN=0.77.0`.
- Backlog drafts filed from findings: none.
- Cost, sub-agent tokens as the harness reported them at each stop: implementor 116,572,
  176,083, 186,281 (three rounds), adversary 120,964, 166,817 (two passes).

## What remains of the plan

Wave 5, `story:protocol-check-tool`, waits on
`upstream-blocker:consumers-compile-against-canon-engineering`: Loom's four crates still compile
against `b10x-canon-engineering`. It opens when Loom reads the catalog as files. A release that
carries the manifest and the archive is the operator's to order.
