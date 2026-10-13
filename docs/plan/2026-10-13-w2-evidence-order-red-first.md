# Wave 2026-10-13-w2: the red-first rule from evidence order, Canon 0.3.0

Skill: `aep:implementing` 0.22.8, wave mode. Release 0.6.0 follows the wave.

| unit | objective | scope |
|---|---|---|
| `story:evidence-order-red-first` | `vision:O2` | cited and inferred lines in its `## Scope` |

One story, delivered in three units one after another on one tree,
`$HOME/.local/state/worktree/trees/b10x/engineering-protocols/ep-wave-20261013-w2`: the wave
branch `wave/2026-10-13-w2` (base `ec83ed8`) and the `impl/` branches
`impl/evidence-order-red-first`, `impl/evidence-order-red-first-fixes` and
`impl/docs-system-0-10-0-pin`; build dir `<worktree>/target`; scratch
`$HOME/.cache/engineering-protocols-wave/w2`.

## Planning record

- `story:evidence-order-red-first` filed and moved to active (`31e9474`). One story, so the
  four-critic panel was skipped (§ 7 of the planning skill).
- Canon 0.3.0's `first` is FALSE for green then red of one kind on one revision (its
  `story:evidence-order-predicate`, acceptance 5), so the suite's run and the regression test's
  run are told apart by kind: `regression_test_result`, with `implementation_change` for
  `before`.
- `blocker:graph-schema-evidence-order` filed (`31e9474`) and cleared by docs-system 0.10.0
  (`48afe6a1d55434f393baaeb769ad7f9bdb284b02`), pinned in `031ada6`.

## Delivery record

| unit | red commit, failing test | implementation | merge |
|---|---|---|---|
| Canon 0.3.0 and the rule | `6b184e2`, `a_bugfix_reads_red_first_from_evidence_order`, exit 101 | `d785c0f`, `0c1e710` | `d661165` |
| adversary pass 1 fixes | `c6b8c3e` (adversary), 4 cases red | `6f097df`, `b620820`, `2edb542` | `369ead8` |
| docs-system 0.10.0 graph | `the_transcription_follows_the_0_10_0_order_leaves` red before `031ada6` | `031ada6`, `4ae8e59` | `92733c0` |

- Adversary pass 1 (`review-result:adversary-w20261013-w2-red-first-pass-1`): a regression test
  whose first result passed discharged `red_first.required` (fixed: `regression.shown_first` needs
  the first result failed); the step-order page disagreed (fixed); no fixture exercised the
  subject of either order form (fixed: `red-first-other-subject`); an implementation change made
  without an `implementation_change` record is not seen (stated on the step-order page and in the
  changelog).
- Known limit from Canon: a red record about any earlier revision counts, whether or not the
  current revision descends from it.
- Full gate `task check` on the release commit `bed4143`: process group 1061841 under the disk
  watchdog, exit 0, 232 passed, 0 failed, `generate --check` 18 generated files fresh;
  `cargo test --workspace --locked -- --list` lists 232.
