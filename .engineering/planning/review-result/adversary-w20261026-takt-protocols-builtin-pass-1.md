---
format: aep.planning-md/3
id: review-result:adversary-w20261026-takt-protocols-builtin-pass-1
kind: review-result
status: active
title: Adversary pass 1 on story:takt-protocols-builtin
relations:
- reviews: story:takt-protocols-builtin
revision: 1
---
unit: story:takt-protocols-builtin
verdict: red
cases: executed 282→286, red 4
origin: introduced 5, pre-existing 0, undecided 0
wrote-outside-worktree: the session scratch root's unit-takt-protocols/adv1/ (case-run-1.log, case-run-2.log, suite-run.log, docs-check.log, fmt.log, clippy.log, commit.log)
needs-coordinator: yes

My test case is commit `449ddd3` on `impl/takt-protocols-builtin`. The bot made it: `git log --format='%an|%cn' -1` gives `b10x-bot[bot]|b10x-bot[bot]`. Nothing was pushed. The findings below cover commit 7d960a2, with my case on top.

**1. Diff stat** (`git diff --stat 7d960a2..449ddd3`), test file only:
```
 .../tests/adversary_takt_protocols.rs              | 482 +++++++++++++++++++++
 1 file changed, 482 insertions(+)
```

**2. Cases added** in `crates/canon-engineering/tests/adversary_takt_protocols.rs`. All four are red now and were red on their first run, before the suite ran.

The three mutant tests work like this. Each takes one rule of a protocol, writes a changed copy into `CARGO_TARGET_TMPDIR`, compiles it through the test harness, and runs every fixture of that protocol against it. A mutant passes the test if at least one fixture disagrees with it. Before any mutant runs, the test asserts that every fixture agrees with the unchanged protocol, so a kill is a real difference.

| Test | Mutants the fixtures miss |
|---|---|
| `adversary_story_delivery_fixtures_kill_every_rule_mutant` | 5 of 20: `pr.open` without `change.handed_over`, without `tests.pass`, without `gate.green`; `repository.merge` without `pr.opened`; `tests.required` met by `gate.green` |
| `adversary_self_improvement_fixtures_kill_every_rule_mutant` | 4 of 26: `binary.promote` without `fix.merged`; `candidate.merge_judge_surface` without `scenario.passes_after`; `accepted` without `scorecard.no_regression`; `accepted` without `scenario.passes_after` |
| `adversary_loop_improvement_fixtures_kill_every_rule_mutant` | 2 of 20: `accepted` without `loop.definition_honest`; `accepted` without `loop.change_merged` |
| `adversary_every_builtin_write_action_names_its_capability` | 6 write actions name no capability: story-delivery `implementation.edit`, `pr.open`, `tests.write`; self-improvement `finding.file`; loop-improvement `loop.change.deliver`, `loop.finding.file` |

Red output from the run of this file alone (`case-run-1.log`):
```
5 of 20 mutants of story-delivery@1 pass every fixture under fixtures/story-delivery/:
4 of 26 mutants of self-improvement@1 pass every fixture under fixtures/self-improvement/:
2 of 20 mutants of loop-improvement@1 pass every fixture under fixtures/loop-improvement/:
write actions that name no capability, outside AGENTS.md's two exceptions:
test result: FAILED. 0 passed; 4 failed
```

**3. Suite run:** `cargo test --workspace --locked --no-fail-fast`, exit 101.
- 280 passed, 6 failed.
- 4 of the failures are my cases.
- The other 2 are in `ess_contract`, which the brief already names as not mine.
- `generate --check` exit 0: "30 generated files fresh (8 protocols)".
- `fmt --check` and `clippy -D warnings` on my file: both exit 0.

**4. Findings**
- **F1 (gap).** The fixtures cannot tell `pr.open`'s `tests.pass` and `gate.green` apart: `checks-failed.fixture.yaml:21` fails both together. A realistic state reaches this: tests pass and the gate fails.
- **F2 (gap).** No state expects `tests.required` to differ from `gate.required`, so the same tests-pass, gate-fail state is missing.
- **F3 (gap).** `judge-surface` has no state where `scenario.passes_after` is not true. The not-independent check in the existing test cannot catch it either, because that fixture uses `surface: non_judge`.
- **F4 (defect).** AGENTS.md:12 promises that every write action names its capability, with two exceptions for `software.change`. Six actions in the new protocols break that promise. The protocols must stay byte-identical to takt, so the fix is an AGENTS.md exception line or a takt story. Both are outside this unit's scope, which is why the header says needs-coordinator: yes.
- **F5 (note).** The other 7 surviving mutants are reached only when evidence arrives that the protocol's own actions did not produce. For example, a `change_record` exists only after `change.read`, which needs a handover. Nothing I found reaches them.

**5. Attacked and could not break**
- Blob ids match `git rev-parse 0.12.0:...` and `git hash-object` for all three files. The in-test SHA-1 matches git's known values, and a one-byte change turns it red.
- `NOT_SPELLABLE` and `OVER_BOUND` are asserted exactly in both directions.
- The vocabulary walk covers every category. Canon 0.6.0 only allows a decision as an outcome's whole requirement (`model/requirement.rs:13`), so none can hide in a nested term.
- Every `Marking` match handles the new values, and they round-trip through serde. No consumer in loom or takt matches on `Marking`.
- The widened meanings of `baseline`, `candidate` and `declined` still hold for every protocol that uses them.

**6. Paths written outside the worktree:** the scratch logs in the header. Inside the worktree I also wrote mutant copies to `target/tmp/adversary-takt-mutants/` (ignored by git).

(Coordinator's note: home-directory paths shortened to the scratch root; the findings block below is the adversary's corrected JSON form, sent after `aep` refused its first YAML shape.)

```findings
[
  {"file": "fixtures/story-delivery/checks-failed.fixture.yaml", "line": 21, "category": "mutant", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Every fixture fails the tests and the gate together, so pr.open (protocols/story-delivery/1.yaml:165) still passes every fixture with either tests.pass or gate.green dropped from its precondition."},
  {"file": "protocols/story-delivery/1.yaml", "line": 108, "category": "mutant", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "No state expects tests.required and gate.required to differ, so tests.required discharged by gate.green passes every fixture."},
  {"file": "protocols/self-improvement/1.yaml", "line": 157, "category": "mutant", "severity": "warning", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "No judge-surface state lacks scenario.passes_after, so candidate.merge_judge_surface without that precondition passes every fixture."},
  {"file": "AGENTS.md", "line": 12, "category": "contract-drift", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Six write actions of the takt protocols name no capability (story-delivery pr.open, tests.write, implementation.edit; self-improvement finding.file; loop-improvement loop.finding.file, loop.change.deliver), against O1's rule that allows only software.change/1's two exceptions."},
  {"file": "protocols/self-improvement/1.yaml", "line": 192, "category": "mutant", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "Seven more mutants survive (the accepted and binary.promote conjuncts here, the accepted conjuncts at protocols/loop-improvement/1.yaml:145, and story-delivery pr.open without change.handed_over and repository.merge without pr.opened), but each needs evidence the protocol's own actions never produce, and nothing found reaches that."}
]
```
