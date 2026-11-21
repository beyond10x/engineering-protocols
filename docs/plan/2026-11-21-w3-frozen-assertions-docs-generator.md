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
| `story:assertions-frozen-at-0-16-0` | `ep-unit-assertions-frozen` | `impl/assertions-frozen-at-0-16-0` | `aep:implementor` (Sonnet), then `aep:adversary` (Opus) | merged | `c0c19bd` | `e4628bb` |
| `story:docs-generator-reads-files` | `ep-unit-docs-generator-reads-files` | `impl/docs-generator-reads-files` | `aep:implementor` (Sonnet), then `aep:adversary` (Opus) | merged | `476c46a` | `b1f8df0` |

Coordinator-owned files: `AGENTS.md`, `CHANGELOG.md`, the store, `docs/plan/`. `README.md` is the
first unit's for § Assertion gates and the coordinator's elsewhere.

## Decided for the units before dispatch

In each story, section "Decided before dispatch": for the frozen crates, that the READMEs name
Loom as the owner of the replacement (the decision-blocker cleared after the story was written)
and how the public items are compared; for the generator, the new crate's name and what stays in
the registry crate.

## Result

Delivered on 2026-11-21. No release, no tag, no version number: `CHANGELOG.md` carries both
changes under `[Unreleased]`.

- `story:assertions-frozen-at-0-16-0`: `publish = false` in both manifests; a README for each
  crate and README.md § Assertion gates state the freeze and that Loom owns what replaces the
  crates. No source or test file of the two crates changed. `cargo public-api --simplified`
  (cargo-public-api 0.52.0, rustc 1.101.0-nightly 0abfedbc7 2026-10-02) prints the same lines at
  tag 0.16.0 and at the unit: 345 for `b10x-canon-engineering-assertions`, 338 for
  `b10x-assertion-providers`.
- Adversary on it: pass 1 on `ba94af2` found 5 things, none a red case: the status page and the
  assertion guide still listed AEP acceptance and Kubernetes providers as planned, both READMEs
  gave a wrong reason for pins moving (a shared lockfile), and the changelog had no entry. Pass 2
  on `f665bcc`: carried 0, new 2, resolved 4 (the changelog entry is the coordinator's and not
  in the ledger's worktree); the two new ones were incomplete file lists in the READMEs. No third
  pass: the coordinator compared the last correction (`c0c19bd`) with what commit `708b84f`, the
  last ESS move, touched: 10 files, each covered.
- Coordinator's call on that unit: the status item and the guide were outside the story's scope
  and acceptance and were taken into the unit, because the freeze is what made them false.
- `story:docs-generator-reads-files`: the vocabulary reader is `crates/engineering-vocabulary`
  (`b10x-engineering-vocabulary`, `publish = false`, no embedded data), moved verbatim;
  `canon_engineering::vocabulary` re-exports it beside the embedded released vocabulary;
  `canon-engineering-docs` depends on the new crate and not on `b10x-canon-engineering`. Red run
  on the base: `no_registry_dependency` 0 passed, 3 failed; `vocabulary_reexport` did not
  compile. The root `Cargo.toml` and `crates/canon-engineering/src/lib.rs`, both in the story's
  scope, did not need an edit.
- Adversary on it: pass 1 on `d975e9e` confirmed the move and found 5 things, no red case: the
  unit's two acceptance tests let four mutants through (a registry dependency under a target
  table or a renamed key, a `use … as` import, a second definition with other spacing), `serde`
  and `serde_yaml_ng` were left as normal dependencies of the registry crate, one doc sentence
  was stale, and the new crate lacked `#![forbid(unsafe_code)]`. Its 6 pinning cases and the
  three fixes are `476c46a`. Pass 2: carried 0, new 0, resolved 5.
- Whole gate on the wave branch at `b1f8df0` with the coordinator's `AGENTS.md`, `README.md`
  and `CHANGELOG.md` edits, one exit status per step: `cargo fmt --all --check` 0, clippy
  `-D warnings` 0, `cargo test --workspace --locked --no-fail-fast` 0 (484 passed, 0 failed, 0
  ignored, 110 result lines; `--list` 484), `generate --check` 0 (35 files fresh),
  `aep plan artifact validate` 0, `catalog-archive` 0 (sha256 unchanged, `73bacc79…f004`),
  `sha256sum -c` 0. `cargo tree -p canon-engineering-docs`: 0 lines `b10x-canon-engineering`, 1
  line `b10x-engineering-vocabulary`. No step skipped itself. Run with `ESS_TOOLCHAIN=0.77.0`.
  The site build ran in CI only.
- Found and not fixed here: `AGENTS.md` § Public documentation sends site changes to a skill
  file, `~/beyond10x/.agents/skills/docs/SKILL.md`, that does not exist on the workstation. It
  goes to `story:agents-boundary-data-only` (wave 4), which rewrites `AGENTS.md`. After the wave
  2 merge the `Documentation site` run on `main` failed twice on its `github-pages` artifact
  (none found, then two found after a re-run); wave 2 changed nothing under `website/`.
- Record accuracy: the review-outcome, test-result and last verification records of this
  wave's close carry instants 07:10:11Z to 07:10:42Z and were written at 06:43Z, 27 minutes
  earlier. Wave 1's close records (05:45:00Z, 05:45:01Z) were written at about 04:45Z: local
  time was typed as UTC. Evidence records are not edited after the fact.
- Backlog drafts filed from findings: none.
- Cost, sub-agent tokens as the harness reported them at each stop: `assertions-frozen`
  implementor 46,841, 65,770, 74,408 (three rounds), its adversary 95,652, 116,274 (two
  passes); `docs-generator-reads-files` implementor 94,197, 103,417 (two rounds), its adversary
  117,373, 124,254 (two passes).
