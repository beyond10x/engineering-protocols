---
format: aep.planning-md/3
id: review-result:adversary-w20261017-eval-gated-change-pass-1
kind: review-result
status: active
title: Wave 2026-10-17-w1 adversary, story:eval-gated-change-protocol, pass 1
relations:
- reviews: story:eval-gated-change-protocol
revision: 1
---
Independent adversary on `eval.gated_change/1` at the unit head `79dcdca`; cases in
`crates/canon-engineering/tests/adversary_eval_gated_change.rs` (`c46ab63`, 15 cases). The protocol made no
wrong decision under attack: no keep or reject from the loop's own run or an unlisted producer, UNKNOWN never
became a decision, a basis move left no keep claim true, and kept, rejected and proposed were never legitimate
together. The gaps were in the fixtures: mutants of the YAML survived because no fixture state exercised them.

```findings
[
 {"file":"protocols/eval-gated-change/1.yaml","line":153,"category":"mutant","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"each of change.reject's six not-unknown guards can be dropped alone with every fixture and canon check still passing"},
 {"file":"protocols/eval-gated-change/1.yaml","line":192,"category":"mutant","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"the rejected outcome's five not-unknown guards survive removal because no fixture has the candidate within the surface with another keep claim unknown"},
 {"file":"protocols/eval-gated-change/1.yaml","line":81,"category":"mutant","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"tuning.improved read as 'evidence reproduced' survives because no fixture records not_reproduced"},
 {"file":"protocols/eval-gated-change/1.yaml","line":176,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"change.apply_outside_surface is admissible whenever a case grants loop_change.apply_outside_surface; 'never granted to a loop' rests on the case's authority input, not the protocol"},
 {"file":"protocols/eval-gated-change/1.yaml","line":214,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"kept and inconclusive can be legitimate together; acceptance 13 leaves inconclusive out of the exclusive set"},
 {"file":"fixtures/eval-gated-change/proposed.fixture.yaml","category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"in state outside-decided change.reject is admissible while the rejected outcome is blocked, by design"}
]
```

Corrections: fixture states that kill each surviving mutant (the three warnings and the note on line 81).
The three judgement notes are kept as stated limits of /1: Canon grants capabilities through the case's
authority input, as for every other protocol's capability; acceptance 13 excludes `inconclusive` by design;
`change.reject` without `rejected` is the out-of-surface case, where `proposed` closes it.
