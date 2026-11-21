---
format: aep.planning-md/3
id: review-result:adversary-w20261121-catalog-release-archive-pass-1
kind: review-result
status: active
title: Wave 2026-11-21-w2 adversary, story:catalog-release-archive, pass 1
relations:
- reviews: story:catalog-release-archive
revision: 1
---
unit: story:catalog-release-archive, commit 1edaaf0 on impl/catalog-release-archive plus one untracked test file (base 8be96b3)
verdict: INFEASIBLE
cases: executed 75→81, red 2
origin: introduced 4 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path (the adversary scratch directory under the coordinator's session scratch)
needs-coordinator: none

The strongest verdict is INFEASIBLE. Both red cases need a manifest that `task check` already
refuses, so nothing in the documented release path reaches them. The third finding is a set of
three guards that the unit's own tests do not exercise; the adversary's new file kills all three.

**1. What was touched.** One untracked file,
`crates/canon-engineering-docs/tests/adversary_catalog_archive.rs`. No implementation file.
`cargo fmt -p canon-engineering-docs --check` exits 0 and clippy with `-D warnings` is clean.

**2. Cases added** (`crates/canon-engineering-docs/tests/adversary_catalog_archive.rs`)

| Case | What it asserts | State |
|---|---|---|
| `a_listed_file_whose_bytes_differ_from_its_digest_is_refused_and_writes_nothing` | one extra byte in a listed file: refused, path named, `--out` stays empty | green (pin, kills M1) |
| `a_manifest_of_another_format_is_refused_and_writes_nothing` | `format: b10x-protocol-catalog/2` is refused, nothing written | green (pin, kills M2) |
| `an_out_directory_that_does_not_exist_is_created` | `--out <fresh>/a/b` works | green (pin, kills M4) |
| `another_checkout_of_the_same_tree_gives_the_same_bytes` | a copy with mode 0600, mtime 2009, relative `--root`/`--out` and another working directory gives the same bytes | green (pin) |
| `a_path_spelled_with_a_dot_component_is_refused_or_keeps_entry_order` | `protocols/./software-change/1.yaml` is refused, or members are unique and in byte order | red |
| `a_schema_other_than_the_one_the_format_names_is_refused` | `schema: Cargo.toml` is refused | red |

Red output of the file run alone (`ESS_TOOLCHAIN=0.77.0 cargo test -p canon-engineering-docs
--locked --test adversary_catalog_archive`, exit 101):

```
assertion `left == right` failed: the command accepted a non-plain path and wrote entries out of byte order
  left: ["protocols/software-change/1.yaml", "protocols/eval-gated-change/1.yaml", ...
a manifest naming schema Cargo.toml was packaged: ["Cargo.toml", "protocols/eval-gated-change/1.yaml", ...
test result: FAILED. 4 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s
```

**3. Suite run** (`ESS_TOOLCHAIN=0.77.0 cargo test -p canon-engineering-docs --locked
--no-fail-fast`, exit 101): `adversary_catalog_archive` 4 passed, 2 failed; `catalog_archive`
5 passed, 0 failed; every other binary ok; executed sum 81, 75 before the new file.

Six mutants in a scratch copy, against the unit's 5 tests: M1 (digest check made `true`), M2
(format check made `true`) and M4 (`create_dir_all` removed) survive and are killed by the new
pins. M3 (`ensure_inside` returns `Ok`) survives everything with the same outcome, because the tar
crate refuses `..` and absolute paths. M5 (gzip OS byte 3→255) and M6 (level 9→6) survive; neither
value is claimed, so they are not findings.

**4. Findings**

| Finding | file:line | Measured | What reaches it | Verdict | Origin |
|---|---|---|---|---|---|
| `schema` is packaged whatever file it names, with no digest. The command checks `format` but not the schema's fixed value, the vocabulary path, the protocol path pattern, duplicates or unlisted files, all of which `catalog_manifest.rs` refuses. | `crates/canon-engineering-docs/src/catalog_archive.rs:114` | `adversary_catalog_archive.rs:235` red: the archive holds `Cargo.toml` and no `schema/` entry | only a manifest `task check` refuses; a dirty manifest in the release checkout would get through, since nothing re-runs the check there | INFEASIBLE | introduced |
| `ensure_inside` lets through `protocols/./x` and `protocols//x` (`Path::components` drops them). The set is sorted by spelling but tar stores it normalised, so entries come out of byte order and two spellings of one path become duplicate members. | `crates/canon-engineering-docs/src/catalog_archive.rs:78` | `adversary_catalog_archive.rs:212` red | only a manifest the check refuses (`OutsideCatalog`) | INFEASIBLE | introduced |
| Three claimed behaviours are untested: the digest refusal, the format refusal, creating a missing `--out`. M1, M2 and M4 survive all 5 unit tests. | `crates/canon-engineering-docs/tests/catalog_archive.rs:251` | mutant runs in scratch | `task catalog-archive` on a fresh clone has no `target/catalog` | CONFIRMED | introduced |
| The `<version>` in the file name is the binary's `CARGO_PKG_VERSION`, not the version of the `--root` tree. Any commit after a release also produces an archive with the same name and different content. | `crates/canon-engineering-docs/src/catalog_archive.rs:161` | read in the code; no case | `--root` on another checkout, or a build between releases | INFEASIBLE | introduced |

**5. Attacked and not broken.** `..` and absolute paths, a listed directory, symlinks (stored as
regular files), backslash paths; different mtimes, modes, working directory, relative `--root` and
`--out` (identical bytes, pinned); header fields, long names, the deflate backend (miniz_oxide in
`Cargo.lock`, only this crate depends on flate2); the digest file's format; an older version's pair
left in `--out`; partial output (every refusal precedes `create_dir_all`; writes are not atomic,
an interrupted run is caught by `sha256sum -c`); the serde types accept what the check accepts; an
unlisted file under `protocols/` left out, as the story decided.

```findings
- file: crates/canon-engineering-docs/src/catalog_archive.rs
  line: 114
  category: contract-drift
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: the command packages whatever file the manifest's schema field names, without a digest, and skips the schema, vocabulary-path, path-pattern, duplicate and unlisted checks that catalog_manifest.rs makes, so it archives a tree the gate refuses
- file: crates/canon-engineering-docs/src/catalog_archive.rs
  line: 78
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: ensure_inside accepts dot and doubled-slash segments because Path components normalise them, so the entries are sorted by spelling but stored normalised, which breaks byte order and allows duplicate members
- file: crates/canon-engineering-docs/tests/catalog_archive.rs
  line: 251
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: removing the digest refusal, the format refusal or the out-directory creation leaves all five unit tests green, and adversary_catalog_archive.rs now pins each one
- file: crates/canon-engineering-docs/src/catalog_archive.rs
  line: 161
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: the archive version comes from the compiled binary rather than the root tree, so a foreign root or any commit between releases yields a same-named archive with different content
```
