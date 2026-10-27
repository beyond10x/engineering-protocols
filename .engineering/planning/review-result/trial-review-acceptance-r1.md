---
format: aep.planning-md/3
id: review-result:trial-review-acceptance-r1
kind: review-result
status: active
title: Acceptance critic, downstream trial-review set, round 1
relations:
- reviews: epic:gates-and-delivery-for-any-repository
- reviews: story:a-gate-sees-the-change-and-may-not-apply
- reviews: story:story-delivery-2-with-settings
- reviews: story:gate-commands-name-a-program-portably
revision: 1
---
needs-revision
epic:gates-and-delivery-for-any-repository — the body has no `## Acceptance` between `## Stories` and `## Out of scope`, so nothing says when the epic is done — .engineering/planning/epic/gates-and-delivery-for-any-repository.md:43
story:a-gate-sees-the-change-and-may-not-apply — scenario 3 names `tests/gates_result_values.rs` with no crate, and the repository has no root `tests/`, so nobody can say where the check lives — .engineering/planning/story/a-gate-sees-the-change-and-may-not-apply.md:63
story:gate-commands-name-a-program-portably — scenarios 2 to 4 name `tests/process_program.rs` with no crate, and the repository has no root `tests/` — .engineering/planning/story/gate-commands-name-a-program-portably.md:52
story:gate-commands-name-a-program-portably — no scenario covers `program:` with no `search_path`, which the body says keeps today's refusal of a relative executable — .engineering/planning/story/gate-commands-name-a-program-portably.md:43
Read: 4 of 4 ids with `aep plan artifact show`, `validate` (valid), `cat -n`, and `ls`/`git grep` on `fixtures/assertions/`, `crates/*/tests` and `STATE_BOUND`.
Not established: whether `task check` would pick up a new `fixtures/assertions/*.fixture.yaml`. Today only `fixtures/assertions/gates.yaml` is loaded, by explicit path from `crates/canon-engineering/tests/`.
```findings
- {file: .engineering/planning/epic/gates-and-delivery-for-any-repository.md, line: 43, category: acceptance, severity: warning, verdict: needs-revision, origin: introduced, message: "the body has no Acceptance section between Stories and Out of scope, so nothing says when the epic is done"}
- {file: .engineering/planning/story/a-gate-sees-the-change-and-may-not-apply.md, line: 63, category: acceptance, severity: warning, verdict: needs-revision, origin: introduced, message: "scenario 3 names tests/gates_result_values.rs with no crate, and the repository has no root tests/, so nobody can say where the check lives"}
- {file: .engineering/planning/story/gate-commands-name-a-program-portably.md, line: 52, category: acceptance, severity: warning, verdict: needs-revision, origin: introduced, message: "scenarios 2 to 4 name tests/process_program.rs with no crate, and the repository has no root tests/"}
- {file: .engineering/planning/story/gate-commands-name-a-program-portably.md, line: 43, category: acceptance, severity: warning, verdict: needs-revision, origin: introduced, message: "no scenario covers program: with no search_path, which the body says keeps today's refusal of a relative executable"}
```
