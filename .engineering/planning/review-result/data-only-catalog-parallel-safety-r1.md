---
format: aep.planning-md/3
id: review-result:data-only-catalog-parallel-safety-r1
kind: review-result
status: active
title: data-only-catalog decomposition, parallel-safety critic, round 1
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
story:catalog-release-archive — its AGENTS.md § Releases step and README.md edit land on the same section and file as story:agents-boundary-data-only (§ Releases, README.md), story:assertions-frozen-at-0-16-0 (AGENTS.md, README.md) and story:protocol-check-tool (§ Releases, Taskfile.yml), with no `depends_on` edge to any of the three and no body naming them (cited) — .engineering/planning/story/catalog-release-archive.md:47, .engineering/planning/story/agents-boundary-data-only.md:57, .engineering/planning/story/protocol-check-tool.md:70
story:assertions-frozen-at-0-16-0 — its AGENTS.md edit shares a file with story:catalog-manifest's § Invariants row, and neither body names the other nor is an edge between them (cited) — .engineering/planning/story/assertions-frozen-at-0-16-0.md:61, .engineering/planning/story/catalog-manifest.md:65

Both findings need the same decision from you. Either add an ordering edge that records the shared file as its reason, or split the surface so the two items no longer share it. `aep plan artifact waves` already puts these pairs in different waves, but only because it derives that from scope. No body or edge states it.

What I read: 9 artifacts (7 stories, 2 blockers) plus epic:data-only-catalog. Commands run: `aep plan artifact waves --kind story --status draft`, `aep plan artifact show` on each, and `aep plan artifact list --format json`. I also ran `git grep` over `crates/`, `Taskfile.yml`, `website/` and `.github/`, and read `generate.rs`, `lint.rs` and the crate manifests.
Surfaces: 7 items cited, 0 inferred-only, 0 unplaceable. Some individual scope lines are inferred: `protocols/manifest.yaml` (a file that does not exist yet), `Taskfile.yml`, `.github/workflows/check.yml`, `website/docs/index.md` and `website/product.json`. No finding rests on one of them.
Acceptance reads traced to a producer in the set: 6. All 6 are ordered: 5 by a direct `depends_on` edge and 1 (story:protocol-check-tool reading the rows story:assertions-frozen-at-0-16-0 writes in AGENTS.md) only through story:agents-boundary-data-only.

What I could not establish:
- Out of your lane (acceptance/scope), not counted in the verdict: story:assertions-frozen-at-0-16-0 requires "both crates' READMEs" in its acceptance. `crates/canon-engineering-assertions/README.md` does not exist and the scope says "if present".
- Out of your lane (design): story:docs-generator-reads-files offers "the generator depends on the check crate story:protocol-check-tool keeps" as one option. That would create a cycle with story:protocol-check-tool's `depends_on` edge to it.
- The `Cargo.lock` / `Cargo.toml` overlap is not a finding. No scope other than story:protocol-check-tool's names `Cargo.lock`, and story:catalog-manifest needs no new dependency, so I found no second writer.

```findings
- file: .engineering/planning/story/catalog-release-archive.md
  line: 47
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "its AGENTS.md § Releases step and README.md edit land on the same section and file as story:agents-boundary-data-only (§ Releases, README.md), story:assertions-frozen-at-0-16-0 (AGENTS.md, README.md) and story:protocol-check-tool (§ Releases, Taskfile.yml), with no depends_on edge to any of the three and no body naming them (cited)"
- file: .engineering/planning/story/assertions-frozen-at-0-16-0.md
  line: 61
  category: parallel-safety
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "its AGENTS.md edit shares a file with story:catalog-manifest's § Invariants row (catalog-manifest.md:65), and neither body names the other nor is an edge between them (cited)"
```
