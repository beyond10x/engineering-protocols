---
format: aep.planning-md/3
id: review-result:adversary-w20261013-w2-red-first-pass-1
kind: review-result
status: active
title: Wave 2026-10-13-w2 adversary, story:evidence-order-red-first, pass 1
relations:
- reviews: story:evidence-order-red-first
revision: 1
---
needs-revision

Adversary (`aep:adversary`) on `ec83ed8..cdcffee` of wave 2026-10-13-w2, story:evidence-order-red-first. Test commit `c6b8c3e` (`adversary3_red_first_order.rs`, 6 cases; `adversary3_red_first_mutants.rs`, 3 cases); red 4. Excerpt of its report: the findings and what it could not break. The 7 failures from blocker:graph-schema-evidence-order were excluded by the brief.

## Findings

- **F1, the obligation can be met by a test that never failed.** `protocols/software-change/1.yaml:130`: `regression.shown_first` is only `before{regression_test_result, implementation_change}`. A suite `test_result` fail on the current revision admits the edit (ADR 0084 Decision 1), the edit records `implementation_change`, and a regression test whose first result passed discharges `red_first.required`. Red at `adversary3_red_first_order.rs:180` and `:246`, exit 101. Fix named: require `first{regression_test_result, fail}` beside the `before`.
- **F2, the concept page drifts.** `website/docs/concepts/step-order.mdx:86-87` says a regression test that passed first leaves `red_first.required` open, and `:101` marks fail-first-before-change as shipped; F1's case contradicts both.
- **F3, F4, the subjects of both order forms are untested.** Dropping `subject: implementation` from the `first` arm of `regression.reproduced` (`adversary3_red_first_mutants.rs:75`) or from the `before` of `regression.shown_first` (`:91`) survives every software-change fixture; no fixture records either kind about another artifact.
- **F5 (note, infeasible as a case).** `before` sees only recorded `implementation_change` evidence, which `implementation.edit` may but need not produce, so a revision moved without that record before the first red run can still discharge `red_first.required`.

## Attacked and not broken

Ties at the earliest instant (UNKNOWN, edit blocked, obligation open); a record without `observed_at` (UNKNOWN); a red run about the release (reproduces nothing); the harness passing each entry's instant as `observed_at` (no protocol declares freshness, no fixture's `at` changed meaning); `chg-1842` moving from FALSE to UNKNOWN (edit blocked either way); graph refusal reaching every condition site through `not`, `any`, `all`; page wording of both forms; profile nesting per change kind; swapped `before`, wrong `applicable_when`, discharge on `regression.reproduced`, dropped `test_result` arm, flipped `first` result, each killed. Canon's stated limit (a red about a revision never held counts) confirmed, not a finding.

```findings
- file: protocols/software-change/1.yaml
  line: 130
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: regression.shown_first reads only that the first regression_test_result preceded the first implementation_change, so a regression test whose first result passed discharges red_first.required (red at adversary3_red_first_order.rs:180 and :246)
- file: website/docs/concepts/step-order.mdx
  line: 86
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the page says a regression test that passed first leaves red_first.required open for good and marks a fail-first rule as shipped, but the obligation is discharged in that history
- file: crates/canon-engineering/tests/adversary3_red_first_mutants.rs
  line: 75
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: dropping subject implementation from the first arm of regression.reproduced survives every software-change fixture
- file: crates/canon-engineering/tests/adversary3_red_first_mutants.rs
  line: 91
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: dropping subject implementation from the before form of regression.shown_first survives every software-change fixture
- file: protocols/software-change/1.yaml
  line: 130
  category: judgement
  severity: note
  verdict: INFEASIBLE
  origin: introduced
  message: before sees only recorded implementation_change evidence, which implementation.edit may but need not produce, so a revision moved without that record before the first red run can still discharge red_first.required
```
