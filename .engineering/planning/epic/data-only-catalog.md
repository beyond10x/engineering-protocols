---
format: aep.planning-md/3
id: epic:data-only-catalog
kind: epic
status: draft
title: engineering-protocols is a data-only catalog that nothing compiles against
summary: A runtime takes protocol files from a tag and checks them by sha256; the Rust left here is a CI check and the docs generator.
relations:
- serves: vision:O2
revision: 3
---
## Outcome

engineering-protocols is a catalog of protocol files that nothing compiles against. A runtime
takes `protocols/<name>/<major>.yaml` and `protocols/vocabulary.yaml` from a release tag as files,
checks each against a published sha256, and links Canon itself. A Canon release then needs no
release of this repository before a runtime can move to it. The Rust that remains here is a check
run in CI, the documentation generator with the vocabulary reader it shares with the check, and,
until their replacement has an owner, the two frozen assertion crates. All of it is
`publish = false`.

## Decision

approval-record:data-only-catalog. Operator decision of 2026-11-21 ("option A"), relayed to this
repository by the `sdc` session, not heard here directly: "engineering protocols was just that, a
collection of potential flows, and then there is an example implementation for the binding, so we
could demonstrate it, but maybe this is now superseded anyway."

The research behind it is `sdc/docs/engineering-protocols-boundary.md` (2026-11-21, uncommitted in
the `sdc` repository on that day). The facts this epic's stories cite from it were re-read in this
repository at tag 0.16.0 (70e6eff).

## Why

- Releases 0.8.0 and 0.16.0 changed no protocol, fixture verdict or generated file; they exist
  because `registry::get` returns Canon's model type, so a consumer can move to a Canon release
  only after this repository has released on it (CHANGELOG.md § 0.8.0, § 0.16.0).
- A dependent that only calls `registry::get` also compiles the assertion collectors, which start
  programs and open sockets: `crates/canon-engineering/Cargo.toml:17` depends on
  `b10x-canon-engineering-assertions`, which depends on `b10x-assertion-providers`.
- The 63 test files under `crates/canon-engineering/tests/` and the 115 files under `fixtures/`
  test the protocols through Canon, not the registry. They stay.

## Stories, in order

1. story:catalog-manifest: the manifest and its CI check. Additive.
2. story:catalog-release-archive: a `task` target for the catalog archive and its digest, and the
   release step that attaches both. After 1.
3. story:assertion-catalog-path: `catalog/engineering.yaml` moves to a path that names assertions.
4. story:assertions-frozen-at-0-16-0: the assertion crates are frozen and `publish = false`, not
   deleted. After 3.
5. story:docs-generator-reads-files: the vocabulary reader gets its own crate and the generator
   stops depending on the registry crate. After 1.
6. story:agents-boundary-data-only: AGENTS.md and README.md state the new boundary. After 1, 2
   and 4.
7. story:protocol-check-tool: `canon-engineering` is deleted, one check crate remains. After 1,
   4, 5 and 6. Waits on upstream-blocker:consumers-compile-against-canon-engineering.

AGENTS.md is edited by 1, 2, 6 and 7, README.md by 4 and 6, and `Cargo.lock` by 1, 5 and 7; the
`depends_on` edges put each pair in order. 3 shares no file with 1 and can run beside it.

The plan-critic panel ran twice (review-result:data-only-catalog-*-r1 and -r2). There was no
third round: the second round's 6 findings were revised and not re-read by a critic.

## Not here

| Not in this epic | Owner |
|---|---|
| Loom loads the catalog from files and drops `b10x-canon-engineering` from four crates | the `loom` repository |
| takt reads vendored files, or stays on its pin at tag 0.9.0 | the `takt` repository |
| Observation collection as Loom executors through Substrate; where `engineering-gates/1` execution and offline replay live | not settled: decision-blocker:assertion-executor-owner |
| Deleting `b10x-canon-engineering-assertions` and `b10x-assertion-providers` | a later story, after that decision |
| Superseding Atlas ADR 0094 decision 2 and updating Atlas's catalog relations | the `atlas` repository |
| SDC's eight protocols joining the catalog | `sdc`, then a story here |

## Open

- epic:gates-and-delivery-for-any-repository (draft) and story:gate-program-identity-bound-to-spawn
  (draft) plan more work in the assertion crates this epic freezes. Under the decision that work
  belongs with whoever owns the collectors next, so decision-blocker:assertion-executor-owner
  `blocks` both. The owner is the operator's decision; the `sdc` session told him on 2026-11-21
  that it is open.
- No released consumer breaks while the tags stay: every dependent pins a tag or a lock commit
  (`sdc/docs/engineering-protocols-boundary.md` § 2; that grep was not repeated here).
