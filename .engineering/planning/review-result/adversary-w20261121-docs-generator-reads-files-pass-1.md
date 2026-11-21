---
format: aep.planning-md/3
id: review-result:adversary-w20261121-docs-generator-reads-files-pass-1
kind: review-result
status: active
title: Wave 2026-11-21-w3 adversary, story:docs-generator-reads-files, pass 1
relations:
- reviews: story:docs-generator-reads-files
revision: 1
---
unit: story:docs-generator-reads-files, commit d975e9e on impl/docs-generator-reads-files (base 794db23) plus two uncommitted adversary test files
verdict: CONFIRMED
cases: executed 381→387, red 0
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: the adversary scratch directory under the coordinator's session scratch
needs-coordinator: none

The move holds: the reader moved verbatim, every public path still resolves, and the generator
works as before. No red case. All five findings are introduced by the unit and small: two
acceptance tests weaker than the acceptance, and three notes.

**1. What was touched.** Two untracked test files, no implementation file:
`crates/canon-engineering-docs/tests/adversary_no_registry_dependency.rs` and
`crates/canon-engineering/tests/adversary_vocabulary_reexport.rs`. `cargo fmt --check` exits 0 for
both crates; clippy with `--all-targets -D warnings` is clean.

**2. Cases added** (pins, all green)

- `adversary_no_registry_dependency.rs`, 3 tests: asks `cargo metadata --no-deps --offline` for
  each dependency's real package name, kind and target, follows path dependencies through the
  workspace as `cargo tree -p canon-engineering-docs` does, and asserts the generator never
  reaches `b10x-canon-engineering` and has an unconditional normal dependency on
  `b10x-engineering-vocabulary`; and that no file under `src/` and no `build.rs` contains
  `canon_engineering` as a whole identifier.
- `adversary_vocabulary_reexport.rs`, 3 tests: every path the module had at the base against the
  base's signatures (fn-pointer types for each method and free function, the types of `FORMAT`
  and `ALL`, `const fn as_str`, the derived and `Display`/`Error` traits,
  `Serialize + DeserializeOwned` on `Category` and `Marking`, field names, an exhaustive match
  over `VocabularyError`); each of the six moved types has one declaration under `crates/`, in
  any visibility and spacing, in `engineering-vocabulary`.

Mutants on a scratch copy, the unit's tests against the new ones:

| Mutant | Unit's test | Adversary's |
|---|---|---|
| M1: `[dev-dependencies] b10x-canon-engineering` | killed | killed |
| M2: the dependency under `[target.'cfg(unix)'.dependencies]` | survives | killed |
| M3: `registry = { package = "b10x-canon-engineering", … }` | survives | killed |
| M4: M2 plus `use canon_engineering as registry;` in `src/status.rs` | survives | killed |
| M5: a second definition spelled `pub  struct  Vocabulary {` | survives | killed |

**3. Suite run** (`ESS_TOOLCHAIN=0.77.0 cargo test -p b10x-engineering-vocabulary
-p b10x-canon-engineering -p canon-engineering-docs --locked --no-fail-fast`, exit 0): 87 result
lines, 387 passed, 0 failed.

**4. Findings**

| Finding | file:line | Measured | What reaches it | Verdict | Origin |
|---|---|---|---|---|---|
| The acceptance test reads only the keys of three top-level tables and searches for `canon_engineering::`. A registry dependency under a target table or another key, and `use canon_engineering as x`, pass it. | `crates/canon-engineering-docs/tests/no_registry_dependency.rs:22`, `:70` | M2, M3, M4 survive | a later edit that brings the dependency back that way; the acceptance is "cargo tree lists none"; the new pin covers it | CONFIRMED | introduced |
| The one-definition test matches one spelling, for `Vocabulary` only, and turns red if any `.rs` file under `crates/` merely contains that text. | `crates/canon-engineering/tests/vocabulary_reexport.rs:57` | M5 survives | a second definition written differently, or a test quoting the text; the new pin covers all six types | CONFIRMED | introduced |
| `serde` and `serde_yaml_ng` stay in the registry crate's `[dependencies]`, but its `src/` no longer uses them; only integration tests do. | `crates/canon-engineering/Cargo.toml:21-22` | `git grep` for `serde` in the crate's `src` and `build.rs` finds nothing | Git consumers compile them through the new crate anyway | CONFIRMED | introduced |
| The module doc says "the few the spelling rule below refuses", but the spelling rule moved. | `crates/canon-engineering/src/vocabulary.rs:5` | read | readers of rustdoc | CONFIRMED | introduced |
| The moved code was covered by `#![forbid(unsafe_code)]` at `canon-engineering/src/lib.rs:1`; the new crate root has no such line. | `crates/engineering-vocabulary/src/lib.rs:1` | grep over the crate roots | nothing today; the code has no `unsafe` | CONFIRMED | introduced |

**5. Attacked and not broken.** The public surface (the diff of the base file against the new
`lib.rs` plus what stayed shows only doc text and the embedded-data lines; all 7 public items and
all methods are re-exported; no `pub(crate)`; the orphan rule gives the same result downstream).
Git consumers (a path dependency of a Git dependency resolves inside the same checkout;
`publish = false` matters only to `cargo publish`; the registry crate already path-depends on the
assertions crate this way). No embedded data and one definition in the new crate. The generator:
its `src/` diff is two import lines; on a scratch copy the head binary gives 35 files fresh, an OS
error and exit 1 for a missing file or a directory in its place, and the format refusal for
`vocabulary/2`. No sentence in `AGENTS.md`, `README.md` or `website/` names the old reader
location.

```findings
- file: crates/canon-engineering-docs/tests/no_registry_dependency.rs
  line: 22
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the acceptance test reads only the keys of three top-level dependency tables and the text canon_engineering followed by two colons, so a registry dependency under a target table or a renamed key plus a use-as import all pass it
- file: crates/canon-engineering/tests/vocabulary_reexport.rs
  line: 57
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the one-definition check matches one exact spelling for Vocabulary only, so a differently spaced second definition survives, and any file quoting that text turns it red without a defect
- file: crates/canon-engineering/Cargo.toml
  line: 21
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: serde and serde_yaml_ng stay normal dependencies of the registry crate although after the move only its integration tests use them
- file: crates/canon-engineering/src/vocabulary.rs
  line: 5
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the module doc still refers to the spelling rule below, which moved to the new crate
- file: crates/engineering-vocabulary/src/lib.rs
  line: 1
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the moved reader lost the forbid(unsafe_code) it had as a module of canon_engineering
```
