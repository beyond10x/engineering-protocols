---
format: aep.planning-md/3
id: review-result:trial-review-parallel-safety-r1
kind: review-result
status: active
title: Parallel-safety critic, downstream trial-review set, round 1
relations:
- reviews: epic:gates-and-delivery-for-any-repository
- reviews: story:a-gate-sees-the-change-and-may-not-apply
- reviews: story:story-delivery-2-with-settings
- reviews: story:gate-commands-name-a-program-portably
revision: 1
---
needs-revision
story:a-gate-sees-the-change-and-may-not-apply — its scope claims the directory `crates/assertion-providers/src`. Waves compares paths as plain strings, so it reports no overlap with story:gate-commands-name-a-program-portably on `crates/assertion-providers/src/process.rs` or with any file-scoped story under that directory. The body does not name the new provider file (cited) — .engineering/planning/story/a-gate-sees-the-change-and-may-not-apply.md:17
story:story-delivery-2-with-settings — its scope claims the directory `crates/canon-engineering`, although the built-ins table is `crates/canon-engineering/src/registry.rs:14` (generated at build). Waves therefore misses its overlap with the draft story:protocol-docs-render, which is scoped on `crates/canon-engineering/src/lib.rs` (cited dir, file inferred) — .engineering/planning/story/story-delivery-2-with-settings.md:15

Read: 4 artifacts via `aep plan artifact show` and `waves` (text and yaml); `git grep` in `crates/canon-engineering/src`. Surfaces: 3 cited, 0 inferred, 0 unplaced. Acceptance reads: 0 within the store; the cross-store reads are recorded as dependency-blockers in takt. The gates pair (lib.rs, gates.yaml) is recorded and kept apart, in waves 1 and 2.
Not established: whether the active takt story `story:gate-commands-find-programs-on-a-system-path` will also land on `process.rs` here (outside the set).

```findings
- {file: .engineering/planning/story/a-gate-sees-the-change-and-may-not-apply.md, line: 17, category: parallel-safety, severity: warning, verdict: needs-revision, origin: introduced, message: "scope claims the directory crates/assertion-providers/src, so waves reports no overlap with gate-commands on process.rs or any file-scoped story under it, and the body names no provider file"}
- {file: .engineering/planning/story/story-delivery-2-with-settings.md, line: 15, category: parallel-safety, severity: warning, verdict: needs-revision, origin: introduced, message: "scope claims the directory crates/canon-engineering instead of the built-ins file, so waves misses its overlap with draft story:protocol-docs-render on crates/canon-engineering/src/lib.rs (file inferred)"}
```
