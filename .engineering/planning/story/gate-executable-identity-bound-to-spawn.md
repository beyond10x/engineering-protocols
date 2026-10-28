---
format: aep.planning-md/3
id: story:gate-executable-identity-bound-to-spawn
kind: story
status: implemented
title: A gate command spawns the bytes admission digested
summary: a link resolved again at spawn could run bytes admission never digested
relations:
- decomposes: epic:gates-and-delivery-for-any-repository
- serves: vision:governed-autonomy
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T14:19:31Z", actor: "human:timo", revision: 5}
- {from: "proposed", to: "active", at: "2026-10-28T14:19:31Z", actor: "human:timo", revision: 6}
- {from: "active", to: "implemented", at: "2026-10-28T14:35:41Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1}}}
---
# Story: A gate command spawns the bytes admission digested

## Outcome

A repository owner whose gate command names an executable link gets a pass only from the bytes admission digested: a link swapped between the digest and the spawn cannot run a program the bundle never locked.

## Context

The adversary pass on `story:gate-executable-link-keeps-its-name` (release 0.11.1) found that the link was resolved three times: at admission (digest), in `verify_executables` before and after the request, and by the kernel at spawn. A swap undone between those reads ran bytes admission never digested, and the bundle certified their result.

Admission now resolves the link once (`command_paths`): `executable` becomes the file it reached, which is what is digested and what runs, and the admitted path is kept in `CommandBinding::argv0`, which `process::run` passes as `argv[0]`. A link retargeted after planning still refuses collection, and the links enter the context identity.

## Acceptance

- `a_link_swapped_between_its_checks_runs_a_program_admission_never_digested` (`crates/canon-engineering-assertions/tests/executable_link_adv1.rs`) passes without `#[ignore]`.
- `an_executable_link_runs_under_its_own_name` still passes, with a compiled program that prints its `argv[0]` (a shell script's `$0` names the file executed, not `argv[0]`).

## Out of Scope

- The same gap for a `program:` binding and the unbounded digest read: `story:gate-program-identity-bound-to-spawn`.
- A hostile process of the gate's own user that rewrites the resolved file itself.

## Second adversary pass (2026-10-28)

The pass found two gaps in the first fix: the links entered the identity keyed by link, so two commands that swapped their links kept one identity, and a script reached through a link saw its target as `$0`. Under approval-record:gate-link-identity-and-script-name, the links enter the identity keyed by binding, and a script is spawned through its link so it keeps the link as its name; a program is still spawned at the file admission resolved. The cases are in `crates/canon-engineering-assertions/tests/executable_link_adv2.rs`, and the assertions guide says how a link runs.
