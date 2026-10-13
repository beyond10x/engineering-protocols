---
format: aep.planning-md/3
id: review-result:adversary-w20261013-software-change-profiles-pass-1
kind: review-result
status: active
title: Wave 2026-10-13-w1 adversary, story:software-change-profiles, pass 1
relations:
- reviews: story:software-change-profiles
revision: 1
---
needs-revision

Adversary (`aep:adversary`) on `a51a581..51fb753` of wave 2026-10-13-w1, story:software-change-profiles. Test commit `95cde62` (`adversary_software_change_profiles.rs`, 5 cases; `adversary_graph_case_inputs.rs`, 2 cases); red 2. Excerpt of its report: the findings and what it could not break.

## Findings

- **F1, a normal red-first bugfix gets stuck at R1.** Measured `adversary_software_change_profiles.rs:146`, exit 101. Canon treats a pass and a fail on the same revision as a contradiction (`evidence_match_in`: both counts non-zero gives `Unknown`), and `tests.write` does not change the implementation revision. A bugfix that runs the suite green, adds its regression test and runs it red records both against R1; `regression.reproduced` is UNKNOWN and `implementation.edit` stays blocked on R1.
- **F2, nothing tests what discharges the four obligations.** Measured `adversary_software_change_profiles.rs:354`, exit 101. Four mutants, each pointing one obligation's `discharged_when` at another claim, pass every `software.change/1` fixture; `profiles` is the only fixture expecting obligation statuses and has no evidence.
- Note: nothing holds `change_kind` fixed across evaluations of one case; Canon keeps no case history.

## Attacked and not broken

A bugfix edit without a failing test about the current revision (no evidence, pass, stale revision, wrong subject, no result, rejected review, R1 red at R2); an obligation `open` or `discharged` outside its profile; values outside the closed lists, missing or undeclared inputs; the reader's dotted obligation ids and new categories; `graph::document` emitting an empty or repeated `in` list, `case` without inputs, `applicable_when` off an obligation; the schema transcription accepting what the 0.9.0 schema rejects.

## Outcome (coordinator)

F1 kept as a stated limitation (`15fc26c`: the case pins it, `step-order.mdx` states it); F2 fixed (`15fc26c`, fixture `profiles-discharged`).

```findings
- file: protocols/software-change/1.yaml
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: a bugfix that records a pass and then a fail on the same implementation revision leaves regression.reproduced UNKNOWN, so implementation.edit stays blocked on that revision with no way forward
- file: fixtures/software-change/profiles.fixture.yaml
  category: mutant
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: no software.change/1 fixture expects an obligation discharged, so pointing any obligation's discharged_when at another claim leaves every fixture green
```
