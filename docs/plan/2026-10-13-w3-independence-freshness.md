# Wave 2026-10-13-w3: independence and freshness, Canon 0.4.0

Skill: `aep:implementing`, wave mode. Release 0.7.0 follows the wave.

| unit | objective | scope |
|---|---|---|
| `story:canon-0-4-0-pin` | `vision:O2` | the three crate manifests, `Cargo.lock` |
| `story:security-independence-rules` | `vision:O2` | cited lines in its `## Scope` |
| `story:rollback-verification-rule` | `vision:O2` | cited lines in its `## Scope` |
| `story:stale-evidence-fixtures` | `vision:O2` | cited lines in its `## Scope`, the fixture harness |
| `story:protocol-pages-independence-freshness` | `vision:O2` | the docs generator and generated pages |

Five units one after another on one tree,
`$HOME/.local/state/worktree/trees/b10x/engineering-protocols/ep-wave-20261013-w3`: the wave
branch `wave/2026-10-13-w3` (base `decf907`) and one `impl/` branch per unit, plus
`impl/adversary-w3-pass-1` and `impl/adversary-w3-fixes`; build dir `<worktree>/target`; scratch
`$HOME/.cache/engineering-protocols-wave/w3`. The units share `protocols/*/1.yaml`,
`protocols/vocabulary.yaml` and the fixture trees, so none ran in parallel.

## Planning record

- `fc72e4d`: the three stories state Canon 0.4.0 (`7c2cec13`) as the release that carries
  independence; revision binding and freshness are carried since Canon 0.1.0. The pin and the
  docs story were filed. The ordering edge from `story:security-independence-rules` to
  `story:ess-conformance-evidence` (draft) was dropped: it only ordered two edits of
  `implementation.verified`.
- No part needs Canon's requirements catalogue; the rules are held by fixtures here.

## Delivery record

| unit | red commit, failing test | implementation | merge |
|---|---|---|---|
| Canon 0.4.0 | none (no behaviour change); 232 passed on `53f4118` | `53f4118` | `0e52771` |
| independent security review | `2aa3890`, `independent_security_review_gates_implementation_verified`, item 5 | `05963e6`, `038ba3b` | `3741648` |
| independent rollback verification | `4e2b4c6`, `rollback_verified_needs_an_independent_verifier`, item 4 | `719f53f`, `8d84f53` | `47193a2` |
| stale evidence | `16aa046`, `stale_evidence_is_unknown_and_fresh_evidence_holds`, items 3 to 6 (items 1, 2 baseline) | `e7b65cb`, `6d72bfa` | `f564f21` |
| pages | `ffb708f`, 5 docs tests | `6694949`, `94a7962` | `f695bbf` |
| adversary pass 1 | `12ed5cd`, 3 of 14 cases red | `6d96f29` | `a151da1`, `3e2b82a` |

- Adversary pass 1 (`review-result:adversary-w20261013-w3-independence-pass-1`): no rule decided
  wrongly; a rejected security review, a partial rollback result and a rollback result about
  another artifact had no fixture, so dropping that condition survived. Fixed by four fixtures,
  one more of the same class (a security review about another artifact).
- Stated limits (CHANGELOG 0.7.0): `rollback.verified` decides nothing yet; independence is the
  case writer's word; no Canon catalogue entry.
- Full gate on the release commit `daa6f80` (`task check`'s steps, the test step
  `--no-fail-fast`): process group 1038973 under the disk watchdog, no stop, every step exit 0,
  257 passed, 0 failed; `generate --check` 18 generated files fresh;
  `cargo test --workspace --locked -- --list` lists 257. Target `du` 5.3G after the gate.
