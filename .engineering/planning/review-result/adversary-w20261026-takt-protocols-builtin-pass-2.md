---
format: aep.planning-md/3
id: review-result:adversary-w20261026-takt-protocols-builtin-pass-2
kind: review-result
status: active
title: Adversary pass 2 on story:takt-protocols-builtin
relations:
- reviews: story:takt-protocols-builtin
revision: 1
---
unit: story:takt-protocols-builtin
verdict: green
cases: executed 286→289, red 0
origin: introduced 2, pre-existing 0, undecided 0
wrote-outside-worktree: the session scratch root's unit-takt-protocols/adv2/ (case-run-1.log, case-run-2.log, suite-run.log, clippy.log, commit.log; head.rs, tail.rs and the commit message file were created there and deleted)
needs-coordinator: no

The new fixtures kill every mutant from both passes; I found no new red case. Two findings remain: invented comment text in four fixtures (warning) and a stale test comment about AGENTS.md (note). Findings cover 3aa5211 plus my commit 2420c1d. The bot made the commit (`%an|%cn` = `b10x-bot[bot]|b10x-bot[bot]`); nothing was pushed.

**1. Diff stat** (`git diff --stat 3aa5211..2420c1d`), test file only:
```
 .../tests/adversary2_takt_protocols.rs             | 285 +++++++++++++++++++++
```

**2. New mutant classes:** `crates/canon-engineering/tests/adversary2_takt_protocols.rs`, same mechanism as pass 1, 34 mutants. The new classes: an evidence match's subject or kind swapped; a capability swapped for another one the fixtures also grant; `all` and `any` swapped; a precondition swapped for another claim; `independent_on` added where the protocol has none; an outcome's result swapped to `error`; `is: unknown` changed to `is: false`; a precondition added to `tests.run`.

| Test | Result |
|---|---|
| story-delivery, 10 mutants | all killed |
| self-improvement, 14 mutants | all killed |
| loop-improvement, 10 mutants | all killed |

One mutant I first wrote was invalid: it added `independent_on` to a claim in story-delivery, which declares no independence dimension. I replaced it with a precondition swap. None of these cases is red; I committed them so the coverage stays in the suite.

**3. Suite:** `cargo test --workspace --locked --no-fail-fast`, exit 101. 287 passed, 2 failed, both in `ess_contract` (known). All four pass-1 cases are now green. `generate --check`: "30 generated files fresh (8 protocols)". `fmt --check` and `clippy -D warnings`: both exit 0.

**4. Findings**
- **G1 (warning).** Four fixtures have comments that state how evidence gets produced, and takt 0.12.0's own code says otherwise: `precondition-gaps:15` "a job that diffs every pushed revision" (only `change.read` writes `change_record`, `validator.rs:3739`); `precondition-gaps:17` "CI runs on every push" (`ci.read` writes nothing when there is no pull request, `validator.rs:2518`); `scored-unmerged:8` "a nightly job" (only `scorecard.compare` writes `scorecard_result`, `improvement.rs:841`); `promoted-with-regression:7` "installed it by hand" (the promotion record comes only from the `binary.promote` validator, and only with the grant, `improvement.rs:1663`). The expectations themselves are correct against the protocol text; only the comments are wrong. I don't know whether the "by hand" merges in `promoted-unverified:7` and `accepted-gaps:11` are realistic.
- **G2 (note).** The doc comment at `adversary_takt_protocols.rs:452` still quotes the old two-exception O1 text; on this branch AGENTS.md:12 still names only two exceptions. The wave tree's uncommitted AGENTS.md names all six, so this resolves once that is committed.

**5. Attacked and could not break:** my pass-1 mutant lists are unchanged; the state counts 21/28/16 equal the old counts plus the new states, and the fixture lists are asserted equal to the directories; each new state leaves exactly one conjunct unestablished.

**6. Paths written outside the worktree:** the logs in the header; mutant copies in the worktree's `target/tmp/adversary2-takt-mutants/` (ignored by git).

(Coordinator's note: home-directory paths shortened. G1 and G2 answered by the coordinator in fa5b460, comments only, expectations unchanged, read by the coordinator as the wave procedure requires for the correction answering a second pass.)

```findings
[
  {"file": "fixtures/story-delivery/precondition-gaps.fixture.yaml", "line": 15, "category": "judgement", "severity": "warning", "verdict": "CONFIRMED", "origin": "introduced", "message": "Comments in four new fixtures (precondition-gaps:15 and :17, scored-unmerged:8, promoted-with-regression:7) name evidence producers (a diff job on every push, CI on every push, a nightly scorecard job, an operator's hand install) that takt 0.12.0's validators contradict; the expectations match the protocol text, but the states are constructed, not observed."},
  {"file": "crates/canon-engineering/tests/adversary_takt_protocols.rs", "line": 452, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The widened EXCEPTED list (eight entries) is not tied to AGENTS.md, and the test's doc comment still quotes the two-exception O1 text that AGENTS.md:12 carries on this branch, until the wave tree's uncommitted AGENTS.md edit lands."}
]
```
