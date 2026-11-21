---
format: aep.planning-md/3
id: story:assertions-frozen-at-0-16-0
kind: story
status: draft
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
revision: 5
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
