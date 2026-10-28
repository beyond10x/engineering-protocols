---
format: aep.planning-md/3
id: story:gate-executable-link-keeps-its-name
kind: story
status: implemented
title: A gate command whose executable is a link runs under the link's name
summary: rustup's cargo proxy runs as cargo, not rustup
tags:
- priority:high
relations:
- decomposes: epic:gates-and-delivery-for-any-repository
- serves: vision:governed-autonomy
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T13:13:53Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-28T13:13:53Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-28T13:56:19Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

A gate command whose admitted executable is a symbolic link runs with the name it was admitted under as its `argv[0]`, so a multiplexing binary such as rustup's `cargo` proxy (`/usr/bin/cargo` linked to `rustup`) runs as `cargo`; admission still resolves the link for its checks and the digest.

## Context

A takt 0.17.0 run on beyond10x/tools (2026-10-28) failed its gate: `executable: /usr/bin/cargo`, a link to rustup, was canonicalized at admission (`crates/canon-engineering-assertions/src/lib.rs`, `command_paths`) and started as `/usr/bin/rustup`, which refused cargo's arguments. The repository worked around it with `executable: env`.

## Protocol first

No protocol changes: the assertions runner's behaviour does. The failing test lands first.

## Scope

- `crates/canon-engineering-assertions/src/lib.rs` (`command_paths`)
- `crates/assertion-providers/src/process.rs` (`run`)
- `crates/canon-engineering-assertions/tests/`

## Acceptance

- `an_executable_link_runs_under_its_own_name`: a command whose executable is a link to a program that prints its `argv[0]` collects the link's name, not the target's; a changed link target still invalidates collection.
