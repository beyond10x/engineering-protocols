---
format: aep.planning-md/3
id: approval-record:release-0-17-0
kind: approval-record
status: approved
title: Release 0.17.0
summary: 'Operator, 2026-11-21, in this session: release. Covers the release commit, the tag 0.17.0, the GitHub Release and its two catalog assets.'
relations:
- decides: epic:data-only-catalog
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-21T09:16:30Z", actor: "human:timo", revision: 2}
---
# Approval: release 0.17.0

## Decision

The operator wrote `release` in the engineering-protocols session on 2026-11-21. It answered this
session's question, put to him in the same session, with one recommended option: "A: type
`release 0.17.0`. Pushes a bot PR (version 0.16.0 → 0.17.0, CHANGELOG, README § Status), tag
`0.17.0`, the GitHub Release and its two catalog assets."

Heard directly in this repository's session, not relayed. Before it, the session `host-operator`
had ordered the same release as its own decision on the operator's behalf; that order was not
taken, because the `sdc` session, which decides for the operator here
(approval-record:sdc-orchestrates-engineering-protocols-2026-11-21), kept releases, tags and
version numbers with the operator himself.

## What it covers

- Release 0.17.0 of this repository from `main` at `11d2694` plus the release commit: the
  workspace version in `Cargo.toml` and `Cargo.lock`, the CHANGELOG entry, README.md § Status;
  the annotated tag `0.17.0` by `b10x-bot[bot]`; the GitHub Release; and its two assets,
  `engineering-protocols-catalog-0.17.0.tar.gz` and the same name with `.sha256` appended.
- It is the first release with `protocols/manifest.yaml` and the catalog archive, and the first
  to upload assets with `b10x-gates gh -- release upload`.

## What it does not cover

- Any later release, tag or version number.
- A consumer's pin: Loom and takt move theirs by their own decisions.
- Deploying or republishing the documentation site.
