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
| `story:catalog-manifest` | `ep-unit-catalog-manifest` | `impl/catalog-manifest` | `aep:implementor` (Opus), then `aep:adversary` (Opus) | implemented, merged; red commit `7f5240b`, implementation `5b11148`; adversary pass 1 (8 findings) answered in `3cc3244`, pass 2 (5 findings) in `b9b49f9` | `b9b49f9` | `7ec41c9` |
| `story:assertion-catalog-path` | `ep-unit-assertion-catalog-path` | `impl/assertion-catalog-path` | `aep:implementor` (Sonnet), then `aep:adversary` (Sonnet) | implemented, merged; `d3990a2`; adversary pass 1 red (a registry test still copied `catalog/`), corrected in `9e9a2e6`; pass 2 found nothing and added one case (`a6a92dd`) | `a6a92dd` | `1b94360` |

Rate limits (HTTP 429) stopped the `story:catalog-manifest` implementor twice and its adversary
once, each with its work saved; each was resumed, none replaced.

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

Delivered on 2026-11-21. No release, no tag, no version number: `CHANGELOG.md` carries both
changes under `[Unreleased]`.

- `story:catalog-manifest`: `protocols/manifest.yaml` (`b10x-protocol-catalog/1`) with its schema
  and `tests/catalog_manifest.rs`, 26 tests. Red run on `7f5240b` (0 passed, 14 failed, the
  manifest and schema missing). `canon` values measured by running Canon's parse, validate and
  compile from each tag 0.1.0 to 0.9.0 over every file: 0.8.0 for `story.delivery/2`, 0.4.0 for
  six protocols, 0.1.0 for three.
- Adversary on it: pass 1 found the check correct and 8 untested weakenings of it; pass 2 found
  it still correct, 4 more and a missing link from the manifest to its schema. Findings ledger
  between the passes (`aep plan artifact findings`): carried 0, new 5, resolved 8. All 13 are
  answered: 12 by cases that each turn exactly one test red on its mutant (16 mutants in all),
  one by the manifest field `schema`. There was no third pass; the coordinator read the last
  correction's diff: no assertion dropped, the check gained four lines for the schema field.
- `story:assertion-catalog-path`: `catalog/engineering.yaml` is
  `crates/canon-engineering-assertions/catalog.yaml`, sha256 unchanged from tag 0.16.0;
  `catalog/` is gone. Adversary pass 1 found the one thing the story's scope and acceptance grep
  missed, a registry test that copied the directory by its bare name; pass 2 found nothing.
  Ledger: carried 0, new 0, resolved 2.
- Coordinator's calls: the format and schema names and the `schema` field; AGENTS.md gains the
  manifest's invariant row and the rule that a change under `protocols/` updates the manifest in
  the same commit; planning artifacts that cite the old catalog path keep it.
- Whole gate on the wave branch at `7ec41c9` with the coordinator's AGENTS.md and CHANGELOG.md
  edits, one exit status per step: `cargo fmt --all --check` 0, clippy `-D warnings` 0,
  `cargo test --workspace --locked --no-fail-fast` 0 (448 passed, 0 failed, 0 ignored, 101
  result lines; `--list` 448), `generate --check` 0 (35 files fresh), `aep plan artifact
  validate` 0. No step skipped itself. Run with `ESS_TOOLCHAIN=0.77.0`.
- Backlog drafts filed from findings: none.
- Cost, sub-agent tokens as the harness reported them at each stop: `catalog-manifest`
  implementor 215,199 (three rounds), its adversary 155,155 (two passes);
  `assertion-catalog-path` implementor 49,233 (two rounds), its adversary 52,355 (two passes).
  One tree's `target/debug` measured 7.7 GB at discard.
