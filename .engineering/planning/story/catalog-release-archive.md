---
format: aep.planning-md/3
id: story:catalog-release-archive
kind: story
status: implemented
title: Each release carries the catalog as one archive with its sha256
summary: protocols/ and its manifest as a reproducible archive attached to the GitHub Release.
relations:
- decomposes: epic:data-only-catalog
- depends_on: story:catalog-manifest
- serves: vision:O2
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Cargo.lock
- confidence: inferred
  path: Taskfile.yml
- confidence: cited
  path: crates/canon-engineering-docs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T04:47:03Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-21T04:47:03Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "active", to: "implemented", at: "2026-11-21T05:29:08Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":7,"verification":2}}}
---
## Outcome

A `task` target builds the catalog as one archive, `protocols/` with its manifest, and the
archive's sha256. AGENTS.md § Releases makes attaching both to the GitHub Release a step of every
release from the next one on, so a consumer that does not clone the repository downloads one file
and checks one digest.

## Context

AGENTS.md § Releases: releases are source releases at bare-version tags, "there is no release
workflow", and the GitHub Release is created by the bot with notes only. No release has an asset
today. AGENTS.md § Release completion names "required artifacts"; this story is what first gives
that phrase a member.

This story cuts no release. The first release after it is the first with the two assets, and
that release's own completion check (AGENTS.md § Release completion) is where the attached digest
is compared with the attached archive.

## Open

- UNMAPPED: the archive's name and layout (`engineering-protocols-catalog-<version>.tar.gz` is an
  inferred name) and whether `vocabulary.yaml` is inside `protocols/` in it, as in the tree.
- Who builds it at a release: the bot from a local checkout of the tag, as the tag itself is made
  today, or a workflow on the tag push. This story takes the first; a workflow is a later change
  to the release rules.

## Order

After story:catalog-manifest, whose file the archive carries. It edits AGENTS.md § Releases
before story:agents-boundary-data-only and story:protocol-check-tool do; both depend on it,
directly or through the other. README.md is not in this story's surface: how to fetch and check
the archive is written by story:agents-boundary-data-only.

## Protocol first

Exempt: no protocol YAML and no fixture verdict changes.

## Scope

- Taskfile.yml: one target that builds the archive and its digest from a tree
- AGENTS.md § Releases: one more step, and § Release completion's "required artifacts" named

## Acceptance

Run twice on one commit, the `task` target writes an archive with the same sha256 both times and
a digest file that equals the archive's sha256; every file in the archive has the sha256 the
manifest in it lists, checked by a test that unpacks the target's output; AGENTS.md § Releases
names the step and both asset names, so a release without the two assets is not reported as
complete.

## Decided before dispatch (wave 2026-11-21-w2)

The two open points above, settled by the coordinator:

- The archive is `engineering-protocols-catalog-<version>.tar.gz`, `<version>` being the
  workspace version, and its digest file is the same name with `.sha256` appended, one line in
  `sha256sum` format. The archive has no top-level directory: it holds `protocols/` as in the
  tree, `vocabulary.yaml` and `manifest.yaml` included, and the schema file the manifest names,
  `schema/b10x.protocol-catalog.v1.schema.json`, so every path in the manifest resolves from the
  directory the archive is unpacked into.
- It is built by a Rust command, because anything that runs here is Rust (AGENTS.md
  § Invariants): `canon-engineering-docs catalog-archive --out <dir>`, in the `publish = false`
  tool crate that stays. `task catalog-archive` runs it into `target/catalog/`. At a release the
  bot builds it from a checkout of the tag and attaches both files; no workflow.
- Reproducible means: entries in byte order of their paths, each with mode 0644, owner and
  group 0 with empty names, modification time 0, and a gzip header with no time and no name.

Scope therefore also holds `crates/canon-engineering-docs/` (the command and its tests,
`Cargo.toml`) and `Cargo.lock` (the `tar` and `flate2` crates). story:docs-generator-reads-files
edits the same crate and `Cargo.lock` in the next wave of the plan.

## Delivered (wave 2026-11-21-w2)

- `canon-engineering-docs catalog-archive --out <dir> [--root <dir>]`
  (`crates/canon-engineering-docs/src/catalog_archive.rs`), and `task catalog-archive`, which runs
  it into `target/catalog/`. It writes `engineering-protocols-catalog-<version>.tar.gz` and the
  same name with `.sha256` appended, one `sha256sum` line. `<version>` is
  `[workspace.package] version` of the root's `Cargo.toml`.
- The archive holds the files `protocols/manifest.yaml` lists and the manifest, 13 entries at
  0.16.0, at repository-relative paths with no top-level directory: plain entries in byte order,
  mode 0644, owner and group 0 with no names, time 0, gzip header with no time and no name.
- Refused before anything is written, naming the path: a manifest of another format; a `schema`,
  `vocabulary.path` or protocol path that the manifest check
  (`crates/canon-engineering/tests/catalog_manifest.rs`) refuses; a path listed twice; a listed
  file that is missing or whose sha256 differs; a file under `protocols/` the manifest does not
  list; a root with no workspace version, or one that cannot go in a file name. Id, revision and
  `canon` are not re-checked: they need Canon and stay with `task check`.
- Tests: `tests/catalog_archive.rs` (11), `tests/adversary_catalog_archive.rs` (6),
  `tests/adversary2_catalog_archive.rs` (7) in `crates/canon-engineering-docs/`.
- `AGENTS.md`: § Releases step 5 (the archive is built from a checkout of the tag and both files
  are attached with `b10x-gates gh -- release upload`), § Release completion names the two assets
  as the required artifacts, § Invariants has the archive's row, § Work says which GitHub writes
  go through `b10x-gates api` and which through `b10x-gates gh`.
- Not done here, by the story's own boundary: no release carries the archive yet. The first one
  is the release after 0.16.0, which is the operator's to order. `b10x-gates gh -- release upload`
  is admitted by the gates source (`release` is one of its six command words) and has not been run
  for this repository. Not measured: whether GitHub keeps a `+` in an asset name (a version with
  build metadata).
