---
format: aep.planning-md/3
id: review-result:data-only-catalog-design-r1
kind: review-result
status: active
title: data-only-catalog decomposition, design critic, round 1
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
story:docs-generator-reads-files — its first open point lets the vocabulary reader live in the check crate that story:protocol-check-tool keeps, so the generator would need a crate produced by the story that `depends_on` it (a cycle), and a reader left in `crates/canon-engineering` fails its own acceptance (`cargo tree` lists no `b10x-canon-engineering`); the body should settle the reader's home as the generator or a new crate created in this story — .engineering/planning/story/docs-generator-reads-files.md:35-39, .engineering/planning/story/protocol-check-tool.md:48
story:protocol-check-tool — its acceptance requires every package `cargo metadata --no-deps` lists to be `publish = false`, but `b10x-canon-engineering-assertions` and `b10x-assertion-providers` stay in the tree with no `publish = false` and no story in the set adds it (the freeze story's scope has no manifest edit, and this story's scope touches the assertions crate only "for the binary's commands"), so the acceptance reads state nothing produces; add the manifest edit to this story's scope or to the freeze story's — .engineering/planning/story/protocol-check-tool.md:76-77, crates/canon-engineering-assertions/Cargo.toml (no `publish` key)
story:assertions-frozen-at-0-16-0 — the open decision-blocker `blocks` it, yet its acceptance only requires the READMEs to name that blocker and reads no answer, so story:agents-boundary-data-only (a documentation story) and story:protocol-check-tool stall behind an operator decision the freeze does not need; either the acceptance states what it reads from the decision, or the block belongs on the later deletion story and this edge goes — .engineering/planning/story/assertions-frozen-at-0-16-0.md:64-70, `aep plan artifact graph` (`decision-blocker:assertion-executor-owner -> story:assertions-frozen-at-0-16-0 [blocks]`, `story:agents-boundary-data-only -> story:assertions-frozen-at-0-16-0 [depends_on]`)

What I read: 9 artifacts (the epic, 7 stories, 2 blockers) with `aep plan artifact show`, plus `relations`, `graph` and `validate`; I walked about 20 edges including those to the epic, `vision:O2` and the two blockers, and found no cycle among declared edges. I traced 9 acceptance reads to a producer in the set, and an edge records 8 of them; the unrecorded one is the `publish = false` read above.

What I could not establish:
- Whether three stories editing AGENTS.md § Releases (archive, agents-boundary, check-tool) need an ordering edge — out of my lane, parallel-safety.
- Whether the epic outcome "both `publish = false`" fits the frozen crates that takt still consumes — out of my lane, scope.

```findings
- file: .engineering/planning/story/docs-generator-reads-files.md
  line: 35
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the first open point lets the vocabulary reader live in the check crate story:protocol-check-tool keeps, so the generator would need a crate produced by the story that depends_on it (a cycle), and a reader left in crates/canon-engineering fails the story's own cargo tree acceptance; the body should settle the reader's home as the generator or a new crate created in this story"
- file: .engineering/planning/story/protocol-check-tool.md
  line: 76
  category: design
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance requires every workspace package to be publish = false, but the two assertion crates stay publishable and no story in the set adds publish = false to them, so the acceptance reads state nothing produces; add the manifest edit to this story's scope or the freeze story's"
- file: .engineering/planning/story/assertions-frozen-at-0-16-0.md
  line: 64
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the open decision-blocker blocks this story but its acceptance reads no answer from the decision, so story:agents-boundary-data-only and story:protocol-check-tool stall transitively behind an operator decision the freeze does not need; either the acceptance states what it reads from the decision or the block moves to the later deletion story"
```
