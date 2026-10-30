---
format: aep.planning-md/3
id: story:forge-neutral-delivery
kind: story
status: implemented
title: Public protocols name no organisation's delivery route
summary: pr.open opens its pull request through the run's forge, not through one organisation's bot route
relations:
- decomposes: epic:gates-and-delivery-for-any-repository
- serves: vision:governed-autonomy
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-30T03:43:52Z", actor: "human:timo", revision: 3, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-30T03:43:53Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-10-30T03:43:53Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":2}}}
---
## Outcome

No built-in protocol a runtime can still move to and no vocabulary entry names an organisation's
delivery route. `pr.open` in `story.delivery/2` and in `protocols/vocabulary.yaml` opens its pull
request through the run's forge, so a coding agent that reads the action's description is not
offered one organisation's bot as a way to deliver.

## Context

A trial's step prompt for `story.delivery/1` carried `pr.open`'s description, "push the
implementation revision ... and open its pull request through the repository's bot route". The
bot is one organisation's internal delivery account; a public protocol has no business naming it.
approval-record:forge-neutral-delivery records the direction.

`story.delivery/1` is not edited: AGENTS.md § Boundary keeps it takt 0.12.0's file byte for byte
until a new major is agreed with takt, and `takt_protocols_builtin.rs` checks its Git blob id. Its
successor is `story.delivery/2`. Changing `/1` needs takt's agreement first; the guard test names
it as the one exception.

## Protocol first

Exempt: the change is to action descriptions only. No precondition, effect, capability, evidence
kind or fixture verdict changes, so there is no behaviour for a failing fixture to pin. The guard
test `forge_neutral.rs` lands with the wording and was shown red against the old wording.

## Scope

- `protocols/story-delivery/2.yaml` (`pr.open` description)
- `protocols/vocabulary.yaml` (`pr.open` meaning)
- `crates/canon-engineering/tests/forge_neutral.rs`
- the generated protocol page, graph and vocabulary page
- `website/static/showcase/2026-10-04/` (the forge write credential's label)

## Acceptance

- `forge_neutral.rs::built_in_protocols_and_vocabulary_name_no_bot_route` passes: no file under
  `protocols/` but the frozen `story-delivery/1.yaml` contains "bot route", "b10x-gates",
  "b10x-bot" or the word "bot".
- `forge_neutral.rs::pr_open_opens_through_the_runs_forge` passes for `story.delivery/2` and the
  vocabulary.
- `takt_protocols_builtin.rs` passes unchanged, every existing fixture passes unchanged, and
  `canon-engineering-docs generate --check` is clean.
