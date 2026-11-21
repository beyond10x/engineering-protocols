---
format: aep.planning-md/3
id: review-result:data-only-catalog-scope-r2
kind: review-result
status: active
title: data-only-catalog decomposition, scope critic, round 2
relations:
- reviews: story:catalog-manifest
- reviews: story:catalog-release-archive
- reviews: story:assertion-catalog-path
- reviews: story:assertions-frozen-at-0-16-0
- reviews: story:docs-generator-reads-files
- reviews: story:agents-boundary-data-only
- reviews: story:protocol-check-tool
revision: 2
---
approve

**What you read:** 12 artifacts: the epic, the 7 stories, the 2 blockers, the approval record and the round-1 scope review. I read them with `aep plan artifact show <id>`, plus `aep plan artifact graph` and greps of `Cargo.toml` and `crates/*/Cargo.toml`. I also read the request paragraph, `sdc/docs/engineering-protocols-boundary.md:111`, and `README.md` and `AGENTS.md` for the stale lines the stories cite. I wrote the epic's promises down before opening any story. I extracted 10 and traced all 10 to a story.
- Files taken from a tag: story:catalog-manifest.
- Published sha256: story:catalog-manifest and story:catalog-release-archive.
- Check run in CI: story:catalog-manifest.
- Catalog path rename: story:assertion-catalog-path.
- Docs generator plus shared vocabulary reader: story:docs-generator-reads-files.
- Two assertion crates frozen: story:assertions-frozen-at-0-16-0.
- Everything `publish = false`: stories 4, 5 and 7. The docs crate already has it at `crates/canon-engineering-docs/Cargo.toml:7`.
- AGENTS.md and README.md fixes: story:agents-boundary-data-only.
- Registry deleted, one check crate kept: story:protocol-check-tool.
- A Canon release needs no release here: story:protocol-check-tool.

All five request points are claimed. Three cases of a promise left unclaimed are named by the drafter, so none is a gap:
- Deleting the assertion crates is "a later story" in the epic's "Not here" table.
- The assertion reference page is decided in story:docs-generator-reads-files, "Decided here".
- The binary's `gates` and `assertions` commands, in story:protocol-check-tool's "Open".

I found no reach beyond the parent and no two stories claiming one outcome. The round-1 `website/` scope in story:agents-boundary-data-only is gone.

**What you could not establish:**
- Whether Loom's and takt's pins match what the stories say. They come from the boundary document and the blocker, and I did not re-read them.
- Out of my lane (design): story:protocol-check-tool outcome says the check crate "holds" the vocabulary check. It does not say that crate depends on the new vocabulary crate from story:docs-generator-reads-files, which the epic calls "the vocabulary reader it shares with the check". Nothing in the stories forces this link.

```findings
[]
```
