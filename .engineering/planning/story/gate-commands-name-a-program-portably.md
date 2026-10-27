---
format: aep.planning-md/3
id: story:gate-commands-name-a-program-portably
kind: story
status: active
title: A gate command names a program resolved at admission and recorded
summary: program plus search_path, resolved outside the tree, sha256 recorded
tags:
- priority:medium
relations:
- serves: vision:governed-autonomy
- decomposes: epic:gates-and-delivery-for-any-repository
scope:
- confidence: inferred
  path: crates/assertion-providers/src/process.rs
- confidence: inferred
  path: crates/assertion-providers/tests/process_program.rs
- confidence: inferred
  path: crates/canon-engineering-assertions/src/lib.rs
- confidence: inferred
  path: crates/canon-engineering/tests/assertion_replay.rs
- confidence: inferred
  path: ess/domains/gates.yaml
- confidence: inferred
  path: fixtures/assertions/program-by-name.fixture.yaml
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-27T12:14:01Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-27T12:14:01Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
---
# Story: gate-commands-name-a-program-portably

## Outcome

A command in an assertions project can name a program (`program: <name>`) instead of an absolute executable.
- At admission, the name resolves on a `search_path`, in this order:
  1. the project's own `search_path`;
  2. otherwise a default `search_path` the runtime passes in when it admits the project.
- The resolved path must be absolute and outside the repository tree. It is recorded with its sha256 with the run.
- The provider still starts an absolute path with a cleared environment, so nothing ambient is trusted. Neither the
  caller's `PATH` nor any default of this library's own is ever used.

## Decision

Program resolution is owned here, in the assertions library. takt's active `story:gate-commands-find-programs-on-a-system-path`
(takt store) asks whether its bare-name default belongs in takt or here. The answer: the binding is here, and takt
passes its fixed system path as the runtime default. takt records that it waits on this story as
`dependency-blocker:gate-programs-resolve-by-name`. There is one binding, not two.

## Context

- The process provider refuses a relative executable and clears the environment (`crates/assertion-providers/src/process.rs:163-172`).
- A downstream trial (2026-10-27) wrote its type-check gate as `/usr/bin/env` with a hand-typed
  `PATH=<home>/.local/bin:<brew>/bin:/usr/local/bin:/usr/bin:/bin` argument, then the package tool. That document
  works on one host only.

## Acceptance

Each is a named fixture or test run by `task check`.

1. `fixtures/assertions/program-by-name.fixture.yaml`, loaded from `crates/canon-engineering/tests/assertion_replay.rs`:
   `program: tool` with `search_path: [<dir>]` runs `<dir>/tool`, and the run records its absolute path and sha256.
2. `crates/assertion-providers/tests/process_program.rs::a_runtime_default_search_path_is_used`: a project with no
   `search_path`, admitted with a runtime default `[<dir>]`, runs `<dir>/tool`.
3. `crates/assertion-providers/tests/process_program.rs::no_search_path_keeps_the_refusal`: a project with no
   `search_path`, admitted with no runtime default, refuses `program: tool` at admission as a relative executable is
   refused today, naming the program.
4. `crates/assertion-providers/tests/process_program.rs::a_program_inside_the_tree_is_refused`: a `search_path`
   entry inside the repository root is refused at admission, naming it.
5. `crates/assertion-providers/tests/process_program.rs::a_missing_program_is_named`: no match is refused at
   admission, naming the program and the search path.
6. `crates/assertion-providers/tests/process_program.rs::executable_still_works`: every existing `executable:`
   command runs unchanged.

## Scope

- `crates/assertion-providers/src/process.rs`
- `crates/assertion-providers/tests/process_program.rs`
- `crates/canon-engineering-assertions/src/lib.rs` (project admission, runtime default)
- `crates/canon-engineering/tests/assertion_replay.rs`
- `ess/domains/gates.yaml`
- `fixtures/assertions/program-by-name.fixture.yaml`

## Delivery

The consumer is takt's `story:gate-commands-find-programs-on-a-system-path`. It is delivered in one workspace with an
uncommitted `[patch]` path override in takt's `Cargo.toml`, then this repository releases, and takt takes it as a
pin-only hop.
