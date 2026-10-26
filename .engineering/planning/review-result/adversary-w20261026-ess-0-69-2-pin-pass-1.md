---
format: aep.planning-md/3
id: review-result:adversary-w20261026-ess-0-69-2-pin-pass-1
kind: review-result
status: active
title: Adversary pass 1 on story:ess-0-69-2-pin
relations:
- reviews: story:ess-0-69-2-pin
revision: 1
---
unit: story:ess-0-69-2-pin
verdict: green
cases: executed 26→28, red 0
origin: introduced 1, pre-existing 1, undecided 0
wrote-outside-worktree: the session scratch root's unit-ess-pin/adv1/ (6 entries, listed in part 6)
needs-coordinator: no

Nothing failed against the unit. My two new cases pass, and no ESS 0.69.2 command printed a warning. The only defect-like finding is an old test gap, which my new cases now close. My cases are in commit `3f44d07`, made by `b10x-bot[bot]` as author and committer.

**1. `git --no-pager diff --stat 8eb9771..HEAD`** (the working tree is clean afterwards):
```
 .../tests/adversary_status_wire.rs                 | 136 +++++++++++++++++++++
 1 file changed, 136 insertions(+)
```
I changed only a test file. No implementation file was touched.

**2. My cases** are in `crates/assertion-providers/tests/adversary_status_wire.rs`. I ran them alone first and both passed (exit 0):
- `generated_status_keeps_the_wire_names`: `Status::Known` and `Status::Unavailable` serialize to `"known"` and `"unavailable"`, and read back from them. The Rust-style names `Known`, `Unavailable`, `V0` and `V1` are refused on the wire.
- `external_response_status_and_payload_follow_the_documented_contract`: I checked the external provider path against the README contract (`README.md:73`).
  - `known` with `"false"` gives `Known(false)`.
  - `unavailable` with a reason gives `Unavailable(reason)`.
  - These 8 inconsistent responses are refused: `unavailable` with `value_json`, `unavailable` with both payloads, `unavailable` with no payload, `unavailable` with an empty reason, `known` with only a reason, `known` with no payload, `"Known"` and `"V0"`.

Mutation probes, run on a scratch copy of `process.rs`:

| mutant | existing 15 assertion-providers cases | new case |
|---|---|---|
| `:80` `(_, Present, Absent)` | green | red at `:131` |
| `:83` `(_, Absent, Present)` | green | red: `{"reason":"down","status":"known"} … got Ok(Unavailable("down"))` |
| `:84` drop `!reason.is_empty()` | green | red: `{"reason":"","status":"unavailable"} … got Ok(Unavailable(""))` |
| Known and Unavailable swapped | red (`conformance.rs:174`) | red at `:107` |

**3. Suite run** with ESS 0.69.2 first on PATH:
- Command: `cargo test -p b10x-canon-engineering-assertions -p b10x-assertion-providers --locked`
- Result: 1+0+2+14+0+5+1+3+2+0+0 = 28 passed, exit 0.
- The 26 before comes from the implementor's own run log, `unit-ess-pin/test.txt`.
- clippy `-D warnings` on the package and `rustfmt --check` on my file both pass.

**4. Judgement findings** (they cover commit `3f44d07`):
- **F1, pre-existing.** The status check at `process.rs:80/83/84` had no test that could fail if it broke: the first three mutants above survive the existing suite. The base commit `e74fdb2` has the same match logic, and the unit did not change any assertion-providers test. My commit closes the gap.
- **F2, introduced.** The story's Scope says 0.69.2 changes only the header, `generator_version` and the `Cargo.toml` requirements. In fact it also renames the public `assertion_providers::contract::Status` variants `V0`/`V1` to `Known`/`Unavailable`. The README presents that module as shared with external Rust providers, so this breaks their source.
  - The story's scope list does not include `process.rs`.
  - `CHANGELOG.md` `[Unreleased]` is empty.
  - Who is affected: I found no external Rust consumer anywhere in the workspace (I searched all `*.rs` and `Cargo.toml`). `mcp`'s `Status::V0` is its own type.

**5. What I attacked and could not break:**
- **Generated files:** the regenerated `Cargo.toml`, `types.rs`, `types-report.json` and `source.schema.json` in both crates match a fresh 0.69.2 run of the test's own commands byte for byte.
- **ESS checks:** `validate --strict-requires`, `compile`, `synthesize` and `generate types` print no warning on either contract. `--format json` reports `unresolved_references: []`.
- **CI checksum:** it matches the `SHA256SUMS` downloaded from the 0.69.2 release.
- **Caret serde requirements:** both generated crates are in the workspace `exclude` list and nothing builds them standalone. `Cargo.lock` still pins serde 1.0.229 and serde_json 1.0.151.
- **Acceptance 2:** `git grep 0\.60\.0` is empty outside the excluded paths.
- **Not run:** the docs generator `--check`, which the brief does not allow me to build.

**6. Paths I wrote outside the worktree**, all under the scratch root's `unit-ess-pin/adv1/`: `release/SHA256SUMS`, `out/`, `gen/`, `process.rs.orig`, the commit message, `mutant/` (a copy of the workspace with its own `target/`, 506M, left in place).

(Coordinator's note, not the adversary's: home-directory paths in the report were shortened to the scratch root to keep personal paths out of the store; `mutant/target` was removed with `cargo clean --manifest-path`.)

```findings
[{"file": "crates/assertion-providers/src/process.rs", "line": 80, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "Three mutants of the provider response match survived the existing suite (status ignored on the value arm, status ignored on the reason arm, empty-reason guard dropped); the new cases in 3f44d07 (adversary_status_wire.rs) fail on each."},
 {"file": "crates/assertion-providers/generated/types.rs", "line": 97, "category": "contract-drift", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "ESS 0.69.2 renames the public assertion_providers::contract::Status variants V0/V1 to Known/Unavailable, which breaks the source of external Rust providers; the story scope did not foresee it, process.rs was edited outside the cited scope, and CHANGELOG [Unreleased] does not record it."}]
```
