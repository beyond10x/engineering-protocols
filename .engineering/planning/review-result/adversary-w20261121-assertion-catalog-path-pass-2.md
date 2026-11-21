---
format: aep.planning-md/3
id: review-result:adversary-w20261121-assertion-catalog-path-pass-2
kind: review-result
status: active
title: Wave 2026-11-21-w1 adversary, story:assertion-catalog-path, pass 2
relations:
- reviews: story:assertion-catalog-path
revision: 1
---
unit: story:assertion-catalog-path at 9e9a2e6 (diff d2d567c...9e9a2e6), plus one added test file in the working tree
verdict: nothing found
cases: executed 421→422 (`cargo test --workspace --locked --no-fail-fast`, 100 `test result` lines summed: 422 passed, 0 failed, 0 ignored; 421 is that run minus my one added case), red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: no (my added test is green and untracked; keep or drop it as you see fit)

**1. `git --no-pager diff --stat`:** empty. `git status --short` shows one untracked file, `crates/canon-engineering/tests/adversary3_catalog_path.rs`, which is a test path. No implementation file changed.

**2. Case I added.** `crates/canon-engineering/tests/adversary3_catalog_path.rs::a_checkout_without_the_assertion_catalog_does_not_build_and_names_the_file`.
- What it does: it builds the same scratch checkout as `build_embeds_only_…`, then deletes the copied `crates/canon-engineering-assertions/catalog.yaml` and builds `b10x-canon-engineering`.
- What it asserts: the build fails and stderr names `catalog.yaml`.
- Result: green on first run, so no defect. The case alone:
```
test a_checkout_without_the_assertion_catalog_does_not_build_and_names_the_file ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 25.61s
```

**On "does it pass for the right reason".** Yes. `crates/canon-engineering/Cargo.toml` depends on `b10x-canon-engineering-assertions` by path. A checkout without `catalog.yaml` therefore fails at `include_str!("../catalog.yaml")`, which is what my case shows. In the fixed test, the `crates/` copy supplies the file, so step 1 of `build_embeds_only_what_the_rule_and_canon_engineering_docs_accept` builds only because of it. Deleting line 161 therefore lost nothing. That test alone: `3 passed; 0 failed`.

**3. Suite run, after the case existed**
```
$ ESS_TOOLCHAIN=0.77.0 cargo test --workspace --locked --no-fail-fast
100 test-result lines: passed=422 failed=0 ignored=0 (exit 0)
```
The log is `suite2.log` in the unit's scratch directory.
```
$ cargo run --locked -p canon-engineering-docs -- generate --check
canon-engineering-docs: 35 generated files fresh (10 protocols)   (exit 0)
$ git grep -n "catalog/engineering.yaml" -- . ':!.engineering' ':!CHANGELOG.md'
(no output)
```

**4. Judgement findings:** none. I did not re-raise the planning-store paths, which you decided.

**5. What I attacked and could not break**
- The pass-1 break is fixed. `adversary2_protocol_registry` is green: 3 of 3 pass, and the whole workspace has 0 failures.
- The right-reason probe above came back green: the test does depend on the copied `catalog.yaml`.
- Generated site files are fresh, 35 of 35.
- The acceptance grep prints nothing. The `catalog/` directory is gone, and the new file's sha256 equals `0.16.0:catalog/engineering.yaml` (checked in pass 1).
- I did not run `cargo fmt` or `clippy` this pass. Pass 1 found no non-test change to them.

**6. Paths written outside the worktree:** none. The scratch log is under the assigned scratch directory. The test's own scratch checkout is under the worktree's `target/tmp/` and the test removes it afterwards.

```findings
[]
```
