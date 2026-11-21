---
format: aep.planning-md/3
id: review-result:adversary-w20261121-docs-generator-reads-files-pass-2
kind: review-result
status: active
title: Wave 2026-11-21-w3 adversary, story:docs-generator-reads-files, pass 2
relations:
- reviews: story:docs-generator-reads-files
revision: 1
---
unit: story:docs-generator-reads-files, commit 476c46a on impl/docs-generator-reads-files (correction on top of d975e9e), worktree clean
verdict: nothing found
cases: executed 387→387, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path in the adversary scratch under the coordinator's session scratch
needs-coordinator: none

ledger: carried 0 / new 0 / resolved 5

The correction holds. All five pass-1 findings are resolved and nothing new was found. Nothing was
added to the tree.

**Pass-1 findings**

| Pass-1 finding | Status | Measurement |
|---|---|---|
| `no_registry_dependency.rs:22` does not catch a target-table dependency, a renamed key, or `use canon_engineering as x` | resolved | `adversary_no_registry_dependency.rs` is committed; it caught mutants M2, M3 and M4 in pass 1 and passes in the suite |
| `vocabulary_reexport.rs:57` checks one spelling of one type | resolved | `adversary_vocabulary_reexport.rs` is committed; it caught M5 and passes in the suite |
| `serde` and `serde_yaml_ng` were normal dependencies with no user in the library | resolved | both are dev-dependencies, `Cargo.toml` lines 23 and 25 |
| stale "spelling rule below" at `vocabulary.rs:5` | resolved | reads "the spelling rule of `b10x-engineering-vocabulary`" |
| no `#![forbid(unsafe_code)]` in the new crate | resolved | it is `lib.rs:1`, and the crate builds |

**The dependency move.** `cargo build --locked -p b10x-canon-engineering` builds the library and
the binary, exit 0, without dev-dependencies: the derives on the re-exported types expand inside
`b10x-engineering-vocabulary`. A search of `crates/canon-engineering/src` and `build.rs` for serde,
`yaml_ng`, `Deserialize` and `Serialize` finds nothing. `Cargo.lock` is unchanged between d975e9e
and 476c46a, and `cargo metadata --locked` for the workspace exits 0.

**Suite.** `ESS_TOOLCHAIN=0.77.0 cargo test -p b10x-engineering-vocabulary
-p b10x-canon-engineering -p canon-engineering-docs --locked --no-fail-fast`: 87 result lines,
387 passed, 0 failed, exit 0.

```findings
[]
```
