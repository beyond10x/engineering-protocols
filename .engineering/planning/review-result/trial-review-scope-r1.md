---
format: aep.planning-md/3
id: review-result:trial-review-scope-r1
kind: review-result
status: active
title: Scope critic, downstream trial-review set, round 1
relations:
- reviews: epic:gates-and-delivery-for-any-repository
- reviews: story:a-gate-sees-the-change-and-may-not-apply
- reviews: story:story-delivery-2-with-settings
- reviews: story:gate-commands-name-a-program-portably
revision: 1
---
needs-revision
story:story-delivery-2-with-settings — the amended takt `story:story-delivery-tests-first` assigns story.delivery/2 a `pr.open` "which requires a `change.complete` claim: test paths and implementation paths both changed since the base" (takt .engineering/planning/story/story-delivery-tests-first.md:29, :37), but this Outcome declares only `tests_first` and `gate_on`, and `story.delivery/1` has no such claim (protocols/story-delivery/1.yaml:170-174), so no item claims that promise — .engineering/planning/story/story-delivery-2-with-settings.md:30
Read: 4 artifacts in full (the epic and its 3 stories), plus protocols/story-delivery/1.yaml and a scan of `inputs:` across protocols/*. Promises: the epic has 3, all 3 traced. One more promise reaches this store from takt's amended `story:story-delivery-tests-first`, and it is not traced.
Not established: none.
```findings
- file: .engineering/planning/story/story-delivery-2-with-settings.md
  line: 30
  category: scope
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the amended takt story:story-delivery-tests-first assigns story.delivery/2 a pr.open 'which requires a change.complete claim: test paths and implementation paths both changed since the base' (takt .engineering/planning/story/story-delivery-tests-first.md:29, :37), but this Outcome declares only tests_first and gate_on, and story.delivery/1 has no such claim (protocols/story-delivery/1.yaml:170-174), so no item claims that promise"
```
