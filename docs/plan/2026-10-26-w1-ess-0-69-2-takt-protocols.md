# Wave 2026-10-26-w1: ESS 0.69.2 and takt's protocols (Atlas ADR 0094)

Released as 0.10.0 (release commit `b669cf4`).

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
| `story:ess-0-69-2-pin` | `ep-unit-ess-pin` | `impl/ess-0-69-2-pin` | implemented, merged; adversary pass 1 green (`3f44d07`) | `3f44d07` | `35cdcbe` |
| `story:takt-protocols-builtin` | `ep-unit-takt-protocols` | `impl/takt-protocols-builtin` | implemented, merged; adversary passes 1 (`449ddd3`, red, fixed in `3aa5211`) and 2 (`2420c1d`, green; comments fixed in `fa5b460`) | `fa5b460` | `c690567` |

Coordinator-owned files: `AGENTS.md`, `README.md`, `CHANGELOG.md`, `Cargo.toml`, `Cargo.lock`, the
store.

## Result

- `story:ess-0-69-2-pin` was implemented by the coordinator (a mechanical pin), not by an
  `aep:implementor`; verification: `ess_contract` fails on `e74fdb2` with ess 0.69.2 and passes on
  `8eb9771`. ESS 0.69.2 refused nothing in the specification; it renamed the provider contract's
  `Status` variants (`process.rs` follows). Adversary pass 1 green, two cases added.
- `story:takt-protocols-builtin`: red run on `9933bf8`; adversary pass 1 red (11 surviving rule
  mutants, six capability-less write actions), corrected in `3aa5211`; pass 2 green (34 more
  mutants, all killed; two comment findings fixed in `fa5b460`). Findings ledger between the two
  passes: carried 0, new 2, resolved 5.
- Coordinator's calls: the four unspellable takt terms stay undeclared rather than widening the
  spelling rule; the six capability-less write actions are recorded in AGENTS.md O1. Filed:
  `story:builtin-state-bound`, `story:vocabulary-unspellable-terms`,
  `story:takt-protocol-write-capabilities`.
- Canon check (Canon 0.6.0): eval-gated-change 55,296, incident-response 2,304, loop-improvement
  20,736, story-delivery 49,152, support-triage 41,472 states, no findings; self-improvement,
  ml-experiment and software-change over the 65,536 bound.
- Full gate on `b669cf4` with ess 0.69.2: fmt, clippy, docs `generate --check`, plan validate
  exit 0; `cargo test --workspace --locked --no-fail-fast` 291 passed, 0 failed; `--list` 291.
