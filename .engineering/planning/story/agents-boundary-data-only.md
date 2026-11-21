---
format: aep.planning-md/3
id: story:agents-boundary-data-only
kind: story
status: draft
title: AGENTS.md and README.md state the data-only boundary
summary: No line tells a runtime to pin b10x-canon-engineering; the stale Loom tag in the boundary table goes.
relations:
- decomposes: epic:data-only-catalog
- depends_on: story:catalog-manifest
- depends_on: story:assertions-frozen-at-0-16-0
- depends_on: story:catalog-release-archive
- serves: vision:O2
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: README.md
revision: 6
---
## Outcome

AGENTS.md and README.md state the boundary the operator decided on 2026-11-21
(approval-record:data-only-catalog): the repository is a catalog of protocol files, a runtime
takes the files from a tag or the release archive and checks them against the manifest, and no
line tells a runtime to pin `b10x-canon-engineering`.

## Context

Two statements in AGENTS.md § Boundary are wrong under the decision, and one is wrong today:

- AGENTS.md:39-41: "A runtime pins a release of `b10x-canon-engineering` and reads them through
  `registry::get`". That is Atlas ADR 0094 decision 2, which the decision replaces.
- AGENTS.md:51: Loom's `loom-governor`, `loom-intake-router` and `loom-intake-slice` crates
  "depend on this crate at tag 0.1.0". Loom pins tag 0.15.2 in four crates, `loom-protocols`
  among them (`sdc/docs/engineering-protocols-boundary.md` § 2, read from Loom's manifests on
  2026-11-21).

AGENTS.md:209 (§ Releases): "Consumers pin `b10x-canon-engineering` by `tag`". README.md § Install
(lines 46-56) and § Smallest example (61-104) tell a reader to depend on the crate and call
`registry::list` and `registry::get`. `website/docs/` and `website/product.json` repeat none of
it (`grep -rln "registry::\|b10x-canon-engineering"` over both prints nothing).

## Order

After story:catalog-manifest and story:catalog-release-archive: the text describes the manifest
and the archive as the way to consume the catalog, and the archive story writes AGENTS.md
§ Releases first. After story:assertions-frozen-at-0-16-0: this story writes the frozen state
into AGENTS.md § ESS, and both edit README.md. Until story:protocol-check-tool lands, the text
says the registry crate still exists and is not the way to consume the catalog.

## Protocol first

Exempt: documentation only.

## Scope

- AGENTS.md § Serves (its last sentence, "Loom's `loom-governor` consumes these protocols"),
  § Boundary, § ESS (the assertion crates are frozen), § Releases (the consumers line)
- README.md § What it is not, § Install, § Smallest example, § Crates; the workspace `docs`
  skill applies to README.md

## Acceptance

`git grep -n "b10x-canon-engineering = \|--tag [0-9.]* b10x-canon-engineering\|-p b10x-canon-engineering \|registry::\|pins a release of\|Consumers pin" -- AGENTS.md README.md`
prints nothing (it prints 9 lines at tag 0.16.0: AGENTS.md:39, 40, 209 and README.md:50, 56, 69,
78, 84, 95); README.md has a section, in place of § Install and § Smallest example, that tells a
reader to take the protocol files from a release tag or from the release archive and to check
each file's sha256 against `protocols/manifest.yaml`, with the commands that do it, and running
those commands as written on the story's commit exits 0; AGENTS.md § Boundary names the manifest
and its sha256 check as the way a runtime reads a
protocol, cites approval-record:data-only-catalog, and says Atlas ADR 0094 decision 2 is
replaced, naming the Atlas ADR that records it or the words "not yet recorded in Atlas";
AGENTS.md's "Not here" row for Loom states no tag; AGENTS.md § ESS states that the two assertion
crates are frozen at the behaviour of tag 0.16.0 and names
decision-blocker:assertion-executor-owner.
