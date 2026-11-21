---
format: aep.planning-md/3
id: story:protocol-check-tool
kind: story
status: draft
title: canon-engineering is deleted and one publish = false check crate keeps the protocol tests
summary: No registry, no build-time embedding; vocabulary check, canon check, 63 test files and 115 fixtures stay.
relations:
- decomposes: epic:data-only-catalog
- depends_on: story:catalog-manifest
- depends_on: story:docs-generator-reads-files
- depends_on: story:agents-boundary-data-only
- depends_on: story:assertions-frozen-at-0-16-0
scope:
- confidence: inferred
  path: .github/workflows/check.yml
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: inferred
  path: Taskfile.yml
- confidence: cited
  path: crates/canon-engineering
- confidence: cited
  path: crates/canon-engineering-assertions
- confidence: inferred
  path: crates/protocol-check
revision: 7
---
## Outcome

The workspace has no crate a consumer can compile a protocol out of. `b10x-canon-engineering`
(the registry, `build.rs` and its embedding, the `canon-engineering` binary) is deleted. One
`publish = false` check crate remains and holds what tests the catalog: the vocabulary check,
Canon's `check` of every protocol, the 63 test files of `crates/canon-engineering/tests/` and the
115 files under `fixtures/`.

## Context

`crates/canon-engineering` has 911 lines of source and 20,282 lines of tests
(`sdc/docs/engineering-protocols-boundary.md` § 1). The tests evaluate the protocol files against
the fixtures through Canon; only `tests/protocol_registry.rs`, `tests/crate_names.rs`,
`tests/adversary_rename.rs` and the registry adversary files test the crate itself.

It waits on its consumers: upstream-blocker:consumers-compile-against-canon-engineering. Loom
pins tag 0.15.2 in four crates and takt tag 0.9.0. Both pin tags, so deleting the crate at HEAD
breaks neither; the blocker is that a consumer that wants a newer protocol must have a file-based
way first (story:catalog-manifest, and Loom's own change).

## Order

Last of the epic. After story:docs-generator-reads-files, which moves the vocabulary reader out
of this crate into its own. After story:assertions-frozen-at-0-16-0, which marks the two
assertion crates `publish = false`. After story:agents-boundary-data-only, which rewrites the
AGENTS.md sections this story then edits.

## Decided here

- The check crate is a new directory, `crates/protocol-check` (inferred name), and
  `crates/canon-engineering/` is gone. Its tests read `protocols/` at run time from the
  repository root; there is no `build.rs` and no embedding. AGENTS.md § Work forbids a shared
  target directory because `build.rs` embeds by absolute path; with no embedding that rule goes.
- The `canon-engineering` binary also hosts the `gates` and `assertions` commands
  (README.md § Assertion gates). They belong to the frozen assertion crates, so the binary moves
  to `b10x-canon-engineering-assertions` under the same name with both commands unchanged. Ending
  those commands would change a frozen crate's behaviour and is not this story.
- `protocols list` and `protocols show` end with the registry: the protocols are files under
  `protocols/`. story:agents-boundary-data-only has already taken them out of README.md.

## Protocol first

Exempt: no protocol YAML and no fixture verdict changes. Every fixture test that passes before
passes after.

## Scope

- `crates/canon-engineering/` (deleted), `crates/protocol-check/` (new), `Cargo.toml`,
  `Cargo.lock`
- `crates/canon-engineering-assertions/`: the `canon-engineering` binary and its two commands
- AGENTS.md § Invariants (the registry, build and crate-name rows), § Work (the target-directory
  rule), § Releases (what a release is when no crate is consumed)
- Taskfile.yml, `.github/workflows/check.yml` if the gate's commands change

## Acceptance

`cargo metadata --no-deps` lists no package named `b10x-canon-engineering`, and every package it
lists is `publish = false` (the docs generator already is; story:docs-generator-reads-files and
story:assertions-frozen-at-0-16-0 mark the others; this story marks the check crate); no public
item named `registry` exists in the workspace; every test of `crates/canon-engineering/tests/` at
tag 0.16.0 either still runs under `task check` or is listed in this story's evidence with the
reason it went (it tested the registry, the embedding or the crate names); every row of AGENTS.md
§ Invariants whose "Held by" cell names a test names one that
`cargo test --workspace --locked -- --list` prints. The two rows held by review (AGENTS.md:78,
79) are not part of that check.
`git grep -n "registry\|build\.rs\|CARGO_TARGET_DIR\|crates/canon-engineering/" -- AGENTS.md`
prints nothing (8 lines at tag 0.16.0: AGENTS.md:29, 40, 61, 62, 83, 171, 185, 186), which covers
§ Work and § Releases. `canon-engineering assertions list`, built from
`b10x-canon-engineering-assertions`, prints the lines the binary of tag 0.16.0 prints, and
`canon-engineering gates validate` exits 0 on the gate of `website/docs/guides/assertions.md`;
`canon-engineering protocols list` is refused as an unknown command.
