---
format: aep.planning-md/3
id: story:vocabulary-unspellable-terms
kind: story
status: draft
title: Declare the four takt protocol terms the vocabulary spelling rule refuses
summary: promoted, loop.change.deliver, loop.definition.check and loop.finding.file are listed in NOT_SPELLABLE.
relations:
- decomposes: epic:els-first-domain
- serves: vision:O2
revision: 1
---
## Outcome

Every term of every built-in except `support.triage/1` is declared in `protocols/vocabulary.yaml`.

## Today

Four terms of the protocols moved from takt (Atlas ADR 0094) break the vocabulary's spelling rule (`crates/canon-engineering/src/vocabulary.rs`: a claim id is `<subject>.<predicate>`, an action id has two dotted parts): the claim `promoted` (`self.improvement/1`) and the actions `loop.change.deliver`, `loop.definition.check`, `loop.finding.file` (`loop.improvement/1`). Canon accepts them; the files must stay byte-identical to takt's, so they cannot be renamed here. They are listed in the coverage test's `NOT_SPELLABLE` and asserted undeclared and refused by the reader.

Decision taken in wave 2026-10-26-w1 (coordinator, inside the approved item): keep the list rather than widen the spelling rule, because widening it changes what `Vocabulary::from_yaml` accepts for every consumer and was not part of the item.

## Options

- A. Widen the spelling rule (three-part action ids, one-token claim ids) and declare the four terms.
- B. Rename the four in a new major of each protocol, coordinated with takt.

## Acceptance

1. `NOT_SPELLABLE` is empty and the coverage test passes.
