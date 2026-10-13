# Wave 2026-10-13-w1: software.change/1 profiles, ESS 0.59.0

Skill: `aep:implementing` 0.22.8, wave mode. Release 0.5.0 follows the wave.

| unit | objective | scope |
|---|---|---|
| `story:software-change-profiles` | `vision:O2` | cited and inferred lines in its `## Scope` |
| `story:ess-0-59-0-pin` | `vision:O2` | 10 cited |

The units run one after another: both write `CHANGELOG.md` and `AGENTS.md`, and
`story:ess-0-59-0-pin` `depends_on` `story:software-change-profiles`. One tree,
`$HOME/.local/state/worktree/trees/b10x/engineering-protocols/ep-wave-20261013-w1`, holds the
wave branch `wave/2026-10-13-w1` (base `a51a581`) and each unit's `impl/` branch in turn; build
dir `<worktree>/target`; scratch `$HOME/.cache/engineering-protocols-wave/profiles` and
`$HOME/.cache/engineering-protocols-wave/ess059`.

Commits this wave makes: the opening store commit, the unit commits, their merges into
`wave/2026-10-13-w1`, the adversary's test commit and its fix, the closing store commit, then the
release commit. The wave branch reaches `main` through one bot pull request.

## Planning record

- `blocker:graph-schema-case-inputs` cleared by docs-system 0.9.0
  (`de3e7d84b463d1b6b26e30c28efc07426e8cf012`), which `website/package.json` pins from
  `cd88554`.
- `story:software-change-profiles` § Scope and § Acceptance item 10 name the 0.9.0 graph shapes,
  including the top-level `case.inputs` (`97af4ae`). Its four-critic rounds ran in wave
  2026-10-12-w1; no new panel: one story refined, which § 7 of the planning skill skips.
- `story:ess-0-59-0-pin` filed from a probe: ESS 0.59.0 refused only the `requires: ess 0.55.0`
  line under `--strict-requires`; with it moved, validate, compile, synthesize (0 refusals) and
  both `generate types` exit 0 (`51fb753`).

## Delivery record

| unit | red commit, failing test | implementation | package gates |
|---|---|---|---|
| `story:software-change-profiles` | `270cea4`, `profiles_nest_open_obligations` at items 4, 6, 8 and 10 | `ca6dab7`, `27880a4`, `15fc26c` | exit 0 at `f366f4c` |
| `story:ess-0-59-0-pin` | `8eda583`, both `ess_contract` cases on generated `Cargo.toml` drift | `2770d8a` | exit 0 at `f366f4c`, under `ess` 0.59.0 |

- Merges: `a6afebe` (profiles), `f366f4c` (ESS 0.59.0).
- Package gates: `cargo fmt --all --check`, clippy and test `-p` of `canon-engineering-docs`,
  `b10x-canon-engineering` and `b10x-canon-engineering-assertions`, and `generate --check`, each
  run in its own process group under the disk watchdog (groups 673206 at `27880a4`,
  1095569 after the adversary fixes, 1309488 at `f366f4c`; each started after the one before had
  exited). `b10x-canon-engineering` executed 124 at `cd88554` and
  133 at `f366f4c`. The pull request's CI runs the full gate.
- `story:software-change-profiles` § Acceptance item 6 is amended (§ Amendment): Canon 0.2.0
  binds evidence to the current revision, so red on R1 no longer admits the bugfix edit at R2.
- Adversary pass 1 (`review-result:adversary-w20261013-software-change-profiles-pass-1`, tests
  `95cde62`): two findings. A green run before the red one on a revision leaves the bugfix
  blocked; kept as a stated limitation, pinned by a test and stated on the step-order page. No
  fixture discharged an obligation; fixed by the fixture `profiles-discharged` (`15fc26c`).
- `story:ess-0-59-0-pin` was implemented by the coordinator, not a dispatched implementor: seven
  one-line edits and a regeneration already probed.
- Build dir `target/` of the tree: 2.1G (`du -sh`) after the last gate.
