---
format: aep.planning-md/3
id: review-result:data-only-catalog-acceptance-r1
kind: review-result
status: active
title: data-only-catalog decomposition, acceptance critic, round 1
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
story:agents-boundary-data-only — the acceptance's grep pattern `registry::get\|pins a release of` matches only AGENTS.md:39-40 and README.md:78 today, so it passes while AGENTS.md:209 ("Consumers pin `b10x-canon-engineering` by `tag`") and the README.md Install lines 46-56 still tell a consumer to compile against the crate; the acceptance needs a pattern or check that fails on those lines too, not the judgement "no line that tells a consumer" — .engineering/planning/story/agents-boundary-data-only.md:65
story:assertions-frozen-at-0-16-0 — the acceptance requires "both crates' READMEs" to state the freeze, but `crates/canon-engineering-assertions/README.md` does not exist (only `crates/assertion-providers/README.md` does), and the Scope hedges it as "if present", so the acceptance cannot be met as written; it should either require creating that README or name only the files that exist — .engineering/planning/story/assertions-frozen-at-0-16-0.md:66
story:protocol-check-tool — the acceptance "every row of AGENTS.md § Invariants names a test that exists" cannot pass, because the "Held by" column of two rows is not a test (AGENTS.md:78 "adding the fixture is review's job", AGENTS.md:79 "…; review"); it must exempt those rows or require a different check — .engineering/planning/story/protocol-check-tool.md:77
story:protocol-check-tool — the acceptance "every package it lists is `publish = false`" holds today only for `canon-engineering-docs`; `b10x-canon-engineering-assertions` and `b10x-assertion-providers` carry no `publish` key, and their manifests are not in the Scope, so the sentence demands an edit the body never schedules — crates/canon-engineering-assertions/Cargo.toml:1 (no `publish` key; `grep -n publish Cargo.toml crates/*/Cargo.toml` finds it only in crates/canon-engineering-docs/Cargo.toml:7)
story:catalog-release-archive — the acceptance's "the digest attached to the release equals the digest of the attached archive" is checkable only on a real GitHub Release made after this story, yet the story's scope is a `task` target and documentation and never says it cuts a release, so it cannot be closed from its own work; it needs either a release in scope or a check on the task target's local archive-and-digest output — .engineering/planning/story/catalog-release-archive.md:54
story:docs-generator-reads-files — the acceptance clause "`vocabulary_yaml_is_the_source` still passes against the one reader" does not detect a second vocabulary reader, because that test passes with two; the body does not say what shows there is one (for example a `git grep` for the `Vocabulary` type definition printing one path) — .engineering/planning/story/docs-generator-reads-files.md:59

What I read: 7 of 7 artifacts (`aep plan artifact show` for each id), plus `git grep` and `ls` checks of AGENTS.md, README.md, the crate manifests, `protocols/` and `catalog/` in the worktree.

What I could not establish:
- story:catalog-manifest and story:assertion-catalog-path hold up: each names one observable outcome, and the tree matches the facts they cite (10 protocol files, `catalog/engineering.yaml` read at `lib.rs:26`).
- Out of your lane, not counted in the verdict: story:catalog-release-archive reads "required artifacts" in AGENTS.md § Release completion as already binding, which touches release-rule coupling (design).
- Out of your lane: story:agents-boundary-data-only depends on story:catalog-manifest but also edits the same AGENTS.md sections as story:protocol-check-tool and story:assertions-frozen-at-0-16-0, a shared-surface matter for parallel-safety.

```findings
- file: .engineering/planning/story/agents-boundary-data-only.md
  line: 65
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance's grep pattern `registry::get\\|pins a release of` matches only AGENTS.md:39-40 and README.md:78 today, so it passes while AGENTS.md:209 and the README.md Install lines 46-56 still tell a consumer to compile against the crate; the acceptance needs a pattern or check that fails on those lines too, not the judgement 'no line that tells a consumer'"
- file: .engineering/planning/story/assertions-frozen-at-0-16-0.md
  line: 66
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance requires both crates' READMEs to state the freeze, but crates/canon-engineering-assertions/README.md does not exist and the Scope hedges it as 'if present', so the acceptance cannot be met as written; it should require creating that README or name only the files that exist"
- file: .engineering/planning/story/protocol-check-tool.md
  line: 77
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance 'every row of AGENTS.md § Invariants names a test that exists' cannot pass, because the 'Held by' column of two rows is not a test (AGENTS.md:78 and AGENTS.md:79 end in 'review'); it must exempt those rows or require a different check"
- file: crates/canon-engineering-assertions/Cargo.toml
  line: 1
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: "the acceptance 'every package it lists is publish = false' holds today only for canon-engineering-docs; b10x-canon-engineering-assertions and b10x-assertion-providers have no publish key and their manifests are not in the story's Scope, so the sentence demands an edit the body never schedules"
- file: .engineering/planning/story/catalog-release-archive.md
  line: 54
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the acceptance 'the digest attached to the release equals the digest of the attached archive' is checkable only on a real GitHub Release made after this story, yet the scope is a task target and documentation and never says it cuts a release; it needs either a release in scope or a check on the task target's local archive-and-digest output"
- file: .engineering/planning/story/docs-generator-reads-files.md
  line: 59
  category: acceptance
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: "the clause 'vocabulary_yaml_is_the_source still passes against the one reader' does not detect a second vocabulary reader, because that test passes with two; the body does not say what shows there is one"
```
