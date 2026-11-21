---
format: aep.planning-md/3
id: review-result:data-only-catalog-acceptance-r2
kind: review-result
status: active
title: data-only-catalog decomposition, acceptance critic, round 2
relations:
- reviews: story:catalog-manifest
- reviews: story:catalog-release-archive
- reviews: story:assertion-catalog-path
- reviews: story:assertions-frozen-at-0-16-0
- reviews: story:docs-generator-reads-files
- reviews: story:agents-boundary-data-only
- reviews: story:protocol-check-tool
revision: 1
---
needs-revision
story:assertion-catalog-path — the acceptance never names the new path or requires it to name assertions, so the moved file's location is unchecked and a move to any path other than `catalog/` passes: "the moved file has the sha256 `catalog/engineering.yaml` has at tag 0.16.0" — .engineering/planning/story/assertion-catalog-path.md:52
story:assertions-frozen-at-0-16-0 — "a listing of each crate's public items" names no command or tool that produces the listing, so two readers produce different listings and "no line added" has no stable meaning — .engineering/planning/story/assertions-frozen-at-0-16-0.md:84
story:agents-boundary-data-only — the acceptance is only a grep that must print nothing plus AGENTS.md checks, so deleting README.md § Install and § Smallest example passes while the Outcome's "a runtime takes the files from a tag or the release archive and checks them against the manifest" is never required in README.md, which story:catalog-release-archive's Order assigns to this story — .engineering/planning/story/agents-boundary-data-only.md:62
story:protocol-check-tool — the acceptance cannot tell whether the `gates` and `assertions` commands of the `canon-engineering` binary survive, so both open end states pass, including the one its Open section says "needs the operator's word": nothing runs or lists those commands — .engineering/planning/story/protocol-check-tool.md:84
story:protocol-check-tool — the scope edits AGENTS.md § Work (the target-directory rule) and § Releases, but the acceptance checks only § Invariants rows, so the story can close with those two sections still describing a registry crate and a `build.rs` embedding that no longer exist — .engineering/planning/story/protocol-check-tool.md:84

What I read: 7 artifacts of 7, with `aep plan artifact show` on each id. I checked the repo claims against the tree with `git grep`, `ls`, and `grep`. The 7-line grep count, the 10 protocol documents, the 63 test files, the 115 fixture files, AGENTS.md:78 and 79, and the single `pub struct Vocabulary` all match. I did not run `task check`.

What I could not establish:
- Whether the Open choices (the new catalog path, the check crate's name, `build.rs` embedding versus runtime reads) will be settled before work starts. I judged only what the acceptance can observe.
- Out of my lane (design, scope and parallel-safety): several stories edit AGENTS.md and README.md, and I did not judge their ordering.
- Out of my lane: `story:docs-generator-reads-files` leaves the generator depending on the frozen assertions crate. I did not judge that coupling.

```findings
- file: .engineering/planning/story/assertion-catalog-path.md
  line: 52
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance never names the new path or requires it to name assertions, so the moved file's location is unchecked and a move to any path other than catalog/ passes: 'the moved file has the sha256 catalog/engineering.yaml has at tag 0.16.0'"
- file: .engineering/planning/story/assertions-frozen-at-0-16-0.md
  line: 84
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "'a listing of each crate's public items' names no command or tool that produces the listing, so two readers produce different listings and 'no line added' has no stable meaning"
- file: .engineering/planning/story/agents-boundary-data-only.md
  line: 62
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance is only a grep that must print nothing plus AGENTS.md checks, so deleting README.md Install and Smallest example passes while the Outcome's README statement of how a runtime takes the files from a tag or the release archive and checks them against the manifest is never required, though story:catalog-release-archive assigns it to this story"
- file: .engineering/planning/story/protocol-check-tool.md
  line: 84
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance cannot tell whether the gates and assertions commands of the canon-engineering binary survive, so both open end states pass, including the one the Open section says needs the operator's word"
- file: .engineering/planning/story/protocol-check-tool.md
  line: 84
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the scope edits AGENTS.md Work (the target-directory rule) and Releases, but the acceptance checks only Invariants rows, so the story can close with those sections still describing a registry crate and build.rs embedding that no longer exist"
```
