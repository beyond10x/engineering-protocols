---
format: aep.planning-md/3
id: upstream-blocker:consumers-compile-against-canon-engineering
kind: upstream-blocker
status: open
title: Loom (four crates, tag 0.15.2) and takt (tag 0.9.0) compile against b10x-canon-engineering
summary: Cleared when Loom loads the catalog from files and takt reads vendored files or is declared to stay on its pin.
relations:
- blocks: story:protocol-check-tool
revision: 1
---
## What is open

Two repositories compile against `b10x-canon-engineering`:

- Loom: `b10x-loom-protocols`, `-governor`, `-intake-slice` and `-intake-router` pin tag 0.15.2.
  The `sdc` session asked the `loom` session on 2026-11-21 to load the catalog from files and
  drop the dependency.
- takt: `takt/Cargo.toml:13-14` pins tag 0.9.0, and takt is the one production caller of the
  assertion crates.

Source: `sdc/docs/engineering-protocols-boundary.md` § 2, read from those repositories' manifests
on 2026-11-21; not re-read from this repository's session.

## What clears it

A Loom release whose `Cargo.lock` has no `b10x-canon-engineering`, and for takt either the same
or the operator's word that takt stays on its pin at tag 0.9.0. Both pin tags, so neither breaks
when the crate is deleted at HEAD; the blocker stands for the consumer's next move, not its
current build.
