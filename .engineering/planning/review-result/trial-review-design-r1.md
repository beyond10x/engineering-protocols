---
format: aep.planning-md/3
id: review-result:trial-review-design-r1
kind: review-result
status: active
title: Design critic, downstream trial-review set, round 1
relations:
- reviews: epic:gates-and-delivery-for-any-repository
- reviews: story:a-gate-sees-the-change-and-may-not-apply
- reviews: story:story-delivery-2-with-settings
- reviews: story:gate-commands-name-a-program-portably
revision: 1
---
needs-revision
story:story-delivery-2-with-settings — its outcome declares only `tests_first` and `gate_on`. Takt's story:story-delivery-tests-first (line 29) says story.delivery/2's `pr.open` also requires a `change.complete` claim, which story.delivery/1 does not have, and that story cannot edit the protocol file. So half of the protocol is described only in a story outside this repository — .engineering/planning/story/story-delivery-2-with-settings.md:30, protocols/story-delivery/1.yaml:173
story:gate-commands-name-a-program-portably — it leaves open whether takt's bare-name default lives in this binding or in takt, so it and takt's active story:gate-commands-find-programs-on-a-system-path both own how a gate resolves a program name. Takt's acceptance (a bare `sh` with no `env` runs) cannot pass on a binding with no `search_path`, and no takt dependency-blocker records that. This story has to take the decision, and takt has to record that blocker — .engineering/planning/story/gate-commands-name-a-program-portably.md:39
Read: 4 artifacts, plus takt's two counterpart stories and `protocols/story-delivery/1.yaml`. The set has no edges among its own items, so there is no cycle. I traced 2 acceptance reads; both are fixture-local and need no edge.
Not established: none.
```findings
- {file: .engineering/planning/story/story-delivery-2-with-settings.md, line: 30, category: design, severity: blocker, verdict: needs-revision, origin: introduced, message: "its outcome declares only tests_first and gate_on, while takt's story-delivery-tests-first says story.delivery/2's pr.open requires a change.complete claim that story.delivery/1 lacks, so half of the protocol is described only in a story that cannot edit it"}
- {file: .engineering/planning/story/gate-commands-name-a-program-portably.md, line: 39, category: design, severity: warning, verdict: needs-revision, origin: introduced, message: "it leaves open whether takt's bare-name default lives in this binding or in takt, so it and takt's active gate-commands-find-programs-on-a-system-path both own program resolution, and no takt dependency-blocker records that takt's acceptance needs this binding"}
```
