---
format: aep.planning-md/3
id: story:agents-boundary-data-only
kind: story
status: active
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
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-21T06:56:13Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-21T06:56:13Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":2}}}
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

## Decided before dispatch (wave 2026-11-21-w4)

Settled by the coordinator; where this section and the text above differ, this section holds:

- Line numbers above are from tag 0.16.0. On `main` at `6c9491a` the acceptance grep prints the
  same 9 lines at AGENTS.md:39, 40, 229 and README.md:50, 56, 69, 78, 84, 95.
- No published release has a manifest or an archive: tag 0.16.0 predates both, and the next
  release is the operator's to order. So "running those commands as written on the story's
  commit exits 0" is measured like this: the tag route on a clean copy of the story's commit
  (`git archive` of it), and the archive route on the two files `task catalog-archive` builds
  from that commit, placed where the download command would put them. The download command
  itself is written with `<version>` and is not run; README.md says that the first release with
  a manifest and an archive is the one after 0.16.0.
- The commands a reader runs need no Rust and no tool beyond `git`, `curl` or `gh`, `tar`,
  `sha256sum` and POSIX `awk` or `sed`: a consumer of a data catalog compiles nothing.
- The registry crate and its `canon-engineering` binary still exist until
  story:protocol-check-tool lands. README.md says so once and says they are not the way to
  consume the catalog; where the assertion commands of § Assertion gates need the binary, the
  README runs it from a checkout without the option spelling the acceptance grep refuses.
- AGENTS.md § ESS: the two assertion crates are frozen at the behaviour of tag 0.16.0; the
  owner of what replaces them is Loom; decision-blocker:assertion-executor-owner (cleared) is
  the record.
- Found in wave 3 and fixed here, since this story rewrites the file: AGENTS.md § Public
  documentation names the workspace `docs` skill at a path that does not exist on the
  workstation; the file is in the Atlas repository at `.agents/skills/docs/SKILL.md`.
- Waves 1 to 3 already wrote these parts of AGENTS.md and README.md, and this story keeps them:
  the manifest's and the archive's rows in § Invariants, § Releases step 5, § Release
  completion's required artifacts, the frozen crates' and the vocabulary reader's rows, README.md
  § Assertion gates and § Crates.
