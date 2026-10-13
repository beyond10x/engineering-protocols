---
format: aep.planning-md/3
id: review-result:adversary-w20261013-w3-independence-pass-1
kind: review-result
status: active
title: Wave 2026-10-13-w3 adversary, independence and freshness, pass 1
relations:
- reviews: story:security-independence-rules
- reviews: story:rollback-verification-rule
- reviews: story:stale-evidence-fixtures
revision: 1
---
needs-revision

Adversary (`aep:adversary`) on `decf907..f695bbf` of wave 2026-10-13-w3: story:security-independence-rules, story:rollback-verification-rule, story:stale-evidence-fixtures. Test commit `12ed5cd` (`adversary4_independence_freshness.rs`, 14 cases); red 3. No rule decides wrongly; two rule clauses have no fixture that exercises them.

## Findings

- **F1, a rejected security review is never exercised.** `protocols/software-change/1.yaml:110`: no fixture holds a rejected `security_review`, so dropping `result: approved` from `security.reviewed` survives every fixture and package test, and an independent rejection would verify and merge. Fix: a fixture with an independent rejection (`security.reviewed` FALSE, merge blocked).
- **F2, a partial rollback is never exercised.** `protocols/incident-response/1.yaml:66`: no fixture holds a `partial` `rollback_result`, so dropping `result: complete` from `rollback.verified` survives. Fix: a fixture with a partial result from an independent verifier (`rollback.verified` FALSE).
- **F3, a rollback result about another artifact is never exercised** (`protocols/incident-response/1.yaml:67`); only the structural pin in `incident_response_protocol.rs` item 5 kills the subject-drop mutant.

## Judgement notes (no case)

- `rollback.verified` decides nothing: no obligation, action or outcome of `incident.response/1` reads it; the story asked for the claim only.
- `software.change/1`'s `rolled_back` outcome accepts a complete `rollback_result` from any producer (before this wave).
- `affects_security_boundary` is called fixed when the case opens, but a case writer can flip it between snapshots, as with `risk`; the case writer is trusted.
- The implementer's own rejection does not count against an independent approval (CANON-INDEPENDENCE-001).

## Attacked and not broken

Disagreeing independent reviews (UNKNOWN, merge blocked); an independent rejection alone (FALSE); an independent approval of a stale revision (`revision_mismatch`); producers independent only on `different_agent_run`, without `producer`, or unlisted (`not_independent`); merge at every risk with the boundary true; a partial rollback (FALSE); a complete rollback about the wrong subject or an old release revision (UNKNOWN); an observation exactly 30m old (applies) and 30m+1s (expired) in both protocols; a later state inheriting the per-state instant; per-state `at` reaching Canon; moved fixture times still testing what their tests claim. Mutants killed: `independent_on` dropped or switched, `in: [false]` flipped, `any` to `all`, obligation input flipped, `max_age` dropped in either protocol.

```findings
- file: protocols/software-change/1.yaml
  line: 110
  category: mutant
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: no fixture holds a rejected security_review, so dropping result approved from security.reviewed lets an independent rejection verify and merge with every fixture and package test green
- file: protocols/incident-response/1.yaml
  line: 66
  category: mutant
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: no fixture holds a partial rollback_result, so dropping result complete from rollback.verified lets a partial rollback verify with every fixture and package test green
- file: protocols/incident-response/1.yaml
  line: 67
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: no fixture holds a rollback_result about another artifact; only the structural pin in incident_response_protocol.rs item 5 kills the subject-drop mutant
```
