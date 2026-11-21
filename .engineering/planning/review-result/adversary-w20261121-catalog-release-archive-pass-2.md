---
format: aep.planning-md/3
id: review-result:adversary-w20261121-catalog-release-archive-pass-2
kind: review-result
status: active
title: Wave 2026-11-21-w2 adversary, story:catalog-release-archive, pass 2
relations:
- reviews: story:catalog-release-archive
revision: 1
---
unit: story:catalog-release-archive, commit 8745e6f on impl/catalog-release-archive plus one untracked test file
verdict: CONFIRMED
cases: executed 87→94, red 0
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path (the adversary scratch directory under the coordinator's session scratch)
needs-coordinator: none

ledger: carried 0 / new 1 / resolved 4

The correction holds. No manifest was found that the command and the manifest check judge
differently, apart from id, revision and `canon`, which stay with the gate by decision. The one new
finding is about the tests: five of the six new refusals can be removed and every committed test
stays green. One reason is that the pass-1 schema case stopped isolating the schema rule once
`Cargo.toml` became an input. The new file pins all five.

**1. What was touched.** One untracked file,
`crates/canon-engineering-docs/tests/adversary2_catalog_archive.rs`. No implementation file.
`cargo fmt -p canon-engineering-docs --check` exits 0 and clippy with `-D warnings` is clean.

**2. Cases added** (`crates/canon-engineering-docs/tests/adversary2_catalog_archive.rs`), 7 pins,
all green; every case keeps a valid `Cargo.toml` in the scratch root.

| Case (line) | What it asserts |
|---|---|
| `a_schema_other_than_the_fixed_one_is_refused_on_its_own` (113) | an existing `schema/other.schema.json` named as `schema` is refused, the error names it, `--out` stays empty |
| `a_protocol_directory_that_is_not_a_name_is_refused` (125) | `Software-change`, `software_change`, `1software`, `-software` are refused, each naming its path |
| `a_protocol_file_that_is_not_a_major_yaml_is_refused` (141) | `0.yaml`, `+1.yaml`, `1.yml` and a nested `software-change/v1/1.yaml` are refused |
| `a_version_that_cannot_go_in_a_file_name_is_refused` (155) | versions with a slash, a space or a newline, and the empty version, are refused naming `Cargo.toml` |
| `a_pre_release_version_with_build_metadata_is_named_verbatim` (173) | `0.17.0-rc.1+b5` produces `engineering-protocols-catalog-0.17.0-rc.1+b5.tar.gz` and a matching digest line |
| `a_package_only_cargo_toml_is_refused` (194) | a `Cargo.toml` with only `[package]` is refused naming `Cargo.toml` |
| `stray_files_under_protocols_are_refused` (207) | a second `manifest.yaml` in a subdirectory, `.keep`, a `1.yaml~` backup and `README.md` are each refused naming the path |

**3. Suite run** (`ESS_TOOLCHAIN=0.77.0 cargo test -p canon-engineering-docs --locked
--no-fail-fast`, exit 0): executed 94; `adversary2_catalog_archive` 7 passed,
`adversary_catalog_archive` 6 passed, `catalog_archive` 11 passed, 0 failed.

Six mutants of the correction in a scratch copy: N1 (schema check off), N2 (name rule reduced to
non-empty), N3 (version character rule reduced to no `/`), N4 (empty-version check off) and N5
(walk skips anything ending in `manifest.yaml`) survive the committed tests and are killed by the
new file. N6 (vocabulary-path check off) is killed by `catalog_archive.rs`.

**4. Findings**

Ledger against pass 1: `catalog_archive.rs:114` (schema and path rules weaker than the check)
resolved, the red case is green; `catalog_archive.rs:78` (dot-segment paths) resolved,
`ensure_inside` is gone and the segment comparison rejects `.`; `tests/catalog_archive.rs:251`
(mutants M1, M2, M4) resolved, the pins are committed; `catalog_archive.rs:161` (version from the
binary) resolved, it comes from the root's `Cargo.toml`.

| Finding | file:line | Measured | What reaches it | Verdict | Origin |
|---|---|---|---|---|---|
| Five of the new refusals (schema, name rule, version characters, empty version, the walk's manifest exclusion) can be removed with every committed test green. The committed pass-1 schema case still passes without the schema check: its root `Cargo.toml` is `[workspace]` with no version, so the version refusal also names `Cargo.toml`. | `crates/canon-engineering-docs/tests/adversary_catalog_archive.rs:228` | N1 to N5 leave `catalog_archive.rs` (11) and `adversary_catalog_archive.rs` (6) green; `adversary2_catalog_archive.rs` kills each | the command runs at every release; a later edit that drops one of these rules would not be caught; committing `adversary2_catalog_archive.rs` closes it | CONFIRMED | introduced |

The pass-1 file went in with only the two helper edits (lines 64 and 156-157), no assertion
changed. A fresh run gives the archive sha256
`73bacc79762696ea1e384a2054d57ea3dcf26e48c73a3fa880c98d1e379ef004`, as before the correction.

**5. Attacked and not broken.** The name and major rules and the walk agree with
`catalog_manifest.rs` on every edge case of part 2. A `[package]`-only `Cargo.toml` is refused;
TOML has no include, so the parse reads one file; `..` or a leading dot in a version is harmless
behind the fixed prefix. `pub mod builtin_name` widens nothing: the crate is `publish = false`
with no library target. Not measured: whether GitHub renames an asset whose name contains `+`.

```findings
- file: crates/canon-engineering-docs/tests/adversary_catalog_archive.rs
  line: 228
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: removing the schema, name-rule, version-character, empty-version or manifest-exclusion refusal leaves every committed test green, because the pass-1 schema case is also refused for its versionless Cargo.toml; adversary2_catalog_archive.rs pins all five
```
