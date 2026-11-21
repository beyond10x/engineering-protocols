---
format: aep.planning-md/3
id: story:assertions-frozen-at-0-16-0
kind: story
status: implemented
title: The assertion crates are frozen at tag 0.16.0, not deleted
summary: Replay and the collectors stay until an executor exists elsewhere; they take no new behaviour meanwhile.
relations:
- decomposes: epic:data-only-catalog
- depends_on: story:assertion-catalog-path
- serves: vision:O2
scope:
- confidence: cited
  path: README.md
- confidence: cited
  path: crates/assertion-providers/Cargo.toml
- confidence: cited
  path: crates/assertion-providers/README.md
- confidence: cited
  path: crates/canon-engineering-assertions/Cargo.toml
- confidence: cited
  path: crates/canon-engineering-assertions/README.md
- confidence: cited
  path: crates/canon-engineering-docs/src/status.rs
- confidence: cited
  path: website/data/status.json
- confidence: cited
  path: website/docs/guides/assertions.md
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T05:42:25Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "proposed", to: "active", at: "2026-11-21T05:42:26Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "active", to: "implemented", at: "2026-11-21T06:44:03Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":11,"verification":2}}}
---
## Outcome

`b10x-canon-engineering-assertions` and `b10x-assertion-providers` are frozen as they are at tag
0.16.0: they stay in the tree, build and pass their tests, take no new behaviour, and are marked
`publish = false`. Each crate's README and README.md say so and say why. They are not deleted by
this story.

## Context

Offline replay (`evaluate`, `crates/canon-engineering-assertions/src/lib.rs:754`) and the
collectors that start programs and open sockets (`crates/assertion-providers/src/process.rs:171`,
`src/network.rs:28,59`) have no other home yet. takt is the one production caller of
`prepare`, `collect` and `evaluate` (`sdc/docs/engineering-protocols-boundary.md` § 2, takt at its
pin on tag 0.9.0). takt depends on them by Git tag, which `publish = false` does not affect.

Where `engineering-gates/1` execution and replay go is not settled:
decision-blocker:assertion-executor-owner. This story does not wait on that decision. Deleting
the two crates is a later story, not drafted, that does.

## What frozen means

- No new assertion function, provider, CLI command or wire-model field.
- A Canon or ESS pin still moves when the workspace's does, because the crates share its lockfile.
- A security defect is still fixed.
- The READMEs say the replacement's owner is not decided and name the decision-blocker; they are
  corrected by the story that follows the decision.

## Planned work this stops

Drafts that add behaviour to the frozen crates wait on decision-blocker:assertion-executor-owner,
which now `blocks` the first two. This story changes none of them:

- epic:gates-and-delivery-for-any-repository (draft)
- story:gate-program-identity-bound-to-spawn (draft)
- story:ess-conformance-evidence (draft) names an evidence kind, not the crates; listed because
  AGENTS.md § ESS cites it

## Order

After story:assertion-catalog-path, which changes one `include_str!` path in a crate this story
freezes. AGENTS.md is not in this story's surface: story:agents-boundary-data-only, which depends
on this one, writes the frozen state into AGENTS.md § ESS.

## Protocol first

Exempt: no behaviour changes.

## Scope

- `crates/canon-engineering-assertions/README.md` (new; the crate has none),
  `crates/assertion-providers/README.md`
- `crates/canon-engineering-assertions/Cargo.toml`, `crates/assertion-providers/Cargo.toml`:
  `publish = false`
- README.md § Assertion gates

## Acceptance

`crates/canon-engineering-assertions/README.md` exists, and it and
`crates/assertion-providers/README.md` each state that the crate is frozen at the behaviour of tag
0.16.0 and name decision-blocker:assertion-executor-owner; both crates' manifests carry
`publish = false`; README.md § Assertion gates states the freeze and no longer lists Kubernetes
providers and AEP acceptance integration as planned; the public items of both crates are the ones
tag 0.16.0 has: `cargo public-api --simplified -p b10x-canon-engineering-assertions` and the same
for `-p b10x-assertion-providers`, run at tag 0.16.0 and at the story's commit with one version of
`cargo-public-api` (not installed on the workstation today; the story records the version it
used), print the same lines, and both outputs are in the story's evidence.

## Decided before dispatch (wave 2026-11-21-w3)

Settled by the coordinator; where this section and the text above differ, this section holds:

- decision-blocker:assertion-executor-owner was cleared on 2026-11-21 after this story was
  written: the owner of what replaces the two crates is Loom (observation collection and gates
  as executors through Substrate, design not started). The READMEs and README.md say that, and
  still name the decision-blocker as the record of the decision. They do not say "not decided".
- `cargo-public-api` is installed into the unit's scratch directory, not into the workstation's
  cargo home, and run on the nightly toolchain the workstation already has. The tag's side is
  measured on a `git archive 0.16.0` copy inside the unit tree's `target/`, never by checking the
  tag out in a worktree. The story's evidence records the tool's version and the nightly's.
- The two public-API listings are kept as text under the wave's scratch and summarised in an
  evidence record by the coordinator: files under `.engineering/evidence/` are records only.

## Delivered (wave 2026-11-21-w3)

- `publish = false` in `crates/canon-engineering-assertions/Cargo.toml` and
  `crates/assertion-providers/Cargo.toml`. No source or test file of either crate changed.
- `crates/canon-engineering-assertions/README.md` (new) and `crates/assertion-providers/README.md`
  state the freeze at the behaviour of tag 0.16.0, what frozen means, why, and that Loom owns
  what replaces the crates (decision-blocker:assertion-executor-owner, cleared). Each lists the
  files a Canon or ESS pin move still edits, from the repository root, checked against the last
  ESS move (commit 708b84f, 10 files).
- `README.md` § Assertion gates states the freeze and no longer lists Kubernetes providers and
  AEP acceptance integration as planned; § Crates marks both crates frozen.
- Beyond the story's scope, by the coordinator's decision after adversary pass 1, because the
  freeze made them false: the status record's planned item "AEP acceptance and Kubernetes
  providers" is removed (`crates/canon-engineering-docs/src/status.rs`,
  `website/data/status.json` regenerated), and `website/docs/guides/assertions.md` states the
  freeze instead of listing those integrations as planned.
- Public items, the acceptance's measurement: `cargo public-api --simplified` with
  cargo-public-api 0.52.0 on rustc 1.101.0-nightly (0abfedbc7 2026-10-02), at a `git archive` of
  tag 0.16.0 and at the unit: `b10x-canon-engineering-assertions` 345 lines, sha256
  `3bbc0f96136e1641c12d927dcb07ebdb05ad68c7df710b13cb71d0e0cf00198b` on both sides;
  `b10x-assertion-providers` 338 lines, sha256
  `88c97e66ccc4252a6a270b714bf45c4f5d0d24ae1d58d6379cb455f5428ee0f4` on both sides. The listings
  show signatures; that behaviour is unchanged rests on the source diff between the tag and the
  unit, which is one `include_str!` path (story:assertion-catalog-path), the catalog file being
  the same blob at both paths. The four listings were kept in the coordinator's session scratch,
  not in the repository.
- `AGENTS.md` § Invariants has the freeze's row. § ESS still describes the crates as before:
  story:agents-boundary-data-only writes the frozen state there.
