---
format: aep.planning-md/3
id: story:gate-program-identity-bound-to-spawn
kind: story
status: archived
title: A program binding spawns the bytes admission digested, and a digest read is bounded
summary: 'a program: match that is a link is re-resolved at spawn; executable_digest reads unbounded'
relations:
- decomposes: epic:gates-and-delivery-for-any-repository
- serves: vision:governed-autonomy
revision: 4
transitions:
- {from: "draft", to: "archived", at: "2026-11-21T02:36:46Z", actor: "human:timo", revision: 4}
---
# Story: A program binding spawns the bytes admission digested, and a digest read is bounded

## Outcome

A repository owner whose gate command names a bare `program:` gets the guarantee an `executable:` link has since 0.12.0: the file admission digested is the file that runs, and admitting a path that is not a bounded regular file is refused before any digest is read.

## Context

The adversary pass on `story:gate-executable-link-keeps-its-name` found two gaps beside the link race that `story:gate-executable-identity-bound-to-spawn` closed:

- `admit_command` (`crates/assertion-providers/src/process.rs`) sets `executable` to the search-path entry it matched, which may be a link (`~/.cargo/bin/cargo` to rustup); the spawn resolves it again.
- `executable_digest` (`crates/assertion-providers/src/lib.rs`) reads its file to the end with no bound, so a path reaching `/dev/zero` or a FIFO never returns.

## Acceptance

- A `program:` binding whose match is a link runs the file admission digested, with the program name as `argv[0]`, and a link swapped during collection cannot produce a pass (a test of its own, shaped like `a_link_swapped_between_its_checks_runs_a_program_admission_never_digested`).
- Admitting an executable that is not a regular file, or is larger than a stated bound, is refused at admission, by a test that names `/dev/zero`.

## Out of Scope

- A hostile process of the gate's own user that rewrites the resolved file itself.

## Re-homed to Loom on 2026-11-21

Not implemented here. `b10x-assertion-providers` is frozen at the behaviour of tag 0.16.0
(epic:data-only-catalog), and decision-blocker:assertion-executor-owner named Loom the owner of
observation collection. The `loom` session took this story on 2026-11-21 as its own story of the
same id, with this body's outcome, findings and acceptance, to join its observation-collection
epic. Archived here for that reason.
