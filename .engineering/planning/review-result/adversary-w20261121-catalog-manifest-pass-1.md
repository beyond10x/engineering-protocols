---
format: aep.planning-md/3
id: review-result:adversary-w20261121-catalog-manifest-pass-1
kind: review-result
status: active
title: Wave 2026-11-21-w1 adversary, story:catalog-manifest, pass 1
relations:
- reviews: story:catalog-manifest
revision: 1
---
unit: story:catalog-manifest, tree impl/catalog-manifest at 5b11148 plus one untracked adversary file
verdict: CONFIRMED
cases: executed 268→488, red 9
origin: introduced 8 / pre-existing 0 / undecided 0
wrote-outside-worktree: 6 paths, listed in part 6
needs-coordinator: yes. Do not commit `crates/canon-engineering/tests/adversary_catalog_manifest.rs` as it stands (part 2). The cases worth keeping belong in `catalog_manifest.rs`, which is the implementor's file.

**1. Diff**

`git --no-pager diff --stat` is empty. `git status --short` shows one file:
```
?? crates/canon-engineering/tests/adversary_catalog_manifest.rs
```
That is a test path, and no other file changed. Mutated copies of the check are only under the worktree's ignored `target/adversary-catalog-manifest/`.

**2. The cases I added**

The cases are in `crates/canon-engineering/tests/adversary_catalog_manifest.rs`. It is a probe. Its `#[path]` lines load ten copies of `catalog_manifest.rs` from `target/adversary-catalog-manifest/`:
- `orig`: unchanged.
- `m1` to `m9`: each with one line weakened.

Each copy gets the same 8 `adv_*` cases from `scratch/adversary/killers.rs.part`. It only compiles in this worktree, and part of it is red on purpose: committing it breaks the gate. The check functions are private to the `catalog_manifest` test binary, so no file of mine can call the real `check`. The real fix is to add the 8 cases to `catalog_manifest.rs`.

`ESS_TOOLCHAIN=0.77.0 cargo test -p b10x-canon-engineering --locked --test adversary_catalog_manifest`, run before the suite:
- None of the 14 original tests fails in any mutant: all 9 mutants survive the unit's suite.
- In `orig`, all 22 tests are green.
- Each `adv_*` case is red only in its own mutant.

```
---- m1::adv_a_canon_that_is_not_major_minor_patch_is_a_problem stdout ----
assertion `left == right` failed: 0.4.0.1
  left: []
---- m3::adv_a_canon_one_patch_above_the_pin_is_a_problem stdout ----
assertion `left == right` failed
  left: []
---- m4::adv_a_lock_that_pins_no_canon_is_a_problem stdout ----
[]
test result: FAILED. 211 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.34s
```

| Mutant (catalog_manifest.rs line) | Case that kills it |
|---|---|
| m1 `:157` CanonValue never pushed | `adv_a_canon_that_is_not_major_minor_patch_is_a_problem` (`0.4.0.1`, `v0.4.0`, `0.04.0`, `0.4`) |
| m2 `:315` `release` accepts a 4th component | same case |
| m3 `:163` pin compare ignores patch | `adv_a_canon_one_patch_above_the_pin_is_a_problem` |
| m4 `:130` Lock error swallowed | `adv_a_lock_that_pins_no_canon_is_a_problem` |
| m5 `:184` Unparsed dropped | `adv_a_listed_file_canon_does_not_parse_is_a_problem` |
| m6 `:152` Vocabulary dropped | `adv_a_vocabulary_entry_for_another_file_is_a_problem` |
| m7 `:286` walk stops at depth 2 | `adv_a_file_in_a_subdirectory_of_a_protocol_directory_is_unlisted` |
| m8 `:346` highest of two pins compared | `adv_the_lowest_of_two_pinned_canons_is_compared` |
| m9 `:241` digest compare ignores case | `adv_an_uppercase_sha256_is_a_problem` |

**3. Suite, run after the cases existed**

`ESS_TOOLCHAIN=0.77.0 cargo test -p b10x-canon-engineering --locked --no-fail-fast`, exit 101:
```
test result: FAILED. 211 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.33s
error: 1 target failed:
```
Summed over the 68 `test result` lines: 479 passed, 9 failed, 488 executed. The 268 before is the implementor's `gate-test.txt`; 488 is 268 plus my 220. A first run without `--no-fail-fast` stopped after my target at 291 executed and is not counted.

**4. Findings** (tree at 5b11148)

All rows are about `crates/canon-engineering/tests/catalog_manifest.rs`. The implementation behaves correctly in every row; what is missing is a test that would notice if it stopped doing so.

