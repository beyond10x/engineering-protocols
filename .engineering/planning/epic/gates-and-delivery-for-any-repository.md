---
format: aep.planning-md/3
id: epic:gates-and-delivery-for-any-repository
kind: epic
status: draft
title: A repository that is not ours states its definition of done and its delivery variant in our documents
summary: change-aware gates, story.delivery/2 settings, portable gate programs
relations:
- serves: vision:governed-autonomy
revision: 1
---
# Epic: gates-and-delivery-for-any-repository

## Outcome

A repository that is not ours can say its definition of done, and choose its delivery variant, in this repository's
documents:
- An `engineering-gates/1` gate can test what changed since the case's base, and can declare when it applies, so
  "a consumer-visible change carries a release note" is a gate, not a hope.
- A gate command names its program portably, so a gates document is not tied to one host's absolute paths.
- `story.delivery/2` declares its variants as case inputs, `tests_first` and `gate_on`, that a runtime sets from the
  repository's own configuration.

## Context

A downstream trial of takt on a multi-language repository (2026-10-27) found three things.
- Three of three candidates handed over without the changelog, release-note and documentation changes the
  repository's rules require. The catalog has no operation that sees the change since the base, so no gate could
  catch it (`catalog/engineering.yaml`: fs, document, process, tests, ess, generated, codegate, net, text).
- `story.delivery/1` does not require a red test before the implementation edit
  (`protocols/story-delivery/1.yaml:132-134`).
- The gate runs on every revision, including the known-red test-only one, because `gate.run` has no precondition
  (`:160-164`).

Gate commands also need a hand-written absolute executable and `PATH` (`crates/assertion-providers/src/process.rs:163-172`).

## Stories

- `story:a-gate-sees-the-change-and-may-not-apply`
- `story:story-delivery-2-with-settings`
- `story:gate-commands-name-a-program-portably`

## Out of scope

- takt's side: passing the base, the subject and the story to the gate runner, setting the inputs from its run file,
  and showing each obligation. Those are in takt's store.
