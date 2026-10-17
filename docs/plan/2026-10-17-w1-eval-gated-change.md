# Wave 2026-10-17-w1: eval.gated_change/1

Released as 0.9.0. Integration branch `wave/2026-10-17-w1`.

Width: one unit; the wave had a single ready story, and its protocol-first commits (protocol and
failing test, vocabulary, generated site) depend on each other. The adversary ran in its own tree
beside the unit's last step.

| Story | Commits | Merge |
|---|---|---|
| `story:eval-gated-change-protocol` | `1443d0d` (red run `aab6899`), `c2983a1`, `49dea9a`, `bf6ea5b`, `79dcdca` | `608c0e4` |
| adversary pass 1 and its fixture states | `c46ab63`, `9ea7118` | `35fd829` |

- `architecture-decision-record:eval-gated-change-shape` accepted by the operator on 2026-10-17
  (option A: its own protocol).
- Acceptance item 2 checks no capability ids: `vocabulary/1` has no capability category.
- Implementor's readings beyond the story: a read action `candidate.evaluate` produces the candidate
  evidence (Canon refuses an evidence kind no action produces); `experiment.valid` also demands
  `different_agent_run`; `rejected` needs `change.within_surface` true, so it excludes `proposed`.
- Adversary pass 1 (`review-result:adversary-w20261017-eval-gated-change-pass-1`): no wrong
  decision; four surviving mutants, killed by nine fixture states; three judgement notes kept as
  stated limits in CHANGELOG 0.9.0.
- Release commit `6fe25bb`. Full gate on it with ESS 0.60.0: clippy and docs `generate --check`
  exit 0; `cargo test --workspace --locked --no-fail-fast` 274 passed, 0 failed; `--list` 274;
  `cargo fmt --all --check` flagged one test file, fixed in `89daa83` (formatting only), after
  which fmt, clippy of the package and the test target were run again.
- `canon check` over /1: 55,296 states of 65,536, no findings. Peak target du 3504197354 bytes.