| Finding | file:line | What was measured | What reaches it in practice | Verdict | Origin |
|---|---|---|---|---|---|
| No test refuses a `canon` value that is not MAJOR.MINOR.PATCH | catalog_manifest.rs:157 | m1 and m2 pass all 14 tests; the adv case is red in both | Any edit to `release()` or to this branch; nothing validates the manifest against the schema's pattern | CONFIRMED | introduced |
| No test refuses a Cargo.lock that pins no `b10x-canon`; if dropped, the pin check turns off silently | catalog_manifest.rs:130 | m4 survives | A rename of Canon's package; Canon has renamed packages before (`tests/crate_names.rs`) | CONFIRMED | introduced |
| Pin comparison is untested at patch level | catalog_manifest.rs:163 | m3 survives | A protocol needing 0.9.1 on a 0.9.0 pin; Canon has no patch release yet | CONFIRMED | introduced |
| No test for a listed file Canon does not parse | catalog_manifest.rs:184 | m5 survives | Mostly covered: `protocol_registry.rs` validates every built-in anyway | CONFIRMED | introduced |
| No test for a vocabulary entry naming another file | catalog_manifest.rs:152 | m6 survives | Unlisted still fires, so refusal is kept; only the named problem is lost | CONFIRMED | introduced |
| No test for a file nested below a protocol directory | catalog_manifest.rs:286 | m7 survives | A `drafts/` folder inside a protocol directory; `build.rs` ignores it, so only this check sees it | CONFIRMED | introduced |
| Choosing the lowest of two Canon pins is untested | catalog_manifest.rs:346 | m8 survives | Nothing found: Cargo.lock has one `b10x-canon` (0.9.0) | INFEASIBLE | introduced |
| No test refuses an uppercase sha256, which the schema forbids | catalog_manifest.rs:241 | m9 survives | A hand-typed or tool-written digest | CONFIRMED | introduced |

Fix: add the 8 cases from `scratch/adversary/killers.rs.part` to `catalog_manifest.rs`. They pass on the real code.

**5. Attacked and could not break**
- `canon` values: the probe output (90 lines) matches Canon's changelog, and the keys used by the three 0.1.0-floor files all exist in the 0.1.0 model. Order forms behave the same over whole seconds up to 0.8.0, so the 0.4.0 group holds.
- `0.10.0` against `0.9.0` is compared as numbers, and the existing test's minor+1 kills a text-compare mutant. `<` changed to `<=` is caught.
- Paths are refused: `..`, absolute, `./`, the manifest listing itself, non-`.yaml`, a subdirectory, a duplicate path.
- An empty protocols list makes every file Unlisted.
- CRLF or a trailing change while keeping id and revision is caught as Digest.
- Duplicate keys in a manifest entry are refused by serde.
- A Canon pinned by rev still carries the crate version; a pre-release version is refused as Lock.
- Working directory and target directory: the root comes from runtime `CARGO_MANIFEST_DIR`, and scratch copies use a fresh tempdir.
- The red commit 7f5240b failed for the stated reason: 14 of 14 panic at `read()` `:356` on the missing manifest or schema. The test-file changes from 7f5240b to 5b11148 are rustfmt only.
- Schema against check: everything the schema forbids, the check also refuses. The one gap goes the other way: a major above u32 passes the schema but the check refuses it, and no real file reaches it.
- No `.gitattributes`, and this machine has `core.autocrlf=input`. A CRLF working file would pass locally, but CI checks out the blob bytes and would fail. Not a defect.
- `build.rs` and the docs `sources` skip top-level files, and the registry still lists ten built-ins.

**6. Paths written outside the worktree**

Six files in the unit's scratch directory, under `adversary/`: `killers.rs.part`, `make-mutants.sh`, `probe-alone.txt`, `suite.txt`, `suite-nff.txt`, `adversary_catalog_manifest.rs` (copy of the probe).

Inside the worktree but ignored by git: `target/adversary-catalog-manifest/{orig,m1..m9}.rs`.

```findings
- file: crates/canon-engineering/tests/catalog_manifest.rs
  line: 157
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: no test refuses a canon value that is not MAJOR.MINOR.PATCH, so dropping the CanonValue branch or loosening release() at :315 keeps the suite green
- file: crates/canon-engineering/tests/catalog_manifest.rs
  line: 130
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: no test feeds a Cargo.lock without b10x-canon, so swallowing the Lock error silently disables the pin check and the suite stays green
- file: crates/canon-engineering/tests/catalog_manifest.rs
  line: 163
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the pin comparison is untested at patch level; comparing only major.minor survives
- file: crates/canon-engineering/tests/catalog_manifest.rs
  line: 184
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: no test lists a file Canon does not parse, so dropping the Unparsed problem survives
- file: crates/canon-engineering/tests/catalog_manifest.rs
  line: 152
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: no test names another file in the vocabulary entry, so dropping the Vocabulary problem survives
- file: crates/canon-engineering/tests/catalog_manifest.rs
  line: 286
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: no test puts a file below a protocol directory, so a walk that stops at depth two survives
- file: crates/canon-engineering/tests/catalog_manifest.rs
  line: 346
  category: mutant
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: choosing the lowest of two pinned b10x-canon packages is untested, and the lock pins only one today
- file: crates/canon-engineering/tests/catalog_manifest.rs
  line: 241
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: no test lists an uppercase sha256, so a case-insensitive digest comparison survives although the schema requires lowercase
```
