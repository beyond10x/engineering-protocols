---
format: aep.planning-md/3
id: story:mutant-tests-run-with-any-target-dir
kind: story
status: implemented
title: The mutant tests pass with a target directory outside the repository
summary: the seven mutant tests compile their mutants from text, so an outside CARGO_TARGET_DIR does not fail them
relations:
- decomposes: epic:els-first-domain
- serves: vision:governed-autonomy
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T22:37:03Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-28T22:37:04Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-10-28T22:38:43Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":2}}}
---
## Outcome

The seven mutant tests in `adversary_takt_protocols.rs` (3), `adversary2_takt_protocols.rs` (3)
and `adversary_software_change_profiles.rs` (1) pass when `CARGO_TARGET_DIR` points outside the
repository, as they do with the checkout's own `target/`.

## Context

Each test wrote its mutant protocols under `CARGO_TARGET_TMPDIR` and compiled them through
`support::compile_protocol`, which reads only paths confined to the repository. With a target
directory outside the repository, the mutant files were outside it too, and every one of the seven
tests panicked before judging a mutant. A mutant needs no file: it is compiled from its text, through
the same Canon parse, validation and IR compilation as a protocol read from the repository.

## Protocol first

No behaviour change: no protocol, fixture or expectation changes, only how the tests obtain their
mutants. The failing run is the seven tests under an outside `CARGO_TARGET_DIR` before the change.

## Scope

- `crates/canon-engineering/tests/support/mod.rs` (`compile_protocol_text`)
- `crates/canon-engineering/tests/adversary_takt_protocols.rs`
- `crates/canon-engineering/tests/adversary2_takt_protocols.rs`
- `crates/canon-engineering/tests/adversary_software_change_profiles.rs`

## Acceptance

- With `CARGO_TARGET_DIR` outside the repository, the three test targets pass: 7 passed, 0 failed.
- With the checkout's own `target/`, the same three targets pass unchanged, and they write no
  mutant file.
