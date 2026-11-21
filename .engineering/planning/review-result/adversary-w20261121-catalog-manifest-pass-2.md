---
format: aep.planning-md/3
id: review-result:adversary-w20261121-catalog-manifest-pass-2
kind: review-result
status: active
title: Wave 2026-11-21-w1 adversary, story:catalog-manifest, pass 2
relations:
- reviews: story:catalog-manifest
revision: 1
---
unit: story:catalog-manifest, impl/catalog-manifest at 3cc3244 (base d2d567c); my working tree adds nothing
verdict: CONFIRMED
cases: executed 443→443 (whole workspace, no case added to the tree), red 0 in the tree, 8 in the scratch probe
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 11 paths, listed in part 6
needs-coordinator: yes. The four cases that would kill the new mutants call the private check function, so they are in the scratch directory, not the tree. Moving them into `catalog_manifest.rs` is the implementor's change.

The check is still correct. The correction in 3cc3244 holds, and the workspace gate is green. What I found are five more weakenings of the check that the 22 tests do not notice, plus one note on how a consumer finds the schema.

**1. Diff**

`git --no-pager diff --stat` and `git status --short` are both empty. As instructed, I did not re-create `adversary_catalog_manifest.rs`; every probe ran in a separate scratch package.

**2. The correction (5b11148 to 3cc3244)**
- **Check function unchanged:** every hunk starts at line 446 or later, so `check` and its helpers are untouched.
- **Nothing dropped or relaxed:** the only removed lines are the inline lock rewrite and the expected-problem filter of `a_pinned_canon_lower_than_a_listed_canon_is_a_problem`. They moved into `set_pinned_canon` and `pinned_below`, with the same logic. Both earlier assertions are still there.
- **New cases fail for their own reason:** I re-ran three pass-1 mutants against copies of the file at 3cc3244. Each turns exactly one unit test red and nothing else:
  - r1: `a_canon_that_is_not_major_minor_patch_is_a_problem`
  - r4: `a_lock_that_pins_no_canon_or_no_release_is_a_problem` (`left: []`)
  - r8: `the_lowest_of_two_pinned_canons_is_compared`
- The unit's file now has 22 tests; `--list` shows all 22 in the gated tree.

**3. New mutants and the cases that kill them**

The probe is a separate scratch package that runs copies of the file at 3cc3244: `orig` unmutated, plus r1, r4, r8 and five new mutants n1 to n5. Each copy also carries my four cases from `scratch/adversary/killers2.rs.part`. It was run alone with `cargo test --offline --test probe --no-fail-fast`, exit 101:

```
test n1::a_listed_revision_below_what_canon_parses_is_a_problem ... FAILED
test n2::a_listed_id_that_is_a_prefix_of_the_parsed_one_is_a_problem ... FAILED
test n3::a_non_yaml_file_or_dotfile_under_protocols_is_unlisted ... FAILED
test n4::a_protocol_file_with_crlf_line_endings_is_a_digest_problem ... FAILED
test n5::a_non_yaml_file_or_dotfile_under_protocols_is_unlisted ... FAILED
test r1::a_canon_that_is_not_major_minor_patch_is_a_problem ... FAILED
test r4::a_lock_that_pins_no_canon_or_no_release_is_a_problem ... FAILED
test r8::the_lowest_of_two_pinned_canons_is_compared ... FAILED
test result: FAILED. 226 passed; 8 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.72s
```

All 22 unit tests pass in every n-mutant, so all five survive. Each of my cases is red only in its own mutant, and all 26 tests are green in `orig`.

| Mutant | catalog_manifest.rs line | Weakening | Killed by (scratch only) |
|---|---|---|---|
| n1 | :197 | `!=` becomes `>`: a listed revision lower than the file's is accepted | `a_listed_revision_below_what_canon_parses_is_a_problem` |
| n2 | :190 | id compared by prefix: `id: software` is accepted | `a_listed_id_that_is_a_prefix_of_the_parsed_one_is_a_problem` |
| n3 | :287 | the walk counts only `.yaml` files: `protocols/README.md` is never flagged | `a_non_yaml_file_or_dotfile_under_protocols_is_unlisted` |
| n5 | :284 | the walk skips dotfiles: `.1.yaml.swp` is never flagged | same case |
| n4 | :297 | digest taken after CRLF→LF: a CRLF file passes against the LF digest | `a_protocol_file_with_crlf_line_endings_is_a_digest_problem` |

