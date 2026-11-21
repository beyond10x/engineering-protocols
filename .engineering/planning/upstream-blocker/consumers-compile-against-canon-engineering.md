---
format: aep.planning-md/3
id: upstream-blocker:consumers-compile-against-canon-engineering
kind: upstream-blocker
status: open
title: Loom (four crates, tag 0.15.2) and takt (tag 0.9.0) compile against b10x-canon-engineering
summary: Cleared when Loom loads the catalog from files and takt reads vendored files or is declared to stay on its pin.
relations:
- blocks: story:protocol-check-tool
revision: 4
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

## Relayed on 2026-11-21

The `sdc` session reported an operator decision made in the `sdc` session: takt is frozen as of
2026-11-21 and stays pinned to its tag. If that is confirmed in this repository, takt's half of
"What clears it" is met, and Loom's four crates are the one active compile-time consumer. It is a
relay; this blocker stays open on Loom either way.

## Confirmed on 2026-11-21

The `sdc` session, which decides for the operator in this session
(approval-record:sdc-orchestrates-engineering-protocols-2026-11-21), confirmed: takt is frozen on
its tag by the operator's decision of 2026-11-21. takt's half of "What clears it" is met. This
blocker now waits on Loom's four crates only; Loom accepted its
epic:engineering-protocols-as-vendored-data on 2026-11-21, and its stories run behind its Canon
pin wave.

## Read again on 2026-11-21 (wave 2026-11-21-w4)

Loom's four crates now pin tag 0.16.0, not 0.15.2: `crates/loom-protocols/Cargo.toml:15`,
`crates/loom-governor/Cargo.toml:19`, `crates/loom-intake-slice/Cargo.toml:18` and
`crates/loom-intake-router/Cargo.toml:15`, read in Loom's checkout at commit `c94ccbbf`. The
title and the line above keep the number they were written with. What this blocker waits on is
unchanged: the four crates still compile against `b10x-canon-engineering`. AGENTS.md § Boundary
cites this record and names no tag.
