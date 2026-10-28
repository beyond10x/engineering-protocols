---
format: aep.planning-md/3
id: story:gate-executable-identity-bound-to-spawn
kind: story
status: draft
title: A gate command spawns the bytes admission digested
summary: a link or program re-resolved between digest and spawn can run unadmitted bytes
relations:
- decomposes: epic:gates-and-delivery-for-any-repository
- serves: vision:governed-autonomy
revision: 2
---
# Story: A gate command spawns the bytes admission digested

## Outcome

A repository owner whose gate command names a link (`executable:`) or a bare program (`program:`) gets a pass only from the bytes admission digested: a link retargeted between the digest and the spawn cannot run a program the bundle never locked.

## Context

The adversary pass on `story:gate-executable-link-keeps-its-name` found that the link is resolved three times: at admission (digest), in `verify_executables` before and after the request, and by the kernel at spawn. A swap that is undone between those reads runs bytes admission never digested, and the bundle certifies their result. A `program:` binding has had the same gap since it shipped. Exploiting it needs a process of the gate's own user that rewrites the link during collection, so it is a hardening item, not a release blocker.

The same pass also found:

- `executable_digest` reads its file to the end with no bound, so an admitted path that reaches `/dev/zero` or a FIFO never returns (`crates/assertion-providers/src/lib.rs`, `executable_digest`).
- The bundle's identity names the admitted link, not the file it reached, so two collections through one link to different targets carry the same executable name and differ only in the digest.

## Acceptance

- `a_link_swapped_between_its_checks_runs_a_program_admission_never_digested` (`crates/canon-engineering-assertions/tests/executable_link_adv1.rs`) passes without `#[ignore]`: the spawn runs the resolved file admission digested, with the admitted name as `argv[0]`, so a link swapped during collection cannot produce a pass.
- A `program:` binding resolved to a link gets the same guarantee, by a test of its own.
- Admitting an executable that is not a regular file of bounded size is refused at admission, before any digest is read.

## Out of Scope

- A hostile process of the gate's own user that rewrites the resolved file itself; the file's owner is trusted by admission.

## Ambiguities

- Whether the spawn path joins `CommandBinding` as a field (a public API change for every struct literal) or stays private to collection: inferable from the release plan, decided by whoever implements it.
