---
format: aep.planning-md/3
id: story:gate-commands-name-a-program-portably
kind: story
status: draft
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
  path: crates/canon-engineering-assertions/src/lib.rs
- confidence: inferred
  path: ess/domains/gates.yaml
- confidence: inferred
  path: fixtures/assertions
revision: 2
---
# Story: gate-commands-name-a-program-portably

## Outcome

A command in an assertions project can name a program (`program: <name>`) with a project-level `search_path`
instead of an absolute executable. At admission, the program resolves to an absolute path outside the repository
tree, and its path and sha256 are recorded with the run. The provider still starts an absolute path with a cleared
environment, so nothing ambient is trusted.

## Context

- The process provider refuses a relative executable and clears the environment (`crates/assertion-providers/src/process.rs:163-172`).
- A downstream trial (2026-10-27) wrote its type-check gate as `/usr/bin/env` with a hand-typed
  `PATH=<home>/.local/bin:<brew>/bin:/usr/local/bin:/usr/bin:/bin` argument, then the package manager. That document
  works on one host only.
- takt has an active story, `story:gate-commands-find-programs-on-a-system-path` (takt store). It gives a bare program
  name a fixed system path by default, and it leaves open whether that default lives in takt or in this library. This
  story is the library half either way:
  - an explicit, recorded `program` plus `search_path` binding that the takt default can be expressed in;
  - no default search path of its own. An omitted `search_path` keeps today's refusal of a relative executable.
  If the takt story puts its default here, it extends this binding and adds no second one.

## Acceptance

Each is a named fixture or test run by `task check`.

1. `fixtures/assertions/program-by-name.fixture.yaml`: `program: tool` with `search_path: [<dir>]` runs `<dir>/tool`,
   and the run records its absolute path and sha256.
2. `tests/process_program.rs::a_program_inside_the_tree_is_refused`: a `search_path` entry inside the repository root
   is refused at admission, naming it.
3. `tests/process_program.rs::a_missing_program_is_named`: no match is refused at admission, naming the program and
   the search path.
4. `tests/process_program.rs::executable_still_works`: every existing `executable:` command runs unchanged.

## Scope

- `crates/assertion-providers/src/process.rs`
- `crates/canon-engineering-assertions/src/lib.rs` (project admission)
- `ess/domains/gates.yaml`
- `fixtures/assertions/`

## Delivery

No downstream code change is required. takt takes it with its next engineering-protocols pin.