**4. Workspace gate at 3cc3244, clean tree**
- `cargo fmt --all --check`: `EXIT=0`.
- `ESS_TOOLCHAIN=0.77.0 cargo test --workspace --locked --no-fail-fast`: 100 `test result` lines add up to 443 passed, 0 failed, 443 executed. The `catalog_manifest` line reads `test result: ok. 22 passed; 0 failed; ...`. `--list` gives 443 tests in this tree.
- `cargo run --locked -p canon-engineering-docs -- generate --check`: `canon-engineering-docs: 35 generated files fresh (10 protocols)`, `EXIT=0`.

**5. The manifest and schema as a consumer contract**

One gap: a runtime holding only `protocols/manifest.yaml` cannot find its schema. The manifest has no schema reference and the schema has no `$id`. The link from `b10x-protocol-catalog/1` to `schema/b10x.protocol-catalog.v1.schema.json` is written only in a test comment and in `docs/plan/`. The names themselves were the coordinator's decision; what is missing is any machine-readable link between them.

Nothing else would mislead a consumer:
- **Path base:** the schema states that paths are relative to the repository root and that the manifest is `protocols/manifest.yaml`.
- **Digest:** lowercase hex, enforced by the check.
- **Bytes:** CI checks out exact blob bytes, so a CRLF working copy cannot pass the gate.
- **`canon` wording:** it says "parses, validates and compiles" and makes no claim about evaluation. That matches the probe (monotone up to 0.9.0) and Canon's changelog: order forms keep their 0.7.0 values over whole seconds, and `max_age` keeps whole seconds.
- **id and revision:** both are described as what Canon parses.
- **Duplicates:** "no path listed twice" is stated.

**Attacked and not broken:** the correction's equivalence, the green gate, and the fmt and docs checks.

**Findings**

All rows are about tests the unit adds. The check behaves correctly in each case; the suite just would not notice if it stopped. Fix: add the four cases in `killers2.rs.part` to `catalog_manifest.rs`; all four pass on the real code.

| Finding | file:line | What was measured | What reaches it in practice | Verdict | Origin |
|---|---|---|---|---|---|
| A listed revision lower than the file's is untested | crates/canon-engineering/tests/catalog_manifest.rs:197 | n1 passes all 22 tests; my case is red only in n1 | A revision bump in a protocol file without the manifest update, or the reverse | CONFIRMED | introduced |
| A listed id that is a prefix of the parsed id is untested | crates/canon-engineering/tests/catalog_manifest.rs:190 | n2 survives | A future "loose" id comparison; nothing today | CONFIRMED | introduced |
| No test adds a non-`.yaml` file or a dotfile under `protocols/` | crates/canon-engineering/tests/catalog_manifest.rs:287 | n3 and n5 survive | A README, notes file or editor swap file; `build.rs` ignores them, so only this check sees them | CONFIRMED | introduced |
| No test changes line endings while keeping the content, so "sha256 of the bytes" is not pinned | crates/canon-engineering/tests/catalog_manifest.rs:297 | n4 survives | A digest that normalises text would accept bytes a consumer's raw sha256 refuses | CONFIRMED | introduced |
| The manifest does not name its schema and the schema has no `$id` | schema/b10x.protocol-catalog.v1.schema.json | read: no `$id`, no schema key in the manifest; the mapping is only in a test comment and `docs/plan/` | A runtime that reads only the manifest | CONFIRMED | introduced |

**6. Paths written outside the worktree**

Eleven paths in the unit's scratch directory, under `adversary/`: `killers2.rs.part`, `make-mutants-p2.sh`, `p2-gate.sh`, `p2-gate-head.txt`, `p2-fmt.txt`, `p2-test.txt`, `p2-list.txt`, `p2-docs.txt`, `p2-gate-done.txt`, `p2-probe-run.txt`, and `p2-probe/` (Cargo.toml, Cargo.lock, src/lib.rs, tests/probe.rs, tests/copies/{orig,r1,r4,r8,n1..n5}.rs, and its own `target/` build directory).

From pass 1, `target/adversary-catalog-manifest/` is still inside the worktree; git ignores it.

```findings
- file: crates/canon-engineering/tests/catalog_manifest.rs
  line: 197
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: no test lists a revision below the parsed one, so weakening the revision comparison from != to > keeps all 22 tests green
- file: crates/canon-engineering/tests/catalog_manifest.rs
  line: 190
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the only id case lists a longer id, so an id comparison by prefix survives
- file: crates/canon-engineering/tests/catalog_manifest.rs
  line: 287
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: no test adds a non-yaml file or a dotfile under protocols/, so a walk that skips them survives
- file: crates/canon-engineering/tests/catalog_manifest.rs
  line: 297
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: no test changes line endings while keeping content, so a digest over normalised text instead of raw bytes survives
- file: schema/b10x.protocol-catalog.v1.schema.json
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: a consumer holding only the manifest cannot locate its schema because neither file references the other and the schema has no $id
```
