---
format: aep.planning-md/3
id: review-result:adversary-w20261121-assertion-catalog-path-pass-1
kind: review-result
status: active
title: Wave 2026-11-21-w1 adversary, story:assertion-catalog-path, pass 1
relations:
- reviews: story:assertion-catalog-path
revision: 1
---
unit: story:assertion-catalog-path, commit d3990a2 on impl/assertion-catalog-path (base d2d567c)
verdict: NEEDS-CHANGE
cases: executed n/a (suite not run; one existing case run alone) → n/a, red 1
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: yes (the red test is in `crates/canon-engineering/`, which the brief marks not yours; the planning-store records are coordinator-owned)

**1. `git --no-pager diff --stat`:** empty. I changed nothing in the worktree.

**2. Cases I added.** None. The red case is an existing test that the move broke, so I ran it alone. `crates/canon-engineering/tests/adversary2_protocol_registry.rs::build_embeds_only_what_the_rule_and_canon_engineering_docs_accept` is red now.

```
$ ESS_TOOLCHAIN=0.77.0 cargo test -p b10x-canon-engineering --locked --test adversary2_protocol_registry build_embeds_only
test build_embeds_only_what_the_rule_and_canon_engineering_docs_accept ... FAILED
thread '...' panicked at crates/canon-engineering/tests/adversary2_protocol_registry.rs:97:56:
.../ep-unit-assertion-catalog-path/crates/canon-engineering/../../catalog: No such file or directory (os error 2)
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 2 filtered out
```

**3. Suite run:** none. The one-case run above is the only test I ran.

**4. Findings**

| Finding | file:line | Measured | What reaches it | Verdict | Origin |
|---|---|---|---|---|---|
| A test still copies `catalog/` | `crates/canon-engineering/tests/adversary2_protocol_registry.rs:161` | `copy_tree(&root.join("catalog"), …)` panics at `:97` because `catalog/` no longer exists. The base has `catalog/engineering.yaml`, so it passed there (`git show d2d567c:catalog/engineering.yaml` resolves). | `cargo test --workspace --locked`, the third step of `task check`, which CI's `check` workflow runs. The unit's own gate runs only `-p b10x-canon-engineering-assertions`, so it never ran this test. Nothing in the `git grep "catalog/engineering.yaml"` acceptance sees it, because the line spells the directory as `"catalog"`. | NEEDS-CHANGE | introduced |
| Acceptance clause "`task check` is green" does not hold on d3990a2 | same line | Follows from the row above. | The whole gate on the integration branch. | NEEDS-CHANGE | introduced |

- **Fix I would name, not apply:** delete line 161. The `crates/` copy on the line above already carries `crates/canon-engineering-assertions/catalog.yaml`, which the copied crate's `include_str!("../catalog.yaml")` needs. The file is under `crates/canon-engineering/`, which the brief marks not yours, so this goes back through the coordinator.
- **Stale references in the planning store (note, coordinator-owned):**
  - `.engineering/planning/epic/gates-and-delivery-for-any-repository.md:29` still names the old path.
  - `.engineering/planning/story/a-gate-sees-the-change-and-may-not-apply.md:15` and `:84` still name it. Line 15 is `path: catalog/engineering.yaml`, a cited scope entry.
  - `.engineering/planning/epic/data-only-catalog.md:49` names it as the move's source, which is correct history.
  - The story's context says only two artifacts cite the path. The repository-wide `git grep` I ran, which includes `.engineering/`, shows the references listed above.
  - The acceptance grep excludes `.engineering/`, so it does not see these. I did not verify whether the two stories' cited-path fields drive any tooling.

**5. What I attacked and could not break**
- **Bytes:** the new file's sha256 is `d86dca5d…2856`, identical to `0.16.0:catalog/engineering.yaml` and to the base's copy. `ls catalog` fails.
- **`include_str!`:** `lib.rs:26` is `include_str!("../catalog.yaml")`, relative to the crate directory. A consumer that compiles the crate from a Git-tag checkout resolves it inside the crate; the old `../../../` path was the fragile one.
- **Packaging:** `crates/canon-engineering-assertions/Cargo.toml` has no `include` or `exclude`, so nothing filters the file out.
- **Build and CI inputs:** `.github/`, `Taskfile.yml`, `website/package.json` and `Cargo.toml` contain no `catalog` reference. `ess/` mentions only a domain field named `catalog` and no path. `generated/types.rs` has no path to it.
- **`canon-engineering-docs`:** `adversary_rename.rs` copies only `protocols`, `ess` and `website`, so it does not hit the move. The generator does not read the old path.
- **`website/docs/guides/assertions.md:6`:** the new `source:` is correct. The other `canon-catalog/1` hits are format names, not paths.
- **Other files:** `README.md` and `AGENTS.md` contain no old-path reference. `docs/plan/2026-11-21-w1-catalog-manifest.md:40` already names the new path.

**6. Paths written outside the worktree:** none. The test run built under the worktree's own `target/`.

```findings
- file: crates/canon-engineering/tests/adversary2_protocol_registry.rs
  line: 161
  category: contract-drift
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the test still copies the removed catalog/ directory and panics at read_dir (line 97) with "No such file or directory", so cargo test --workspace and task check are red even though the unit's own package gate is green; drop the line, since the crates/ copy already carries catalog.yaml
- file: .engineering/planning/story/a-gate-sees-the-change-and-may-not-apply.md
  line: 15
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: this story, and the epic gates-and-delivery-for-any-repository at line 29, still cite catalog/engineering.yaml as a path, which the acceptance grep excludes by design, so the coordinator-owned records go stale at merge
```
